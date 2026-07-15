#!/usr/bin/env node

const protocolOutput = process.stdout;
const stderr = console.error.bind(console);

// stdout is reserved for the JSONL protocol consumed by the Rust process.
for (const method of ["log", "info", "debug", "warn"]) {
  console[method] = stderr;
}

function emit(message) {
  protocolOutput.write(`${JSON.stringify(message)}\n`);
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
    if (command.type !== "prompt" || typeof command.message !== "string" || id === null) {
      emit({ type: "error", id, message: "Expected { type: 'prompt', id, message }." });
      return;
    }

    activeRequestId = id;
    activeError = null;

    try {
      await session.prompt(command.message);
      if (activeError) {
        emit({ type: "error", id, message: activeError });
      } else {
        emit({ type: "done", id });
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
      if (line.length > 0) {
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
    pending.finally(dispose);
  });
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);

  emit({ type: "ready" });
}

main().catch((error) => {
  emit({ type: "fatal", message: errorMessage(error) });
  process.exitCode = 1;
});
