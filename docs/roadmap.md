# Cerebrum Roadmap

This roadmap covers work **deferred beyond the initial build**. The initial build delivers a single `cerebrum` MCP server with two memory tiers — `synapse` (volatile, in-memory, per-session) and `cortex` (durable, LanceDB-backed, cross-session) — exposing four tools: `remember`, `recall` (blended across tiers), `memorize` (human-triggered promotion), and `forget`.

Everything below is the **intelligence layer** — the genuinely novel research surface. None of it is in the initial build. Each item should go through a `spar` session before it is planned in and added to the implementation plan.

> These are future directions, not a fixed sequence. More phases may be added here before any of this work begins.

---

## Intelligence layer

### Automatic promotion (synapse $\to$ cortex)
The initial build only promotes on **explicit human instruction** (the `memorize` tool). Automatic promotion lets the system itself decide a short-term memory has earned durability.

- **Triggers to evaluate:** recency + frequency of access, explicit salience scoring, end-of-session flush, summarisation of a continuous session into a few durable facts.
- **Risk:** re-introduces the salience-judgment problem deliberately avoided in the initial design. Any automatic policy must be auditable via provenance and reversible via `forget`.
- **Design contract:** a `PromotionPolicy` trait deciding which synapse memories graduate, kept behind the same orchestrator so the tool surface does not change.

### Decay & forgetting in cortex
Long-term memory should not grow without bound or let stale facts outrank current ones.

- Time-based down-weighting of `salience` at recall time.
- Pruning or archival of low-salience, never-recalled memories.
- Conflict resolution when a newer memory contradicts an older one (e.g. "user moved cities").

### Summarisation on promote
Rather than promoting verbatim conversation snippets, distil them into compact, self-contained facts before writing to cortex — reducing drift and storage while preserving provenance back to the source session.

### Identity & scope model
The initial build scopes `synapse` per-session and `cortex` globally. A richer model is needed for multi-agent / multi-user deployments.

- Per-agent vs. per-user vs. global cortex scoping.
- Isolation guarantees between identities.
- Shared/organisational memory (cf. Mem0's org layer) available to multiple agents.

### Real embedding strategy hardening
The initial build commits to a local `fastembed` model (BGE-small, 384-dim). Future work:

- Pluggable embedding backends (local model vs. remote API) behind the existing `Embedder` trait.
- Migration tooling for when the embedding model — and therefore the cortex vector dimension — changes, since this requires rebuilding the LanceDB table.
- Vendoring model weights in the nix flake for fully offline, reproducible builds.

---

## Guiding principles (apply to all intelligence layer work)
1. **The agent must not see the seams.** Tiering stays an implementation detail; the tool surface (`remember`/`recall`/`memorize`/`forget`) should remain stable.
2. **Every durable memory carries provenance.** Automatic decisions must be auditable and reversible.
3. **Spar before planning.** Each item here is a design question, not a mechanical task — challenge it before committing to an approach.
