# Chasm CLI Architecture

This document provides an overview of the Chasm CLI architecture.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CHASM CLI                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   CLI       │  │  API Server │  │  MCP Server │  │    TUI      │       │
│  │  (clap)     │  │ (actix-web) │  │   (stdio)   │  │ (ratatui)   │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│         │                │                │                │              │
│         └────────────────┴────────────────┴────────────────┘              │
│                                   │                                        │
│                          ┌────────┴────────┐                              │
│                          │   Core Library  │                              │
│                          └────────┬────────┘                              │
│                                   │                                        │
│         ┌─────────────────────────┼─────────────────────────┐             │
│         │                         │                         │             │
│  ┌──────┴──────┐  ┌──────────────┴──────────────┐  ┌───────┴───────┐     │
│  │  Database   │  │        Providers            │  │    Agency     │     │
│  │  (rusqlite) │  │  (cloud + local harvesters) │  │ (AI agents)   │     │
│  └─────────────┘  └─────────────────────────────┘  └───────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── cli.rs               # CLI command definitions (clap)
├── error.rs             # Error types
├── models.rs            # Core data models
├── database.rs          # SQLite database operations
├── storage.rs           # File system operations
├── workspace.rs         # Workspace discovery & management
├── browser.rs           # Browser cookie extraction
│
├── api/                 # REST API server
│   ├── mod.rs           # Server configuration
│   ├── state.rs         # Application state
│   ├── handlers_simple.rs  # API handlers
│   ├── handlers_swe.rs  # SWE mode handlers
│   ├── auth.rs          # Authentication
│   └── sync.rs          # Real-time sync
│
├── mcp/                 # Model Context Protocol
│   ├── mod.rs           # MCP server
│   ├── tools.rs         # MCP tool definitions
│   └── resources.rs     # MCP resources
│
├── tui/                 # Terminal UI
│   ├── mod.rs           # TUI application
│   └── ...              # UI components
│
├── commands/            # CLI command implementations
│   ├── mod.rs           
│   ├── harvest.rs       # Session harvesting
│   ├── show.rs          # Display commands
│   ├── export.rs        # Export commands
│   └── ...              
│
├── providers/           # Chat provider integrations
│   ├── mod.rs           
│   ├── session_format.rs  # Universal format
│   ├── cloud/           # Cloud providers
│   │   ├── anthropic.rs
│   │   ├── chatgpt.rs
│   │   ├── gemini.rs
│   │   ├── perplexity.rs
│   │   └── ...
│   └── local/           # Local providers
│       ├── copilot.rs   # VS Code Copilot
│       ├── cursor.rs    # Cursor AI
│       ├── ollama.rs
│       └── ...
│
├── agency/              # AI Agent framework
│   ├── mod.rs           
│   ├── agent.rs         # Agent definition
│   ├── executor.rs      # Agent execution
│   ├── orchestrator.rs  # Multi-agent orchestration
│   ├── tools.rs         # Tool system
│   ├── session.rs       # Agent sessions
│   ├── memory.rs        # RAG & memory
│   └── ...
│
└── integrations/        # Third-party integrations
    ├── mod.rs
    ├── registry.rs
    └── hooks.rs
```

## Data Flow

### Session Harvesting

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   VS Code    │     │   Provider   │     │   Database   │
│  Workspace   │────▶│  Harvester   │────▶│   (SQLite)   │
│   Storage    │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
        │                    │                    │
        │                    ▼                    │
        │            ┌──────────────┐            │
        │            │  Universal   │            │
        │            │   Format     │            │
        │            └──────────────┘            │
        │                    │                    │
        ▼                    ▼                    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Cursor     │     │    Cloud     │     │   Exports    │
│   Storage    │────▶│   Providers  │────▶│  (MD/JSON)   │
└──────────────┘     └──────────────┘     └──────────────┘
```

### API Request Flow

```
┌────────┐     ┌────────────┐     ┌──────────────┐     ┌──────────┐
│ Client │────▶│  CORS/Auth │────▶│   Handler    │────▶│ Database │
│        │     │ Middleware │     │              │     │          │
└────────┘     └────────────┘     └──────────────┘     └──────────┘
    ▲                                    │
    │                                    ▼
    │                            ┌──────────────┐
    └────────────────────────────│   Response   │
                                 │   (JSON)     │
                                 └──────────────┘
```

### MCP Tool Execution

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  AI Client   │────▶│  MCP Server  │────▶│    Tool      │
│  (Claude,    │     │   (stdio)    │     │  Registry    │
│   VS Code)   │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │
       │                    ▼                    ▼
       │            ┌──────────────┐     ┌──────────────┐
       │            │   Schema     │     │   Database   │
       │            │  Validation  │     │   Query      │
       │            └──────────────┘     └──────────────┘
       │                    │                    │
       ▼                    ▼                    ▼
┌──────────────────────────────────────────────────────┐
│                   Tool Result                        │
│            (text, JSON, or error)                    │
└──────────────────────────────────────────────────────┘
```

## Database Schema

```sql
-- Core tables
workspaces (id, name, path, provider, created_at, updated_at)
sessions (id, workspace_id, title, provider, model, created_at, updated_at)
messages (id, session_id, role, content, timestamp, tokens)
checkpoints (id, session_id, name, message_id, created_at)
share_links (id, session_id, provider, url, status, created_at)

-- Agency tables
agents (id, name, description, instruction, role, model, provider, ...)
swarms (id, name, orchestration, agents, status, ...)

-- Settings tables
settings (key, value, updated_at)
provider_accounts (id, provider, name, credentials, ...)

-- SWE tables
swe_projects (id, name, path, description, ...)
swe_memory (id, project_id, type, content, embedding, ...)
swe_rules (id, project_id, type, pattern, action, ...)
```

## Provider Architecture

### Cloud Providers

```rust
pub trait CloudProvider {
    fn name(&self) -> &str;
    fn authenticate(&mut self, credentials: &Credentials) -> Result<()>;
    fn list_conversations(&self) -> Result<Vec<Conversation>>;
    fn get_conversation(&self, id: &str) -> Result<Conversation>;
    fn export(&self, id: &str, format: ExportFormat) -> Result<String>;
}
```

Implementations:
- `AnthropicProvider` - Claude conversations via API
- `ChatGPTProvider` - ChatGPT via browser cookies
- `GeminiProvider` - Google Gemini via API
- `PerplexityProvider` - Perplexity via API
- `DeepSeekProvider` - DeepSeek via API
- `M365CopilotProvider` - Microsoft 365 Copilot via Graph API

### Local Providers

```rust
pub trait LocalProvider {
    fn name(&self) -> &str;
    fn discover_storage(&self) -> Result<Vec<PathBuf>>;
    fn parse_sessions(&self, path: &Path) -> Result<Vec<Session>>;
}
```

Implementations:
- `CopilotProvider` - VS Code GitHub Copilot Chat
- `CursorProvider` - Cursor AI editor
- `WindsurfProvider` - Windsurf editor
- `OllamaProvider` - Ollama local LLM
- `LMStudioProvider` - LM Studio
- `Jan` - Jan AI

## Agency Framework

The Agency module provides a Rust-native AI agent framework:

```
┌─────────────────────────────────────────────────────────────┐
│                      Agency Runtime                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐  │
│  │   Agent     │     │  Executor   │     │ Orchestrator│  │
│  │  Builder    │────▶│             │────▶│             │  │
│  └─────────────┘     └─────────────┘     └─────────────┘  │
│                             │                    │         │
│                             ▼                    ▼         │
│                      ┌─────────────┐     ┌─────────────┐  │
│                      │   Tools     │     │   Swarms    │  │
│                      │  Registry   │     │ (Pipelines) │  │
│                      └─────────────┘     └─────────────┘  │
│                             │                    │         │
│                             ▼                    ▼         │
│                      ┌─────────────┐     ┌─────────────┐  │
│                      │   Session   │     │   Memory    │  │
│                      │  Manager    │     │   (RAG)     │  │
│                      └─────────────┘     └─────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Orchestration Patterns

- **Sequential**: Agents execute one after another
- **Parallel**: Agents execute simultaneously
- **Hierarchical**: Coordinator delegates to worker agents
- **Loop**: Agent repeats until condition is met

## Security Considerations

1. **Cookie Extraction**: Browser cookies are extracted locally for cloud provider access
2. **Credentials Storage**: API keys stored encrypted in system keyring
3. **Database**: Local SQLite database, no network exposure by default
4. **API Server**: CORS configured for localhost only by default
5. **JWT Auth**: Optional authentication for multi-user deployments

## Performance

- **Async I/O**: Tokio-based async runtime
- **Connection Pooling**: Database connections are pooled
- **Lazy Loading**: Sessions loaded on-demand
- **Caching**: In-memory caching for frequent queries
- **Batch Operations**: Bulk inserts for harvesting

## Extension Points

1. **Custom Providers**: Implement `CloudProvider` or `LocalProvider` traits
2. **Custom Tools**: Register tools with the `ToolRegistry`
3. **Custom Agents**: Use `AgentBuilder` to create specialized agents
4. **Webhooks**: Integration hooks for external services
