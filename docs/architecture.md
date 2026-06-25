# Cerebrum Architecture

## System Overview

Cerebrum is a two-tier agent memory subsystem implemented as a single Model Context Protocol (MCP) server. It provides agents with both short-term, volatile memory and long-term, persistent memory through a unified tool interface.

```mermaid
graph TD
    Agent[MCP Agent] -->|Tools: remember, recall, memorize, forget| Server[Cerebrum MCP Server]
    
    subgraph "Cerebrum Internal"
        Server --> Orchestrator[Orchestrator]
        Orchestrator --> Synapse[Synapse Tier: Short-term/Volatile]
        Orchestrator --> Cortex[Cortex Tier: Long-term/Persistent]
        
        Synapse --> RAM[In-Memory Storage]
    end

    subgraph "Storage"
        Cortex --> LanceDB[(LanceDB + Embeddings)]
    end
```

## Memory Tiers

### 1. Synapse (Short-term)
- **Nature:** Volatile, in-memory.
- **Scope:** Per-session/interaction context.
- **Lifecycle:** Cleared when the session ends or if manually purged.
- **Purpose:** Rapid retrieval of recent conversation context and immediate task details.

### 2. Cortex (Long-term)
- **Nature:** Persistent, disk-backed.
- **Scope:** Cross-session/global persistence.
- **Implementation:** LanceDB using vector embeddings for semantic search.
- **Lifecycle:** Durable; survives server restarts.
- **Purpose:** Long-term facts, user preferences, and historical context.

## Core Workflow: The Recall Process

When an agent calls `recall`, the Orchestrator performs a blended search across both tiers.

```mermaid
sequenceDiagram
    participant Agent
    participant Orchestrator
    participant Synapse
    participant Cortex

    Agent->>Orchestrator: recall(query)
    par Search Synapse
        Orchestrator->>Synapse: semantic_search(query)
        Synapse-->>Orchestrator: results (recent context)
    and Search Cortex
        Orchestrator->>Cortex: semantic_search(query)
        Cortex-->>Orchestrator: results (historical facts)
    end
    Orchestrator->>Orchestrator: merge & rank results
    Orchestrator-->>Agent: blended memory entries
```
