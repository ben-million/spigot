#!/usr/bin/env node

import { highlightCode, highlightShellOutput } from "./highlight.mjs";

const protocolOutput = process.stdout;
const stderr = console.error.bind(console);
const MAX_STREAMED_BASH_CHARS = 100_000;
const BASH_FLUSH_INTERVAL_MS = 16;

// Before SDK initialization completes there is nothing to dispose. If the
// Rust parent disappears during startup, exit immediately rather than orphaning
// the bridge. This handler is replaced with graceful cleanup once ready.
let parentCloseHandler = () => process.exit(0);
let inputHandler = null;
let pendingInput = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  if (inputHandler) {
    inputHandler(chunk);
  } else {
    pendingInput += chunk;
  }
});
process.stdin.once("end", () => parentCloseHandler());

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
    emit({ type: "bash_delta", id, delta });
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

function textOutput(result) {
  return (
    result?.content
      ?.filter((part) => part.type === "text" && typeof part.text === "string")
      .map((part) => part.text)
      .join("\n") || ""
  );
}

function conciseError(result) {
  const lines = textOutput(result)
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0);
  return lines.at(-1)?.trim().slice(0, 500) || "Tool failed";
}

function bashOutput(result, error) {
  const output = textOutput(result);
  if (!error) {
    return output;
  }
  if (output === error) {
    return "";
  }

  const errorSuffix = `\n\n${error}`;
  return output.endsWith(errorSuffix) ? output.slice(0, -errorSuffix.length) : output;
}

function highlightedToolOutput(toolName, args, result, output, getLanguageFromPath) {
  if (toolName === "bash") {
    return highlightShellOutput(output, args?.command);
  }
  if (toolName === "edit") {
    return highlightCode(result?.details?.diff, "diff");
  }
  if (toolName !== "read") {
    return null;
  }

  const path = args?.path ?? args?.file_path;
  if (typeof path !== "string") {
    return null;
  }
  const fileName = path.split(/[\\/]/).at(-1);
  const language = getLanguageFromPath(path) ?? getLanguageFromPath(fileName);
  return highlightCode(textOutput(result), language);
}

function toolSummaryArgs(toolName, args) {
  switch (toolName) {
    case "read":
      return {
        path: args?.path ?? args?.file_path,
        offset: args?.offset,
        limit: args?.limit,
      };
    case "bash":
      return { command: args?.command };
    case "edit":
    case "write":
      return { path: args?.path ?? args?.file_path };
    case "grep":
    case "find":
      return { pattern: args?.pattern, path: args?.path };
    case "ls":
      return { path: args?.path };
    default:
      return {};
  }
}

async function main() {
  const {
    AuthStorage,
    createAgentSession,
    DefaultResourceLoader,
    getAgentDir,
    getLanguageFromPath,
    ModelRegistry,
    SessionManager,
  } = await import("@earendil-works/pi-coding-agent");

  const cwd = process.env.SPIGOT_AGENT_CWD || process.cwd();
  const authStorage = AuthStorage.create();
  const modelRegistry = ModelRegistry.create(authStorage);
  const resourceLoader = new DefaultResourceLoader({
    cwd,
    agentDir: getAgentDir(),
    appendSystemPromptOverride: (base) => [
      ...base,
      "Spigot displays all model output as plain text, including visible thinking and final responses. Do not use Markdown anywhere. Use ordinary sentences and line breaks instead of headings, list markers, emphasis, links, tables, inline code, or fenced code blocks.",
    ],
  });
  await resourceLoader.reload();

  const { session, modelFallbackMessage } = await createAgentSession({
    cwd,
    authStorage,
    modelRegistry,
    resourceLoader,
    sessionManager: SessionManager.inMemory(cwd),
  });

  if (modelFallbackMessage) {
    stderr(modelFallbackMessage);
  }

  let activeRequestId = null;
  let activeAssistantMessageId = 0;
  let activeError = null;
  const activeToolArgs = new Map();

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
    if (activeRequestId === null) {
      return;
    }

    if (event.type === "message_start" && event.message.role === "assistant") {
      activeAssistantMessageId += 1;
      emit({ type: "assistant_start", id: activeRequestId });
    } else if (event.type === "message_update") {
      const update = event.assistantMessageEvent;
      if (update.type === "thinking_start") {
        const thinkingId = `${activeRequestId}:${activeAssistantMessageId}:${update.contentIndex}`;
        emit({ type: "thinking_start", id: activeRequestId, thinking_id: thinkingId });
      } else if (update.type === "thinking_delta") {
        const thinkingId = `${activeRequestId}:${activeAssistantMessageId}:${update.contentIndex}`;
        emit({
          type: "thinking_delta",
          id: activeRequestId,
          thinking_id: thinkingId,
          delta: update.delta,
        });
      } else if (update.type === "thinking_end") {
        const thinkingId = `${activeRequestId}:${activeAssistantMessageId}:${update.contentIndex}`;
        emit({
          type: "thinking_end",
          id: activeRequestId,
          thinking_id: thinkingId,
          content: update.content,
        });
      } else if (update.type === "text_delta") {
        emit({ type: "text_delta", id: activeRequestId, delta: update.delta });
      }
    } else if (event.type === "message_end" && event.message.role === "assistant") {
      if (event.message.stopReason === "error" || event.message.stopReason === "aborted") {
        activeError =
          event.message.errorMessage || `Pi stopped with reason: ${event.message.stopReason}`;
      } else {
        activeError = null;
      }
    } else if (event.type === "tool_execution_start") {
      if (event.toolName === "read" || event.toolName === "bash") {
        activeToolArgs.set(event.toolCallId, event.args);
      }
      emit({
        type: "tool_start",
        id: activeRequestId,
        tool_call_id: event.toolCallId,
        tool_name: event.toolName,
        args: toolSummaryArgs(event.toolName, event.args),
      });
    } else if (event.type === "tool_execution_end") {
      const error = event.isError ? conciseError(event.result) : null;
      const args = activeToolArgs.get(event.toolCallId);
      const output = event.toolName === "bash" ? bashOutput(event.result, error) : null;
      const highlightedHtml =
        event.isError && event.toolName !== "bash"
          ? null
          : highlightedToolOutput(
              event.toolName,
              args,
              event.result,
              output,
              getLanguageFromPath,
            );
      activeToolArgs.delete(event.toolCallId);
      emit({
        type: "tool_end",
        id: activeRequestId,
        tool_call_id: event.toolCallId,
        is_error: event.isError,
        error,
        output,
        highlighted_html: highlightedHtml,
      });
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
    activeAssistantMessageId = 0;
    activeError = null;
    activeToolArgs.clear();

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
          highlighted_html: highlightShellOutput(result.output, command.command),
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
      activeAssistantMessageId = 0;
      activeError = null;
      activeToolArgs.clear();
    }
  }

  let pending = Promise.resolve();
  let buffer = "";
  inputHandler = (chunk) => {
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
  };
  if (pendingInput.length > 0) {
    const input = pendingInput;
    pendingInput = "";
    inputHandler(input);
  }

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

  parentCloseHandler = () => {
    // Abort active work immediately so a detached shell process cannot outlive
    // Spigot, then let the request handler and protocol output unwind.
    dispose();
    pending.finally(async () => {
      await protocolWrites;
    });
  };
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);

  emit({ type: "ready" });
}

main().catch((error) => {
  emit({ type: "fatal", message: errorMessage(error) });
  process.exitCode = 1;
});
