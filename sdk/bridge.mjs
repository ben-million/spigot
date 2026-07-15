#!/usr/bin/env node

const protocolOutput = process.stdout;
const stderr = console.error.bind(console);
const MAX_STREAMED_BASH_CHARS = 100_000;
const BASH_FLUSH_INTERVAL_MS = 16;

// stdout is reserved for the JSONL protocol consumed by the Rust process.
for (const method of ["log", "info", "debug", "warn"]) {
  console[method] = stderr;
}

let protocolWrites = Promise.resolve();
function emit(message) {
  const line = `${JSON.stringify(message)}\n`;
  protocolWrites = protocolWrites.then(
    () =>
      new Promise((resolve) => {
        if (protocolOutput.write(line)) {
          resolve();
        } else {
          protocolOutput.once("drain", resolve);
        }
      }),
  );
  return protocolWrites;
}

function createBashStreamer(id) {
  let buffered = "";
  let streamedCharacters = 0;
  let flushTimer = null;

  const flush = () => {
    flushTimer = null;
    if (buffered.length === 0) {
      return;
    }
    const delta = buffered;
    buffered = "";
    emit({ type: "text_delta", id, delta });
  };

  return {
    push(delta) {
      const remaining = MAX_STREAMED_BASH_CHARS - streamedCharacters;
      if (remaining <= 0) {
        return;
      }

      const limitedDelta = delta.slice(0, remaining);
      streamedCharacters += limitedDelta.length;
      buffered += limitedDelta;
      if (flushTimer === null) {
        flushTimer = setTimeout(flush, BASH_FLUSH_INTERVAL_MS);
      }
    },
    finish() {
      if (flushTimer !== null) {
        clearTimeout(flushTimer);
      }
      flush();
    },
  };
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

async function main() {
  const {
    AuthStorage,
    createAgentSession,
    ModelRegistry,
    SessionManager,
  } = await import("@earendil-works/pi-coding-agent");

  const cwd = process.env.CRUST_AGENT_CWD || process.cwd();
  const authStorage = AuthStorage.create();
  const modelRegistry = ModelRegistry.create(authStorage);
  const { session, modelFallbackMessage } = await createAgentSession({
    cwd,
    authStorage,
    modelRegistry,
    sessionManager: SessionManager.inMemory(cwd),
  });

  if (modelFallbackMessage) {
    stderr(modelFallbackMessage);
  }

  let activeRequestId = null;
  let activeError = null;

  function handleControlLine(line) {
    let command;
    try {
      command = JSON.parse(line);
    } catch {
      return false;
    }

    if (command.type !== "abort") {
      return false;
    }

    if (Number.isSafeInteger(command.id) && command.id === activeRequestId) {
      if (session.isBashRunning) {
        session.abortBash();
      } else {
        void session.abort().catch((error) => stderr("Could not abort Pi:", error));
      }
    }
    return true;
  }

  const unsubscribe = session.subscribe((event) => {
    if (activeRequestId === null || event.type !== "message_update") {
      return;
    }

    const update = event.assistantMessageEvent;
    if (update.type === "text_delta") {
      emit({ type: "text_delta", id: activeRequestId, delta: update.delta });
    } else if (update.type === "error") {
      activeError =
        update.error?.errorMessage || `Pi stopped with reason: ${update.reason}`;
    }
  });

  async function handleLine(line) {
    let command;
    try {
      command = JSON.parse(line);
    } catch (error) {
      emit({ type: "error", id: null, message: `Invalid JSON: ${errorMessage(error)}` });
      return;
    }

    const id = Number.isSafeInteger(command.id) ? command.id : null;
    const isPrompt = command.type === "prompt" && typeof command.message === "string";
    const isBash =
      command.type === "bash" &&
      typeof command.command === "string" &&
      command.command.trim().length > 0 &&
      typeof command.exclude_from_context === "boolean";

    if (id === null || (!isPrompt && !isBash)) {
      emit({
        type: "error",
        id,
        message:
          "Expected a prompt request or { type: 'bash', id, command, exclude_from_context }.",
      });
      return;
    }

    activeRequestId = id;
    activeError = null;

    try {
      if (isBash) {
        const streamer = createBashStreamer(id);
        let result;
        try {
          result = await session.executeBash(command.command, streamer.push, {
            excludeFromContext: command.exclude_from_context,
          });
        } finally {
          streamer.finish();
        }
        await emit({
          type: "bash_done",
          id,
          output: result.output,
          exit_code: result.exitCode,
          cancelled: result.cancelled,
          truncated: result.truncated,
          full_output_path: result.fullOutputPath,
        });
      } else {
        await session.prompt(command.message);
        if (activeError) {
          emit({ type: "error", id, message: activeError });
        } else {
          emit({ type: "done", id });
        }
      }
    } catch (error) {
      emit({ type: "error", id, message: errorMessage(error) });
    } finally {
      activeRequestId = null;
      activeError = null;
    }
  }

  let pending = Promise.resolve();
  let buffer = "";
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", (chunk) => {
    buffer += chunk;

    while (true) {
      const newline = buffer.indexOf("\n");
      if (newline === -1) {
        break;
      }

      let line = buffer.slice(0, newline);
      buffer = buffer.slice(newline + 1);
      if (line.endsWith("\r")) {
        line = line.slice(0, -1);
      }
      if (line.length > 0 && !handleControlLine(line)) {
        pending = pending.then(() => handleLine(line));
      }
    }
  });

  let disposed = false;
  const dispose = () => {
    if (disposed) {
      return;
    }
    disposed = true;
    unsubscribe();
    session.dispose();
  };
  const shutdown = () => {
    dispose();
    process.exit(0);
  };

  process.stdin.on("end", () => {
    pending.finally(async () => {
      await protocolWrites;
      dispose();
    });
  });
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);

  emit({ type: "ready" });
}

main().catch((error) => {
  emit({ type: "fatal", message: errorMessage(error) });
  process.exitCode = 1;
});
