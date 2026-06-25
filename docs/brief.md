# Idea Brief: Cerebrum

## Idea
**`cerebrum`** — a Rust monorepo providing a two-tier memory subsystem for LLM AI agents, exposed as two MCP servers: **`synapse`** (short-term, session-scoped) and **`cortex`** (long-term, persisted in a LanceDB vector store).

## Context
LLM agents need memory that mirrors human cognition: fast, volatile working memory for the active session, and durable, searchable long-term memory that survives across sessions. The agent-memory space is crowded (cognee, supermemory, Memori, Letta) and heavily neuro-themed, so a distinctive yet faithful brain metaphor matters. The chosen design splits memory into two cooperating MCP servers under one anatomical umbrella.

## Explored directions
Naming was the focus of this session. Directions considered:
- **Whole-organ / umbrella names** — `cerebrum`, `encephalon`, `meninges`.
- **Mechanism / integrator names** — `claustrum` (binds brain regions), `saltatory` (fast node-to-node signal leaps), `myelin` (durable consolidation sheath).
- **Function-over-anatomy names** — `recall`, `mneme`, `engram`.
- **Non-brain medium metaphors** — `palimpsest`, `silt`, `amber`.

Availability findings (crates.io): `encephalon`, `claustrum`, `saltatory`, `meninges` were clean; `cerebro`, `psyche`, `myelin`, `ranvier` were taken; `cerebrum` and `cerebra` were crowded on GitHub but not hard-blocked.

## Chosen direction
**`cerebrum`** — won because the user had an immediate gut affinity for it, and once it was clear the repo name is an *umbrella brand over two MCP servers* (not a published crate), the crates.io/GitHub crowding stopped being a real blocker. The metaphor is airtight: the cerebrum is the part of the brain that *contains* the cortex, so `synapse` and `cortex` read naturally as organs/servers within it.

## Key characteristics
- **Two MCP servers, one repo:** `synapse` (short-term, in-session, volatile) and `cortex` (long-term, persistent).
- **Biologically coherent naming:** synapse = fast transient signaling; cortex = consolidated knowledge; cerebrum = the containing whole.
- **Vector-backed long-term memory:** LanceDB as the embedded store for `cortex`.
- **Rust implementation**, developed inside a **nix flake** dev shell.
- **MCP-native:** agents connect to each server over the Model Context Protocol rather than linking a library.
- **Consolidation pathway:** the interesting design challenge is how (intelligently) memories migrate from `synapse` → `cortex`.

## Open questions
- **Consolidation policy:** What triggers promotion from synapse to cortex — explicit tool call, recency/frequency heuristics, summarization, end-of-session flush?
- **Embedding strategy:** Which embedding model feeds LanceDB; is it local (e.g. via the nix shell) or remote? Where does embedding happen — in `cortex` or upstream?
- **Synapse storage:** Pure in-empty per session, or lightweight on-disk (e.g. sled/redb) for crash recovery?
- **Session/identity model:** How are sessions, agents, and users scoped and isolated across both servers?
- **Schema & retrieval:** What metadata travels with each memory (timestamps, source, salience), and what does the MCP tool surface (search, upsert, forget, summarize)?
- **Crate/binary layout:** Single workspace with `synapse`/`cortex` crates + shared `cerebrum-core`? Binary names?
- **Forgetting/decay:** Does cortex prune or down-weight stale memories over time?

## Prior art
- Agent-memory landscape: https://github.com/topics/agent-memory
- cognee (knowledge-graph long-term memory): https://github.com/topicseretes/cognee → https://github.com/topoteretes/cognee
- supermemory (fast local memory/context engine): https://github.com/supermemoryai/supermemory
- Memori (LLM-agnostic memory layer): https://github.com/MemoriLabs/Memori
- honcho (memory library for stateful agents): https://github.com/plastic-labs/honcho
- Letta (stateful agents with memory): https://github.com/letta-ai
- LanceDB (embedded vector store): https://github.com/lancedb/lancedb
- Model Context Protocol: https://modelcontextprotocol.io
- Neuroscience of consolidation (myelin & memory): https://www.nature.com/articles/s41593-020-0648-0

## Recommended next steps
- **`spar`** should challenge first: the **synapse → cortex consolidation policy** (is the two-server split worth the coordination cost vs. one server with two tiers?), and whether MCP-per-tier is the right boundary.
- **`plan`** should focus on: the **Rust workspace layout** (`cerebrum-core` + `synapse` + `cortex`), the **MCP tool surface** for each server, the **LanceDB schema + embedding pipeline**, and the **nix flake** dev environment.
