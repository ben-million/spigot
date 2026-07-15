use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::timeout,
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_PROMPT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

pub type SharedPiClient = Arc<Mutex<Option<PiProcess>>>;

pub fn new_client() -> SharedPiClient {
    Arc::new(Mutex::new(None))
}

pub async fn prompt(
    client: &SharedPiClient,
    message: String,
    mut on_delta: impl FnMut(&str),
) -> Result<(), String> {
    let mut client = client.lock().await;

    if client.is_none() {
        let process = timeout(STARTUP_TIMEOUT, PiProcess::start())
            .await
            .map_err(|_| "the Pi SDK bridge did not start within 30 seconds".to_owned())??;
        *client = Some(process);
    }

    let prompt_timeout = configured_prompt_timeout();
    let result = match timeout(
        prompt_timeout,
        client
            .as_mut()
            .expect("Pi process was initialized")
            .prompt(&message, &mut on_delta),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Pi did not finish within {} seconds",
            prompt_timeout.as_secs()
        )),
    };

    if result.is_err()
        && let Some(process) = client.take()
    {
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

    async fn prompt(
        &mut self,
        message: &str,
        on_delta: &mut impl FnMut(&str),
    ) -> Result<(), String> {
        self.next_request_id += 1;
        let id = self.next_request_id;
        let request = PromptRequest {
            kind: "prompt",
            id,
            message,
        };
        let mut line = serde_json::to_vec(&request)
            .map_err(|error| format!("could not encode prompt: {error}"))?;
        line.push(b'\n');

        self.stdin
            .write_all(&line)
            .await
            .map_err(|error| format!("could not write to the Pi SDK bridge: {error}"))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| format!("could not flush the Pi SDK bridge input: {error}"))?;

        loop {
            match self.next_event().await? {
                BridgeEvent::TextDelta {
                    id: event_id,
                    delta,
                } if event_id == id => on_delta(&delta),
                BridgeEvent::Done { id: event_id } if event_id == id => return Ok(()),
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
struct PromptRequest<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    id: u64,
    message: &'a str,
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
