import assert from "node:assert/strict";
import test from "node:test";

import { highlightCode, highlightShellOutput } from "./highlight.mjs";

test("highlights known code and diff languages", () => {
  const rust = highlightCode('fn main() { println!("hello {}", 42); }', "rust");
  const diff = highlightCode("+1 added\n-1 removed", "diff");

  assert.match(rust, /hljs-keyword/);
  assert.match(rust, /hljs-string/);
  assert.match(rust, /hljs-number/);
  assert.match(diff, /hljs-addition/);
  assert.match(diff, /hljs-deletion/);
});

test("highlights git diff output as a diff", () => {
  const html = highlightShellOutput(
    "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n",
    "git --no-pager diff",
  );

  assert.match(html, /hljs-deletion/);
  assert.match(html, /hljs-addition/);
});

test("auto-detects other shell output", () => {
  const html = highlightShellOutput('{"ok": true, "count": 2}', "printf output");

  assert.match(html, /hljs-attr/);
  assert.match(html, /hljs-literal/);
  assert.match(html, /hljs-number/);
});

test("escapes source text before it reaches the HTML renderer", () => {
  const source = '<script>alert("x")</script><img src=x onerror=alert(1)> &';
  const codeHtml = highlightCode(source, "html");
  const shellHtml = highlightShellOutput(source, "printf output");

  for (const html of [codeHtml, shellHtml]) {
    assert.doesNotMatch(html, /<script|<img/i);
    assert.match(html, /&lt;/);
    assert.match(html, /&quot;/);
    assert.match(html, /&amp;/);
  }
});

test("skips empty output and unknown languages", () => {
  assert.equal(highlightShellOutput("", "git diff"), null);
  assert.equal(highlightCode("plain text", "not-a-language"), null);
});
