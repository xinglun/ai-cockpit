<!-- AI_COCKPIT_ADAPTER_BEGIN provider=codex adapterVersion=1 repositoryId=sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b -->

This repository is attached to AI Cockpit.

Canonical interface: .ai/agent-interface.json

Use AI Cockpit as the repository-governance interface.
Prefer MCP when available; CLI remains the fallback.

Do not infer AI Cockpit state from this file.
Query the Runtime for current governance state.

<!-- AI_COCKPIT_ADAPTER_END -->

## AI Cockpit repository workflow

Read `.ai/README.md` before changing this repository. Use the installed shared
Runtime with an explicit `--repo /path/to/ai-cockpit` on every repository-bound
command. Query `inspect`, `status`, and `doctor` before acting; use the Work Item
lifecycle `start → preflight → checkpoint → verify → finish → archive → close`
for authorized changes. Do not infer state from this file, edit global Agent or
MCP configuration, or claim governance outcomes without current Runtime evidence.
