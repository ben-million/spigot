# Agent Guidance

## Philosophy

When working in this repository, embody simplicity, clarity, and frugality. Produce software that is minimal, usable, reliable, and easy to understand. Treat unnecessary complexity—not a lack of features—as the default risk.

This project is for advanced and experienced computer users. Respect their ability to understand their tools and shape their own workflows. Prefer transparent behavior, direct controls, composability, and clear interfaces over hand-holding, hidden policy, or lowest-common-denominator design. Do not add convenience features that compromise coherence or make the system harder to reason about.

Simple and elegant software requires more discipline than accumulating ad-hoc or over-ambitious features. Pay that design cost up front. Keep goals reasonable and attainable, preserve conceptual integrity, and favor work that brings the project to a complete, maintainable state.

## Working Manifest

Do not mistake the amount of code written for progress. Lines of code are liabilities as well as tools. Judge a change by the problem it solves, the clarity of its design, and the code that can be avoided or removed—not by its size.

Do not stop at “it works” when the result has poor structure. Maintain conceptual clarity throughout the software lifecycle. Resist patches that obscure the design, duplicate responsibility, or bind resources indefinitely. Complexity leads to inconsistency, poor usability, weak performance, defects, and vulnerabilities.

Ingenious ideas and ingenious software are simple. Follow the Unix spirit: make each part clear, focused, and composable. Prefer deleting code, collapsing layers, and removing special cases when doing so preserves required behavior. Smaller, clearer software is progress.

When an existing design is fundamentally compromised, do not perpetuate it with another workaround. Recommend a focused simplification or replacement and explain why. Do not undertake an unrelated or wholesale rewrite without the user's approval.

## Instructions for Every Change

- Understand the existing design before editing it, then choose the smallest complete solution.
- Keep changes narrowly scoped. Do not introduce speculative features, abstractions, dependencies, configuration, or extension points.
- Prefer plain, direct code over clever machinery. Make control flow, state, ownership, and failure modes obvious.
- Preserve a coherent system design; do not trade long-term clarity for short-term convenience.
- Remove dead code, duplication, needless indirection, and obsolete compatibility paths when they are within scope and safe to remove.
- Keep interfaces minimal and consistent. Expose only what users need, and avoid hidden behavior.
- Treat performance, resource use, reliability, maintainability, and security as consequences of simplicity—not as excuses for extra architecture.
- Add tests and documentation only where they clarify and protect intended behavior; keep both concise and purposeful.
- Before finishing, ask whether the same result can be achieved with fewer concepts, fewer branches, fewer dependencies, or less code.
