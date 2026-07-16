import hljs from "highlight.js/lib/index.js";

const SHELL_SAMPLE_CHARS = 2_000;
const SHELL_LANGUAGES = [
  "bash",
  "c",
  "cpp",
  "css",
  "diff",
  "go",
  "ini",
  "java",
  "javascript",
  "json",
  "makefile",
  "markdown",
  "objectivec",
  "perl",
  "php",
  "python",
  "ruby",
  "rust",
  "sql",
  "typescript",
  "xml",
  "yaml",
];

export function countDiffLines(diff) {
  let added = 0;
  let removed = 0;
  for (const line of (diff || "").split(/\r?\n/)) {
    if (line.startsWith("+") && !line.startsWith("+++")) {
      added += 1;
    } else if (line.startsWith("-") && !line.startsWith("---")) {
      removed += 1;
    }
  }
  return { added, removed };
}

export function highlightCode(code, language) {
  if (!code || !language || !hljs.getLanguage(language)) {
    return null;
  }

  try {
    return hljs.highlight(code, { language, ignoreIllegals: true }).value;
  } catch {
    return null;
  }
}

function isDiff(command, output) {
  return (
    /(?:^|[;&|])\s*(?:\S*\/)?(?:git\b[^\n;&|]*\bdiff(?:-(?:files|index|tree))?|diff)\b/.test(
      command,
    ) ||
    (/^--- /m.test(output) && /^\+\+\+ /m.test(output))
  );
}

export function highlightShellOutput(output, command = "") {
  if (!output) {
    return null;
  }
  if (isDiff(command, output)) {
    return highlightCode(output, "diff");
  }

  try {
    const sample = output.slice(0, SHELL_SAMPLE_CHARS);
    const detected = hljs.highlightAuto(sample, SHELL_LANGUAGES);
    return sample.length === output.length
      ? detected.value
      : highlightCode(output, detected.language ?? "plaintext");
  } catch {
    return null;
  }
}
