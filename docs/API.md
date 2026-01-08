# Chasm CLI API Reference

This document describes the REST API endpoints provided by the Chasm CLI API server.

## Starting the API Server

```bash
chasm api serve [--port 8787] [--host 0.0.0.0]
```

Default: `http://0.0.0.0:8787`

## Authentication

Most endpoints are public. For authenticated endpoints (agents, swarms, settings), the API uses JWT bearer tokens:

```
Authorization: Bearer <token>
```

## Response Format

All responses follow this format:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

---

## Health & System

### GET /api/health

Health check endpoint.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "timestamp": "2026-01-08T10:00:00Z"
  }
}
```

### GET /api/system/info

System information.

**Response:**
```json
{
  "success": true,
  "data": {
    "version": "1.0.0",
    "rustVersion": "1.75.0",
    "platform": "windows",
    "databasePath": "C:/Users/.../csm.db",
    "databaseSize": 1048576,
    "uptime": 3600
  }
}
```

### GET /api/system/health

Detailed health status.

**Response:**
```json
{
  "success": true,
  "data": {
    "database": "healthy",
    "storage": "healthy",
    "memory": { "used": 100000000, "total": 16000000000 }
  }
}
```

---

## Workspaces

### GET /api/workspaces

List all workspaces.

**Query Parameters:**
- `provider` (optional): Filter by provider (e.g., "copilot", "cursor")
- `search` (optional): Search by name or path

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "workspace-uuid",
      "name": "my-project",
      "path": "/path/to/project",
      "provider": "copilot",
      "sessionCount": 15,
      "lastAccessed": 1704700000,
      "createdAt": 1704000000
    }
  ]
}
```

### GET /api/workspaces/{id}

Get a specific workspace.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "workspace-uuid",
    "name": "my-project",
    "path": "/path/to/project",
    "provider": "copilot",
    "sessionCount": 15,
    "sessions": [...]
  }
}
```

---

## Sessions

### GET /api/sessions

List all sessions.

**Query Parameters:**
- `workspace_id` (optional): Filter by workspace
- `provider` (optional): Filter by provider
- `search` (optional): Search in title
- `limit` (optional): Max results (default: 50)
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "session-uuid",
      "title": "Implement feature X",
      "workspaceId": "workspace-uuid",
      "provider": "copilot",
      "model": "gpt-4o",
      "messageCount": 24,
      "tokenCount": 15000,
      "createdAt": 1704000000,
      "updatedAt": 1704700000
    }
  ]
}
```

### GET /api/sessions/{id}

Get a specific session with messages.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "session-uuid",
    "title": "Implement feature X",
    "messages": [
      {
        "id": "msg-uuid",
        "role": "user",
        "content": "How do I implement...",
        "timestamp": 1704000000
      },
      {
        "id": "msg-uuid-2",
        "role": "assistant",
        "content": "Here's how...",
        "timestamp": 1704000001
      }
    ]
  }
}
```

### GET /api/sessions/search

Search sessions.

**Query Parameters:**
- `q` (required): Search query
- `limit` (optional): Max results

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "session-uuid",
      "title": "Matching session",
      "snippet": "...matching content...",
      "score": 0.95
    }
  ]
}
```

---

## Providers

### GET /api/providers

List configured providers.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "copilot",
      "name": "GitHub Copilot",
      "type": "cloud",
      "icon": "🤖",
      "models": ["gpt-4o", "gpt-4o-mini", "claude-sonnet-4"],
      "status": "connected"
    },
    {
      "id": "ollama",
      "name": "Ollama",
      "type": "local",
      "endpoint": "http://localhost:11434",
      "models": ["llama3.2", "codellama"],
      "status": "connected"
    }
  ]
}
```

---

## Agents

### GET /api/agents

List all agents.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "agent-uuid",
      "name": "researcher",
      "description": "Research assistant",
      "instruction": "You are a helpful research assistant...",
      "role": "researcher",
      "model": "gpt-4o",
      "provider": "openai",
      "temperature": 0.7,
      "maxTokens": 4096,
      "tools": ["web_search", "read_file"],
      "subAgents": [],
      "isActive": true,
      "createdAt": 1704000000,
      "updatedAt": 1704700000
    }
  ]
}
```

### GET /api/agents/{id}

Get a specific agent.

### POST /api/agents

Create a new agent.

**Request Body:**
```json
{
  "name": "researcher",
  "description": "Research assistant",
  "instruction": "You are a helpful research assistant...",
  "role": "researcher",
  "model": "gpt-4o",
  "provider": "openai",
  "temperature": 0.7,
  "maxTokens": 4096,
  "tools": ["web_search"]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "new-agent-uuid",
    "name": "researcher",
    ...
  }
}
```

### PUT /api/agents/{id}

Update an agent.

**Request Body:**
```json
{
  "name": "updated-name",
  "temperature": 0.5
}
```

### DELETE /api/agents/{id}

Delete an agent.

**Response:**
```json
{
  "success": true,
  "data": { "deleted": true }
}
```

---

## Swarms

### GET /api/swarms

List all swarms.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "swarm-uuid",
      "name": "code-review-team",
      "description": "Multi-agent code review",
      "orchestration": "hierarchical",
      "agents": [
        { "agent_id": "agent-1", "role": "coordinator" },
        { "agent_id": "agent-2", "role": "reviewer" }
      ],
      "maxIterations": 10,
      "status": "idle",
      "createdAt": 1704000000,
      "updatedAt": 1704700000
    }
  ]
}
```

### POST /api/swarms

Create a new swarm.

**Request Body:**
```json
{
  "name": "code-review-team",
  "description": "Multi-agent code review",
  "orchestration": "hierarchical",
  "agents": [
    { "agent_id": "agent-1", "role": "coordinator" },
    { "agent_id": "agent-2", "role": "reviewer" }
  ],
  "maxIterations": 10
}
```

### DELETE /api/swarms/{id}

Delete a swarm.

---

## Statistics

### GET /api/stats

Get database statistics overview.

**Response:**
```json
{
  "success": true,
  "data": {
    "totalWorkspaces": 10,
    "totalSessions": 150,
    "totalMessages": 5000,
    "totalTokens": 2500000,
    "byProvider": {
      "copilot": { "sessions": 100, "messages": 3500 },
      "cursor": { "sessions": 50, "messages": 1500 }
    },
    "byModel": {
      "gpt-4o": { "sessions": 80, "tokens": 1500000 },
      "claude-sonnet-4": { "sessions": 70, "tokens": 1000000 }
    },
    "databaseSize": 52428800,
    "lastUpdated": 1704700000
  }
}
```

---

## MCP Tools

The API exposes Model Context Protocol (MCP) tools for AI agent integration.

### GET /api/mcp/tools

List available MCP tools.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "name": "list_workspaces",
      "description": "List all workspaces in the chat history database",
      "inputSchema": {
        "type": "object",
        "properties": {
          "provider": { "type": "string", "description": "Filter by provider" }
        }
      }
    }
  ]
}
```

### POST /api/mcp/call

Call an MCP tool.

**Request Body:**
```json
{
  "name": "list_workspaces",
  "arguments": {
    "provider": "copilot"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "content": [
      {
        "type": "text",
        "text": "Found 5 workspaces..."
      }
    ]
  }
}
```

### POST /api/mcp/batch

Call multiple MCP tools in batch.

**Request Body:**
```json
{
  "calls": [
    { "name": "list_workspaces", "arguments": {} },
    { "name": "get_stats", "arguments": {} }
  ]
}
```

### GET /api/mcp/system-prompt

Get the CSM system prompt for AI assistants.

---

## Settings

### GET /api/settings

Get all settings.

**Response:**
```json
{
  "success": true,
  "data": {
    "theme": "system",
    "defaultProvider": "copilot",
    "autoSync": true,
    "syncInterval": 300000,
    "maxHistoryDays": 365,
    "enableNotifications": true,
    "compactMode": false
  }
}
```

### PUT /api/settings

Update settings.

**Request Body:**
```json
{
  "theme": "dark",
  "autoSync": false
}
```

---

## Sync API

Real-time synchronization endpoints for multi-device sync.

### GET /sync/version

Get current sync version.

**Response:**
```json
{
  "version": 42
}
```

### GET /sync/delta?from={version}

Get changes since a specific version.

**Response:**
```json
{
  "fromVersion": 40,
  "toVersion": 42,
  "events": [
    {
      "id": "event-uuid",
      "type": "session_created",
      "data": { ... },
      "timestamp": 1704700000,
      "version": 41
    }
  ]
}
```

### POST /sync/event

Push a sync event.

**Request Body:**
```json
{
  "type": "session_updated",
  "data": {
    "sessionId": "session-uuid",
    "changes": { ... }
  }
}
```

### GET /sync/snapshot

Get full data snapshot for initial sync.

### GET /sync/subscribe

Server-Sent Events (SSE) stream for real-time updates.

**Response:** SSE stream with events:
```
event: sync
data: {"type": "session_created", "data": {...}}

event: heartbeat
data: {"timestamp": 1704700000}
```

---

## SWE Mode

Software Engineering mode for project-aware AI assistance.

### GET /api/swe/projects

List SWE projects.

### POST /api/swe/projects

Create a new SWE project.

**Request Body:**
```json
{
  "name": "my-project",
  "path": "/path/to/project",
  "description": "A web application"
}
```

### GET /api/swe/projects/{id}/context

Get project context (files, structure, etc.).

### POST /api/swe/projects/{id}/execute

Execute a tool in project context.

### Memory & Rules

- `GET /api/swe/projects/{id}/memory` - List memory entries
- `POST /api/swe/projects/{id}/memory` - Create memory entry
- `GET /api/swe/projects/{id}/rules` - List project rules
- `POST /api/swe/projects/{id}/rules` - Create project rule

---

## Error Codes

| Code              | Description                        |
| ----------------- | ---------------------------------- |
| `NOT_FOUND`       | Resource not found                 |
| `INVALID_REQUEST` | Invalid request body or parameters |
| `DATABASE_ERROR`  | Database operation failed          |
| `UNAUTHORIZED`    | Authentication required            |
| `FORBIDDEN`       | Insufficient permissions           |

---

## Rate Limiting

Currently no rate limiting is enforced. This may change in future versions.

## CORS

The API server accepts requests from:
- `localhost:3000`, `localhost:5173` (web development)
- `localhost:8081`, `localhost:19006` (Expo/React Native)
- Any `localhost` or `127.0.0.1` origin

For production deployments, configure allowed origins via environment variables.
