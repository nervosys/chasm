-- CSM Universal Intermediate Database Schema
-- Version: 3.0
-- 
-- A provider-agnostic, portable AI memory database supporting:
-- - Multi-provider chat sessions (cloud & local LLMs)
-- - Agentic AI workflows (tool calls, multi-agent orchestration)
-- - Vector embeddings for semantic search (RAG)
-- - Conversation branching and version control
-- - Cross-platform portability

-- =============================================================================
-- CORE INFRASTRUCTURE
-- =============================================================================

-- Schema version and configuration tracking
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Insert/update schema version
INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', '3.0');
INSERT OR IGNORE INTO metadata (key, value) VALUES ('created_at', strftime('%s', 'now'));

-- =============================================================================
-- PROVIDER REGISTRY
-- =============================================================================

-- Registered AI providers (both cloud and local)
CREATE TABLE IF NOT EXISTS providers (
    id TEXT PRIMARY KEY,                    -- e.g., 'openai', 'anthropic', 'ollama'
    name TEXT NOT NULL,                     -- Display name
    type TEXT NOT NULL DEFAULT 'cloud',     -- 'cloud', 'local', 'hybrid'
    endpoint TEXT,                          -- API endpoint URL
    auth_method TEXT,                       -- 'api_key', 'oauth', 'none'
    capabilities TEXT,                      -- JSON: ['chat', 'embeddings', 'tools', 'vision']
    default_model TEXT,                     -- Default model for this provider
    is_active INTEGER DEFAULT 1,            -- Whether provider is enabled
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT                           -- JSON blob for provider-specific config
);

-- Available models per provider
CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,                    -- e.g., 'gpt-4o', 'claude-3.5-sonnet'
    provider_id TEXT NOT NULL,              -- FK to providers.id
    name TEXT NOT NULL,                     -- Display name
    type TEXT DEFAULT 'chat',               -- 'chat', 'embedding', 'vision', 'code'
    context_window INTEGER,                 -- Max context tokens
    max_output_tokens INTEGER,              -- Max output tokens
    supports_tools INTEGER DEFAULT 0,       -- Boolean: supports function calling
    supports_vision INTEGER DEFAULT 0,      -- Boolean: supports image input
    supports_streaming INTEGER DEFAULT 1,   -- Boolean: supports streaming
    pricing_input REAL,                     -- Cost per 1K input tokens (USD)
    pricing_output REAL,                    -- Cost per 1K output tokens (USD)
    is_active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: additional capabilities
    FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE
);

-- =============================================================================
-- WORKSPACES & PROJECTS
-- =============================================================================

-- Workspaces represent project directories or logical groupings
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,                    -- UUID
    name TEXT NOT NULL,
    path TEXT,                              -- Local filesystem path if applicable
    provider TEXT,                          -- Primary provider for this workspace
    provider_workspace_id TEXT,             -- Provider-specific workspace ID
    git_repo TEXT,                          -- Git repository URL
    git_branch TEXT,                        -- Current branch
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT                           -- JSON blob for provider-specific data
);

-- =============================================================================
-- SESSIONS & CONVERSATIONS
-- =============================================================================

-- Sessions represent individual chat conversations
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,                    -- UUID
    workspace_id TEXT,                      -- FK to workspaces.id
    provider TEXT NOT NULL,                 -- Source provider
    provider_session_id TEXT,               -- Provider's session ID
    title TEXT NOT NULL DEFAULT '',         -- User-friendly session name
    model TEXT,                             -- LLM model used
    message_count INTEGER DEFAULT 0,
    token_count INTEGER DEFAULT 0,          -- Total token count
    cost_estimate REAL DEFAULT 0,           -- Estimated cost (USD)
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    archived INTEGER DEFAULT 0,             -- Boolean: 1 = archived
    is_agentic INTEGER DEFAULT 0,           -- Boolean: 1 = agentic workflow
    parent_session_id TEXT,                 -- For forked/branched sessions
    metadata TEXT,                          -- JSON blob for provider-specific data
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- =============================================================================
-- MESSAGES & CONTENT
-- =============================================================================

-- Messages store individual conversation turns
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions.id
    role TEXT NOT NULL,                     -- 'user', 'assistant', 'system', 'tool'
    content TEXT NOT NULL,                  -- Message content (may be large)
    model TEXT,                             -- Model used for this specific message
    token_count INTEGER,                    -- Tokens for this message
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    parent_id TEXT,                         -- For branching conversations
    branch_label TEXT,                      -- Label for this branch (e.g., 'main', 'alt-1')
    sequence_num INTEGER,                   -- Order within branch
    metadata TEXT,                          -- JSON blob (annotations, etc.)
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES messages(id) ON DELETE SET NULL
);

-- Message attachments (files, images, code)
CREATE TABLE IF NOT EXISTS message_attachments (
    id TEXT PRIMARY KEY,                    -- UUID
    message_id TEXT NOT NULL,               -- FK to messages.id
    type TEXT NOT NULL,                     -- 'file', 'image', 'code', 'url', 'audio', 'video'
    name TEXT,                              -- Original filename
    mime_type TEXT,                         -- MIME type
    size_bytes INTEGER,                     -- File size
    content TEXT,                           -- Inline content or path
    url TEXT,                               -- External URL if applicable
    checksum TEXT,                          -- SHA-256 hash
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: dimensions, language, etc.
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- =============================================================================
-- AGENTIC AI SUPPORT
-- =============================================================================

-- Agent definitions (reusable agent configurations)
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,                    -- UUID or name-based ID
    name TEXT NOT NULL UNIQUE,              -- Agent name
    description TEXT,                       -- Agent description
    instruction TEXT NOT NULL,              -- System prompt/instruction
    role TEXT DEFAULT 'assistant',          -- 'assistant', 'coordinator', 'researcher', etc.
    model TEXT,                             -- Preferred model
    provider TEXT,                          -- Preferred provider
    temperature REAL DEFAULT 0.7,           -- Default temperature
    max_tokens INTEGER,                     -- Default max tokens
    tools TEXT,                             -- JSON array of tool names
    sub_agents TEXT,                        -- JSON array of sub-agent IDs
    is_active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT                           -- JSON: additional config
);

-- Tool definitions (functions agents can call)
CREATE TABLE IF NOT EXISTS tools (
    id TEXT PRIMARY KEY,                    -- UUID or name-based ID
    name TEXT NOT NULL UNIQUE,              -- Tool name (e.g., 'web_search')
    description TEXT NOT NULL,              -- Tool description for LLM
    category TEXT DEFAULT 'custom',         -- 'search', 'code', 'file', 'data', 'system'
    parameters TEXT NOT NULL,               -- JSON schema for parameters
    returns TEXT,                           -- JSON schema for return value
    requires_confirmation INTEGER DEFAULT 0,-- Boolean: needs user approval
    is_builtin INTEGER DEFAULT 0,           -- Boolean: built-in vs user-defined
    implementation TEXT,                    -- Code/endpoint for execution
    is_active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT                           -- JSON: rate limits, permissions, etc.
);

-- Tool calls made during sessions
CREATE TABLE IF NOT EXISTS tool_calls (
    id TEXT PRIMARY KEY,                    -- UUID
    message_id TEXT NOT NULL,               -- FK to messages.id (the assistant message)
    session_id TEXT NOT NULL,               -- FK to sessions.id
    tool_name TEXT NOT NULL,                -- Tool that was called
    arguments TEXT NOT NULL,                -- JSON arguments passed
    result TEXT,                            -- JSON result returned
    success INTEGER,                        -- Boolean: call succeeded
    error_message TEXT,                     -- Error if failed
    duration_ms INTEGER,                    -- Execution time
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    completed_at INTEGER,                   -- When execution completed
    metadata TEXT,                          -- JSON: additional context
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Workflow executions (multi-agent orchestration)
CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions.id
    name TEXT,                              -- Workflow name
    type TEXT NOT NULL,                     -- 'sequential', 'parallel', 'loop', 'swarm'
    status TEXT DEFAULT 'pending',          -- 'pending', 'running', 'completed', 'failed'
    root_agent_id TEXT,                     -- Starting agent
    current_agent_id TEXT,                  -- Currently active agent
    iteration INTEGER DEFAULT 0,            -- Current iteration (for loops)
    max_iterations INTEGER,                 -- Max iterations (for loops)
    state TEXT,                             -- JSON: workflow state
    started_at INTEGER,
    completed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (root_agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (current_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

-- Workflow steps (individual agent executions within a workflow)
CREATE TABLE IF NOT EXISTS workflow_steps (
    id TEXT PRIMARY KEY,                    -- UUID
    workflow_id TEXT NOT NULL,              -- FK to workflows.id
    agent_id TEXT NOT NULL,                 -- FK to agents.id
    step_order INTEGER NOT NULL,            -- Execution order
    status TEXT DEFAULT 'pending',          -- 'pending', 'running', 'completed', 'failed', 'skipped'
    input TEXT,                             -- JSON input to this step
    output TEXT,                            -- JSON output from this step
    error TEXT,                             -- Error message if failed
    started_at INTEGER,
    completed_at INTEGER,
    duration_ms INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

-- Agent handoffs (when one agent delegates to another)
CREATE TABLE IF NOT EXISTS agent_handoffs (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions.id
    workflow_id TEXT,                       -- FK to workflows.id (optional)
    from_agent_id TEXT NOT NULL,            -- Delegating agent
    to_agent_id TEXT NOT NULL,              -- Receiving agent
    reason TEXT,                            -- Why handoff occurred
    context TEXT,                           -- JSON: context passed
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE SET NULL,
    FOREIGN KEY (from_agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (to_agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- =============================================================================
-- MEMORY & EMBEDDINGS (RAG SUPPORT)
-- =============================================================================

-- Memory entries (long-term knowledge storage)
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,                    -- UUID
    content TEXT NOT NULL,                  -- The memory content
    type TEXT DEFAULT 'semantic',           -- 'short_term', 'long_term', 'episodic', 'semantic'
    source TEXT DEFAULT 'conversation',     -- 'conversation', 'document', 'user', 'system'
    importance REAL DEFAULT 0.5,            -- 0.0-1.0 importance score
    access_count INTEGER DEFAULT 0,         -- For LRU caching
    last_accessed INTEGER,                  -- Timestamp
    agent_id TEXT,                          -- Associated agent
    session_id TEXT,                        -- Associated session
    workspace_id TEXT,                      -- Associated workspace
    expires_at INTEGER,                     -- Optional expiration
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: tags, categories, etc.
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

-- Vector embeddings for semantic search
CREATE TABLE IF NOT EXISTS embeddings (
    id TEXT PRIMARY KEY,                    -- UUID
    source_type TEXT NOT NULL,              -- 'message', 'memory', 'document', 'chunk'
    source_id TEXT NOT NULL,                -- ID of the source record
    model TEXT NOT NULL,                    -- Embedding model used
    dimensions INTEGER NOT NULL,            -- Vector dimensions (e.g., 1536)
    vector BLOB NOT NULL,                   -- Serialized float32 array
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: additional context
    UNIQUE(source_type, source_id, model)
);

-- Document chunks for RAG pipelines
CREATE TABLE IF NOT EXISTS document_chunks (
    id TEXT PRIMARY KEY,                    -- UUID
    document_id TEXT NOT NULL,              -- Parent document ID
    chunk_index INTEGER NOT NULL,           -- Position in document
    content TEXT NOT NULL,                  -- Chunk text
    token_count INTEGER,                    -- Tokens in this chunk
    start_offset INTEGER,                   -- Character offset start
    end_offset INTEGER,                     -- Character offset end
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: headings, page, etc.
    UNIQUE(document_id, chunk_index)
);

-- Documents for knowledge base
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,                    -- UUID
    workspace_id TEXT,                      -- Associated workspace
    name TEXT NOT NULL,                     -- Document name/title
    type TEXT NOT NULL,                     -- 'text', 'markdown', 'pdf', 'code', 'url'
    source_path TEXT,                       -- Original file path or URL
    content TEXT,                           -- Full content (if stored)
    content_hash TEXT,                      -- SHA-256 for deduplication
    chunk_count INTEGER DEFAULT 0,          -- Number of chunks
    token_count INTEGER,                    -- Total tokens
    is_indexed INTEGER DEFAULT 0,           -- Whether embeddings exist
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,                          -- JSON: author, language, etc.
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

-- =============================================================================
-- VERSION CONTROL & HISTORY
-- =============================================================================

-- Checkpoints provide version history snapshots
CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions.id
    name TEXT NOT NULL,                     -- Version name (e.g., "v1.0")
    description TEXT,                       -- User-provided description
    message_id TEXT,                        -- Message at checkpoint
    message_count INTEGER NOT NULL,         -- Messages at checkpoint
    session_snapshot TEXT NOT NULL,         -- JSON snapshot of session state
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    git_commit TEXT,                        -- Git commit hash if version-controlled
    git_branch TEXT,                        -- Git branch name
    metadata TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE SET NULL
);

-- =============================================================================
-- IMPORT/EXPORT & SHARING
-- =============================================================================

-- Share links track imported shared conversations
CREATE TABLE IF NOT EXISTS share_links (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT,                        -- Linked session after import
    provider TEXT NOT NULL,                 -- 'chatgpt', 'claude', 'gemini', etc.
    url TEXT NOT NULL UNIQUE,               -- Original share URL
    share_id TEXT NOT NULL,                 -- Provider's share identifier
    title TEXT,                             -- Extracted title
    imported INTEGER DEFAULT 0,             -- Boolean: 1 = imported
    imported_at INTEGER,                    -- When imported
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- Import sources track where data came from
CREATE TABLE IF NOT EXISTS import_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,               -- FK to sessions.id
    source_type TEXT NOT NULL,              -- 'file', 'database', 'api', 'share_link'
    source_path TEXT,                       -- File path or URL
    source_provider TEXT,                   -- Original provider
    import_version INTEGER DEFAULT 1,       -- Track re-imports
    checksum TEXT,                          -- Source content hash
    imported_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    metadata TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- =============================================================================
-- ORGANIZATION & TAGGING
-- =============================================================================

-- Tags for organizing content
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,                             -- Hex color
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Many-to-many: sessions <-> tags
CREATE TABLE IF NOT EXISTS session_tags (
    session_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (session_id, tag_id),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Many-to-many: agents <-> tags
CREATE TABLE IF NOT EXISTS agent_tags (
    agent_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (agent_id, tag_id),
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Many-to-many: documents <-> tags
CREATE TABLE IF NOT EXISTS document_tags (
    document_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (document_id, tag_id),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- =============================================================================
-- USAGE & ANALYTICS
-- =============================================================================

-- Token usage tracking per session/provider
CREATE TABLE IF NOT EXISTS usage_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0,
    recorded_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- Events log for audit trail
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,               -- 'session_created', 'message_added', 'tool_called', etc.
    entity_type TEXT,                       -- 'session', 'message', 'agent', 'workflow'
    entity_id TEXT,                         -- ID of the entity
    actor TEXT,                             -- 'user', 'agent:<name>', 'system'
    data TEXT,                              -- JSON event data
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- =============================================================================
-- INDEXES FOR PERFORMANCE
-- =============================================================================

-- Provider/Model indexes
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider_id);
CREATE INDEX IF NOT EXISTS idx_models_type ON models(type);

-- Session indexes
CREATE INDEX IF NOT EXISTS idx_sessions_workspace ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_sessions_provider ON sessions(provider);
CREATE INDEX IF NOT EXISTS idx_sessions_model ON sessions(model);
CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);
CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at);
CREATE INDEX IF NOT EXISTS idx_sessions_archived ON sessions(archived);
CREATE INDEX IF NOT EXISTS idx_sessions_agentic ON sessions(is_agentic);
CREATE INDEX IF NOT EXISTS idx_sessions_parent ON sessions(parent_session_id);

-- Message indexes
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
CREATE INDEX IF NOT EXISTS idx_messages_role ON messages(role);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_messages_parent ON messages(parent_id);
CREATE INDEX IF NOT EXISTS idx_messages_branch ON messages(session_id, branch_label, sequence_num);

-- Attachment indexes
CREATE INDEX IF NOT EXISTS idx_attachments_message ON message_attachments(message_id);
CREATE INDEX IF NOT EXISTS idx_attachments_type ON message_attachments(type);

-- Agent indexes
CREATE INDEX IF NOT EXISTS idx_agents_role ON agents(role);
CREATE INDEX IF NOT EXISTS idx_agents_provider ON agents(provider);

-- Tool call indexes
CREATE INDEX IF NOT EXISTS idx_tool_calls_message ON tool_calls(message_id);
CREATE INDEX IF NOT EXISTS idx_tool_calls_session ON tool_calls(session_id);
CREATE INDEX IF NOT EXISTS idx_tool_calls_tool ON tool_calls(tool_name);
CREATE INDEX IF NOT EXISTS idx_tool_calls_created ON tool_calls(created_at);

-- Workflow indexes
CREATE INDEX IF NOT EXISTS idx_workflows_session ON workflows(session_id);
CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
CREATE INDEX IF NOT EXISTS idx_workflows_type ON workflows(type);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow ON workflow_steps(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_agent ON workflow_steps(agent_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_status ON workflow_steps(status);

-- Handoff indexes
CREATE INDEX IF NOT EXISTS idx_handoffs_session ON agent_handoffs(session_id);
CREATE INDEX IF NOT EXISTS idx_handoffs_workflow ON agent_handoffs(workflow_id);
CREATE INDEX IF NOT EXISTS idx_handoffs_from ON agent_handoffs(from_agent_id);
CREATE INDEX IF NOT EXISTS idx_handoffs_to ON agent_handoffs(to_agent_id);

-- Memory indexes
CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(type);
CREATE INDEX IF NOT EXISTS idx_memories_source ON memories(source);
CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories(agent_id);
CREATE INDEX IF NOT EXISTS idx_memories_session ON memories(session_id);
CREATE INDEX IF NOT EXISTS idx_memories_workspace ON memories(workspace_id);
CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);

-- Embedding indexes
CREATE INDEX IF NOT EXISTS idx_embeddings_source ON embeddings(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_model ON embeddings(model);

-- Document indexes
CREATE INDEX IF NOT EXISTS idx_documents_workspace ON documents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(type);
CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(content_hash);
CREATE INDEX IF NOT EXISTS idx_document_chunks_doc ON document_chunks(document_id);

-- Checkpoint indexes
CREATE INDEX IF NOT EXISTS idx_checkpoints_session ON checkpoints(session_id);
CREATE INDEX IF NOT EXISTS idx_checkpoints_created ON checkpoints(created_at);

-- Share link indexes
CREATE INDEX IF NOT EXISTS idx_share_links_provider ON share_links(provider);
CREATE INDEX IF NOT EXISTS idx_share_links_imported ON share_links(imported);

-- Import source indexes
CREATE INDEX IF NOT EXISTS idx_import_sources_session ON import_sources(session_id);
CREATE INDEX IF NOT EXISTS idx_import_sources_type ON import_sources(source_type);

-- Workspace indexes
CREATE INDEX IF NOT EXISTS idx_workspaces_provider ON workspaces(provider);
CREATE INDEX IF NOT EXISTS idx_workspaces_path ON workspaces(path);

-- Usage indexes
CREATE INDEX IF NOT EXISTS idx_usage_session ON usage_stats(session_id);
CREATE INDEX IF NOT EXISTS idx_usage_provider ON usage_stats(provider);
CREATE INDEX IF NOT EXISTS idx_usage_recorded ON usage_stats(recorded_at);

-- Event indexes
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_events_created ON events(created_at);

-- =============================================================================
-- FULL-TEXT SEARCH
-- =============================================================================

-- Full-text search for message content
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='rowid'
);

-- Full-text search for memory content
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    content,
    content='memories',
    content_rowid='rowid'
);

-- Full-text search for documents
CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    name,
    content,
    content='documents',
    content_rowid='rowid'
);

-- =============================================================================
-- VIEWS FOR COMMON QUERIES
-- =============================================================================

-- Active sessions with workspace info
CREATE VIEW IF NOT EXISTS v_sessions_with_workspace AS
SELECT 
    s.id,
    s.title,
    s.provider,
    s.model,
    s.created_at,
    s.updated_at,
    s.message_count,
    s.token_count,
    s.cost_estimate,
    s.is_agentic,
    w.name AS workspace_name,
    w.path AS workspace_path
FROM sessions s
LEFT JOIN workspaces w ON s.workspace_id = w.id
WHERE s.archived = 0
ORDER BY s.updated_at DESC;

-- Session summary with checkpoint and workflow info
CREATE VIEW IF NOT EXISTS v_session_summary AS
SELECT 
    s.id,
    s.title,
    s.provider,
    s.model,
    s.message_count,
    s.token_count,
    s.cost_estimate,
    s.is_agentic,
    s.created_at,
    s.updated_at,
    COUNT(DISTINCT c.id) AS checkpoint_count,
    COUNT(DISTINCT wf.id) AS workflow_count,
    COUNT(DISTINCT tc.id) AS tool_call_count,
    MAX(c.created_at) AS last_checkpoint
FROM sessions s
LEFT JOIN checkpoints c ON s.id = c.session_id
LEFT JOIN workflows wf ON s.id = wf.session_id
LEFT JOIN tool_calls tc ON s.id = tc.session_id
GROUP BY s.id;

-- Agent overview with tool counts
CREATE VIEW IF NOT EXISTS v_agent_overview AS
SELECT 
    a.id,
    a.name,
    a.description,
    a.role,
    a.model,
    a.provider,
    json_array_length(a.tools) AS tool_count,
    json_array_length(a.sub_agents) AS sub_agent_count,
    a.is_active,
    a.created_at,
    a.updated_at
FROM agents a;

-- Workflow execution summary
CREATE VIEW IF NOT EXISTS v_workflow_summary AS
SELECT 
    w.id,
    w.name,
    w.type,
    w.status,
    w.iteration,
    w.max_iterations,
    s.title AS session_title,
    ra.name AS root_agent_name,
    ca.name AS current_agent_name,
    COUNT(ws.id) AS step_count,
    SUM(CASE WHEN ws.status = 'completed' THEN 1 ELSE 0 END) AS completed_steps,
    w.started_at,
    w.completed_at,
    (w.completed_at - w.started_at) AS duration_seconds
FROM workflows w
LEFT JOIN sessions s ON w.session_id = s.id
LEFT JOIN agents ra ON w.root_agent_id = ra.id
LEFT JOIN agents ca ON w.current_agent_id = ca.id
LEFT JOIN workflow_steps ws ON w.id = ws.workflow_id
GROUP BY w.id;

-- Recent tool calls with context
CREATE VIEW IF NOT EXISTS v_recent_tool_calls AS
SELECT 
    tc.id,
    tc.tool_name,
    tc.success,
    tc.duration_ms,
    tc.created_at,
    s.title AS session_title,
    s.provider,
    t.category AS tool_category,
    t.description AS tool_description
FROM tool_calls tc
JOIN sessions s ON tc.session_id = s.id
LEFT JOIN tools t ON tc.tool_name = t.name
ORDER BY tc.created_at DESC
LIMIT 100;

-- Memory search results view helper
CREATE VIEW IF NOT EXISTS v_memory_with_context AS
SELECT 
    m.id,
    m.content,
    m.type,
    m.source,
    m.importance,
    m.access_count,
    m.created_at,
    a.name AS agent_name,
    s.title AS session_title,
    w.name AS workspace_name
FROM memories m
LEFT JOIN agents a ON m.agent_id = a.id
LEFT JOIN sessions s ON m.session_id = s.id
LEFT JOIN workspaces w ON m.workspace_id = w.id;

-- Usage statistics by provider
CREATE VIEW IF NOT EXISTS v_usage_by_provider AS
SELECT 
    provider,
    model,
    COUNT(*) AS request_count,
    SUM(prompt_tokens) AS total_prompt_tokens,
    SUM(completion_tokens) AS total_completion_tokens,
    SUM(total_tokens) AS total_tokens,
    SUM(cost_usd) AS total_cost_usd,
    MIN(recorded_at) AS first_usage,
    MAX(recorded_at) AS last_usage
FROM usage_stats
GROUP BY provider, model
ORDER BY total_tokens DESC;

-- Pending share links
CREATE VIEW IF NOT EXISTS v_pending_shares AS
SELECT 
    id,
    url,
    provider,
    share_id,
    title,
    created_at
FROM share_links
WHERE imported = 0
ORDER BY created_at DESC;

-- =============================================================================
-- TRIGGERS FOR DATA INTEGRITY
-- =============================================================================

-- Update session message_count on message insert
CREATE TRIGGER IF NOT EXISTS tr_update_session_message_count_insert
AFTER INSERT ON messages
BEGIN
    UPDATE sessions 
    SET message_count = (SELECT COUNT(*) FROM messages WHERE session_id = NEW.session_id),
        updated_at = strftime('%s', 'now')
    WHERE id = NEW.session_id;
END;

-- Update session message_count on message delete
CREATE TRIGGER IF NOT EXISTS tr_update_session_message_count_delete
AFTER DELETE ON messages
BEGIN
    UPDATE sessions 
    SET message_count = (SELECT COUNT(*) FROM messages WHERE session_id = OLD.session_id),
        updated_at = strftime('%s', 'now')
    WHERE id = OLD.session_id;
END;

-- Update session token_count on message insert with tokens
CREATE TRIGGER IF NOT EXISTS tr_update_session_tokens
AFTER INSERT ON messages
WHEN NEW.token_count IS NOT NULL
BEGIN
    UPDATE sessions 
    SET token_count = token_count + NEW.token_count,
        updated_at = strftime('%s', 'now')
    WHERE id = NEW.session_id;
END;

-- Track agentic sessions (mark session as agentic when workflow created)
CREATE TRIGGER IF NOT EXISTS tr_mark_session_agentic
AFTER INSERT ON workflows
BEGIN
    UPDATE sessions 
    SET is_agentic = 1,
        updated_at = strftime('%s', 'now')
    WHERE id = NEW.session_id;
END;

-- Update document chunk count
CREATE TRIGGER IF NOT EXISTS tr_update_document_chunk_count
AFTER INSERT ON document_chunks
BEGIN
    UPDATE documents 
    SET chunk_count = (SELECT COUNT(*) FROM document_chunks WHERE document_id = NEW.document_id),
        updated_at = strftime('%s', 'now')
    WHERE id = NEW.document_id;
END;

-- Update memory access tracking
CREATE TRIGGER IF NOT EXISTS tr_update_memory_access
AFTER UPDATE OF access_count ON memories
BEGIN
    UPDATE memories 
    SET last_accessed = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- =============================================================================
-- DEFAULT DATA
-- =============================================================================

-- Insert common providers
INSERT OR IGNORE INTO providers (id, name, type, capabilities, default_model) VALUES
    ('openai', 'OpenAI', 'cloud', '["chat","embeddings","tools","vision"]', 'gpt-4o'),
    ('anthropic', 'Anthropic', 'cloud', '["chat","tools","vision"]', 'claude-3-5-sonnet-20241022'),
    ('google', 'Google AI', 'cloud', '["chat","embeddings","tools","vision"]', 'gemini-2.0-flash-exp'),
    ('copilot', 'GitHub Copilot', 'cloud', '["chat","code"]', 'gpt-4o'),
    ('ollama', 'Ollama', 'local', '["chat","embeddings"]', 'llama3.2'),
    ('lmstudio', 'LM Studio', 'local', '["chat","embeddings"]', NULL),
    ('llamacpp', 'llama.cpp', 'local', '["chat","embeddings"]', NULL),
    ('jan', 'Jan', 'local', '["chat"]', NULL),
    ('gpt4all', 'GPT4All', 'local', '["chat","embeddings"]', NULL);

-- Insert common tool definitions
INSERT OR IGNORE INTO tools (id, name, description, category, parameters, is_builtin) VALUES
    ('web_search', 'web_search', 'Search the web for information', 'search', '{"type":"object","properties":{"query":{"type":"string","description":"Search query"},"max_results":{"type":"integer","description":"Maximum results to return","default":5}},"required":["query"]}', 1),
    ('read_file', 'read_file', 'Read contents of a file', 'file', '{"type":"object","properties":{"path":{"type":"string","description":"File path to read"}},"required":["path"]}', 1),
    ('write_file', 'write_file', 'Write content to a file', 'file', '{"type":"object","properties":{"path":{"type":"string","description":"File path to write"},"content":{"type":"string","description":"Content to write"}},"required":["path","content"]}', 1),
    ('execute_code', 'execute_code', 'Execute code in a sandboxed environment', 'code', '{"type":"object","properties":{"language":{"type":"string","description":"Programming language"},"code":{"type":"string","description":"Code to execute"}},"required":["language","code"]}', 1),
    ('http_request', 'http_request', 'Make an HTTP request', 'data', '{"type":"object","properties":{"method":{"type":"string","enum":["GET","POST","PUT","DELETE"]},"url":{"type":"string"},"headers":{"type":"object"},"body":{"type":"string"}},"required":["method","url"]}', 1);
