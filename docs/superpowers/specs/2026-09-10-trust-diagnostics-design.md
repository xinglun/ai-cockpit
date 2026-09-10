# Trust diagnostics and observation-bound Outcome design

## Intent

Resolve the five remaining strict-review findings without changing authorization,
exit-code, or historical-record semantics. The implementation keeps
Calibrated Human-Agent Trust as the North Star: a human report may be concise,
but every claim and suggested next action must be derived from the same runtime
facts that govern the machine result.

## Contract

1. Outcome assembly is the only I/O boundary. It captures and validates one
   bounded observation, derives the machine projection and the render input from
   that observation, and retries only a fixed number of times when the relevant
   records change. A failed retry is reported as unknown or failed; it is never
   represented as an atomic snapshot.
2. Rendering is pure. Summary, full report, CLI, and MCP consume the same typed
   render input and the same reason/action projection. Locales translate stable
   reason and action keys; they do not reimplement governance decisions.
3. Governance reasons are projected from `failedGate`, evidence state,
   acceptance/intent/range/authority dimensions, and existing protocol error
   classifications. A red decision alone is never evidence that verification
   failed.
4. Finalization retains classified recovery facts (receipt state, identity,
   observed resources, disposition, and valid plan). Actions are suggestions
   only; they cannot authorize or bypass Runtime operations. Missing, corrupt,
   mismatched, retained, already-deleted, and unknown states remain distinct.
5. Diagnostics use an independent, versioned channel. Phase spans are named,
   nested phases are not summed, counters are measured rather than estimated,
   benchmark-tool work is separated from Runtime work, and unsupported metrics
   carry a reason instead of zero. Existing performance conclusions are not
   rewritten.
6. Collaboration scenarios bind one scenario ID to facts, semantic invariants,
   and executable checks. Natural-language assertions check invariants in
   human output across CLI/MCP, summary/full views, and English/Simplified
   Chinese/Japanese. Machine JSON equality remains an additional protocol check,
   not a substitute for human semantic parity.

## Compatibility

New protocol fields are optional and serde-compatible with archived records.
Historical bytes remain immutable. Existing authorization gates, close rules,
exit codes, receipt validation, and resource-retention semantics remain the
source of truth. No global repository cache or cross-phase context is added.

## Delivery order

Implement the observation-bound assembly first; then add the shared reason and
finalization projections; then bind collaboration tests; then add diagnostics
and documentation. Each stage must preserve focused regression tests before
the next stage is integrated.
