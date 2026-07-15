use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::timeout,
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const ABORT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_PROMPT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

pub type SharedPiClient = Arc<Mutex<Option<PiProcess>>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserRequest {
    Prompt(String),
    Bash {
        command: String,
        exclude_from_context: bool,
    },
}

impl UserRequest {
    pub fn from_input(input: String) -> Self {
        let input = input.trim();

        let bash = if let Some(command) = input.strip_prefix("!!") {
            Some((command, true))
        } else {
            input.strip_prefix('!').map(|command| (command, false))
        };

        if let Some((command, exclude_from_context)) = bash {
            let command = command.trim();
            if !command.is_empty() {
                return Self::Bash {
                    command: command.to_owned(),
                    exclude_from_context,
                };
            }
        }

        Self::Prompt(input.to_owned())
    }

    fn timeout_subject(&self) -> &'static str {
        match self {
            Self::Prompt(_) => "Pi",
            Self::Bash { .. } => "The shell command",
        }
    }
}

#[derive(Debug)]
pub enum RequestOutcome {
    Prompt,
    Bash(BashOutcome),
}

#[derive(Debug)]
pub struct BashOutcome {
    pub output: String,
    pub exit_code: Option<i32>,
    pub cancelled: bool,
    pub truncated: bool,
    pub full_output_path: Option<String>,
}

pub fn new_client() -> SharedPiClient {
    Arc::new(Mutex::new(None))
}

pub async fn run(
    client: &SharedPiClient,
    request: UserRequest,
    mut on_delta: impl FnMut(&str),
) -> Result<RequestOutcome, String> {
    let mut client = client.lock().await;

    if client.is_none() {
        let process = timeout(STARTUP_TIMEOUT, PiProcess::start())
            .await
            .map_err(|_| "the Pi SDK bridge did not start within 30 seconds".to_owned())??;
        *client = Some(process);
    }

    let prompt_timeout = configured_prompt_timeout();
    let timeout_subject = request.timeout_subject();
    let (result, stop_process) = match timeout(
        prompt_timeout,
        client
            .as_mut()
            .expect("Pi process was initialized")
            .run(&request, &mut on_delta),
    )
    .await
    {
        Ok(result) => {
            let stop_process = result.is_err();
            (result, stop_process)
        }
        Err(_) => {
            let abort_succeeded = matches!(
                timeout(
                    ABORT_TIMEOUT,
                    client
                        .as_mut()
                        .expect("Pi process was initialized")
                        .abort_current(),
                )
                .await,
                Ok(Ok(()))
            );
            (
                Err(format!(
                    "{timeout_subject} did not finish within {} seconds",
                    prompt_timeout.as_secs()
                )),
                !abort_succeeded,
            )
        }
    };

    if stop_process && let Some(process) = client.take() {
        process.stop().await;
    }

    result
}

fn configured_prompt_timeout() -> Duration {
    std::env::var("CRUST_PROMPT_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_PROMPT_TIMEOUT)
}

pub struct PiProcess {
    _child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    next_request_id: u64,
}

impl PiProcess {
    async fn start() -> Result<Self, String> {
        let node = std::env::var_os("CRUST_NODE").unwrap_or_else(|| OsString::from("node"));
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let bridge = project_dir.join("sdk/bridge.mjs");

        let mut command = Command::new(&node);
        command
            .arg(&bridge)
            .current_dir(&project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|error| {
            format!(
                "could not start Node.js (`{}`): {error}. Install Node.js 22.19+ or set CRUST_NODE",
                node.to_string_lossy()
            )
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "could not open the Pi SDK bridge stdin".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "could not open the Pi SDK bridge stdout".to_owned())?;

        let mut process = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            next_request_id: 0,
        };
        process.wait_until_ready().await?;

        Ok(process)
    }

    async fn stop(mut self) {
        let _ = self._child.start_kill();
        let _ = self._child.wait().await;
    }

    async fn wait_until_ready(&mut self) -> Result<(), String> {
        loop {
            match self.next_event().await? {
                BridgeEvent::Ready => return Ok(()),
                BridgeEvent::Error { message, .. } | BridgeEvent::Fatal { message } => {
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    async fn run(
        &mut self,
        request: &UserRequest,
        on_delta: &mut impl FnMut(&str),
    ) -> Result<RequestOutcome, String> {
        self.next_request_id += 1;
        let id = self.next_request_id;
        let request = match request {
            UserRequest::Prompt(message) => BridgeRequest::Prompt {
                id,
                message: message.as_str(),
            },
            UserRequest::Bash {
                command,
                exclude_from_context,
            } => BridgeRequest::Bash {
                id,
                command: command.as_str(),
                exclude_from_context: *exclude_from_context,
            },
        };
        self.send_request(&request).await?;

        loop {
            match self.next_event().await? {
                BridgeEvent::TextDelta {
                    id: event_id,
                    delta,
                } if event_id == id => on_delta(&delta),
                BridgeEvent::Done { id: event_id } if event_id == id => {
                    return match request {
                        BridgeRequest::Prompt { .. } => Ok(RequestOutcome::Prompt),
                        _ => Err("the Pi SDK bridge returned the wrong completion type".to_owned()),
                    };
                }
                BridgeEvent::BashDone {
                    id: event_id,
                    output,
                    exit_code,
                    cancelled,
                    truncated,
                    full_output_path,
                } if event_id == id => {
                    return match request {
                        BridgeRequest::Bash { .. } => Ok(RequestOutcome::Bash(BashOutcome {
                            output,
                            exit_code,
                            cancelled,
                            truncated,
                            full_output_path,
                        })),
                        _ => Err("the Pi SDK bridge returned the wrong completion type".to_owned()),
                    };
                }
                BridgeEvent::Error {
                    id: Some(event_id),
                    message,
                } if event_id == id => return Err(message),
                BridgeEvent::Error { id: None, message } | BridgeEvent::Fatal { message } => {
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    async fn abort_current(&mut self) -> Result<(), String> {
        let id = self.next_request_id;
        self.send_request(&BridgeRequest::Abort { id }).await?;

        loop {
            match self.next_event().await? {
                BridgeEvent::Done { id: event_id }
                | BridgeEvent::BashDone { id: event_id, .. }
                | BridgeEvent::Error {
                    id: Some(event_id), ..
                } if event_id == id => return Ok(()),
                BridgeEvent::Error { id: None, message } | BridgeEvent::Fatal { message } => {
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    async fn send_request(&mut self, request: &BridgeRequest<'_>) -> Result<(), String> {
        let mut line = serde_json::to_vec(request)
            .map_err(|error| format!("could not encode request: {error}"))?;
        line.push(b'\n');

        self.stdin
            .write_all(&line)
            .await
            .map_err(|error| format!("could not write to the Pi SDK bridge: {error}"))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| format!("could not flush the Pi SDK bridge input: {error}"))
    }

    async fn next_event(&mut self) -> Result<BridgeEvent, String> {
        loop {
            let line = self
                .stdout
                .next_line()
                .await
                .map_err(|error| format!("could not read from the Pi SDK bridge: {error}"))?
                .ok_or_else(|| "the Pi SDK bridge exited unexpectedly".to_owned())?;

            if line.trim().is_empty() {
                continue;
            }

            return serde_json::from_str(&line)
                .map_err(|error| format!("invalid response from the Pi SDK bridge: {error}"));
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BridgeRequest<'a> {
    Prompt {
        id: u64,
        message: &'a str,
    },
    Bash {
        id: u64,
        command: &'a str,
        exclude_from_context: bool,
    },
    Abort {
        id: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BridgeEvent {
    Ready,
    TextDelta {
        id: u64,
        delta: String,
    },
    Done {
        id: u64,
    },
    BashDone {
        id: u64,
        output: String,
        exit_code: Option<i32>,
        cancelled: bool,
        truncated: bool,
        full_output_path: Option<String>,
    },
    Error {
        id: Option<u64>,
        message: String,
    },
    Fatal {
        message: String,
    },
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::{BridgeEvent, BridgeRequest, UserRequest};
    use serde_json::json;

    #[test]
    fn classifies_visible_bash_commands() {
        assert_eq!(
            UserRequest::from_input("  !  printf hello  ".to_owned()),
            UserRequest::Bash {
                command: "printf hello".to_owned(),
                exclude_from_context: false,
            }
        );
    }

    #[test]
    fn classifies_hidden_bash_commands() {
        assert_eq!(
            UserRequest::from_input("!!pwd".to_owned()),
            UserRequest::Bash {
                command: "pwd".to_owned(),
                exclude_from_context: true,
            }
        );
    }

    #[test]
    fn treats_empty_bash_prefixes_as_prompts() {
        assert_eq!(
            UserRequest::from_input("!".to_owned()),
            UserRequest::Prompt("!".to_owned())
        );
        assert_eq!(
            UserRequest::from_input("!!  ".to_owned()),
            UserRequest::Prompt("!!".to_owned())
        );
    }

    #[test]
    fn serializes_bash_context_visibility() {
        let visible = serde_json::to_value(BridgeRequest::Bash {
            id: 7,
            command: "pwd",
            exclude_from_context: false,
        })
        .expect("bash request should serialize");
        let hidden = serde_json::to_value(BridgeRequest::Bash {
            id: 8,
            command: "env",
            exclude_from_context: true,
        })
        .expect("hidden bash request should serialize");

        assert_eq!(
            visible,
            json!({
                "type": "bash",
                "id": 7,
                "command": "pwd",
                "exclude_from_context": false
            })
        );
        assert_eq!(hidden["exclude_from_context"], true);
    }

    #[test]
    fn deserializes_optional_bash_completion_fields() {
        let event: BridgeEvent = serde_json::from_value(json!({
            "type": "bash_done",
            "id": 9,
            "output": "done",
            "cancelled": false,
            "truncated": false
        }))
        .expect("bash completion should deserialize");

        match event {
            BridgeEvent::BashDone {
                id,
                output,
                exit_code,
                full_output_path,
                ..
            } => {
                assert_eq!(id, 9);
                assert_eq!(output, "done");
                assert_eq!(exit_code, None);
                assert_eq!(full_output_path, None);
            }
            event => panic!("expected bash completion, got {event:?}"),
        }
    }
}
