<p align="center">
  <h1 align="center">🗄️ Chasm</h1>
  <p align="center">
    <strong>Universal Chat Session Manager</strong><br>
    Harvest, merge, and analyze your AI chat history
  </p>
</p>

<p align="center">
  <a href="https://crates.io/crates/chasm"><img src="https://img.shields.io/crates/v/chasm.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/chasm"><img src="https://docs.rs/chasm/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/nervosys/chasm/actions"><img src="https://github.com/nervosys/chasm/workflows/CI/badge.svg" alt="CI Status"></a>
  <a href="https://codecov.io/gh/nervosys/chasm"><img src="https://codecov.io/gh/nervosys/chasm/branch/main/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/LICENSE--2.0-blue.svg" alt="License"></a>
</p>

---

**Chasm** extracts and unifies chat sessions from AI coding assistants like GitHub Copilot, Cursor, and more. Never lose your AI conversations again.

## ✨ Features

- 🔍 **Harvest** - Extract chat sessions from VS Code, Cursor, Windsurf, and other editors
- 🔀 **Merge** - Combine sessions across workspaces and time periods
- 📊 **Analyze** - Get statistics on your AI assistant usage
- 🔌 **API Server** - REST API for building custom integrations
- 🤖 **MCP Tools** - Model Context Protocol support for AI agent integration
- 🗃️ **Universal Database** - SQLite-based storage that normalizes all providers

## 📦 Installation

### From crates.io

```bash
cargo install chasm
```

### From source

```bash
git clone https://github.com/nervosys/chasm.git
cd chasm
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/nervosys/chasm/releases):

| Platform    | Download                                                                       |
| ----------- | ------------------------------------------------------------------------------ |
| Windows x64 | [chasm-windows-x64.zip](https://github.com/nervosys/chasm/releases/latest)     |
| macOS x64   | [chasm-darwin-x64.tar.gz](https://github.com/nervosys/chasm/releases/latest)   |
| macOS ARM   | [chasm-darwin-arm64.tar.gz](https://github.com/nervosys/chasm/releases/latest) |
| Linux x64   | [chasm-linux-x64.tar.gz](https://github.com/nervosys/chasm/releases/latest)    |

### Docker

```bash
docker pull ghcr.io/nervosys/chasm:latest
docker run -v ~/.chasm:/data ghcr.io/nervosys/chasm list workspaces
```

## 🚀 Quick Start

### List discovered workspaces

```bash
chasm list workspaces
```

```
┌────────────────────────┬──────────────────┬──────────┬────────────┐
│ Name                   │ Provider         │ Sessions │ Updated    │
├────────────────────────┼──────────────────┼──────────┼────────────┤
│ my-project             │ GitHub Copilot   │ 15       │ 2026-01-08 │
│ another-project        │ Cursor           │ 8        │ 2026-01-07 │
│ open-source-contrib    │ GitHub Copilot   │ 23       │ 2026-01-06 │
└────────────────────────┴──────────────────┴──────────┴────────────┘
```

### Show sessions for a project

```bash
chasm show path /path/to/your/project
```

### Harvest sessions from VS Code

```bash
chasm harvest
```

### Export a session to Markdown

```bash
chasm export session abc123 --format markdown --output chat.md
```

### Start the API server

```bash
chasm api serve --port 8787
```

## 📖 CLI Reference

### Core Commands

| Command                          | Description                                      |
| -------------------------------- | ------------------------------------------------ |
| `chasm list workspaces`          | List all discovered workspaces                   |
| `chasm list sessions`            | List sessions (optionally filtered by workspace) |
| `chasm show session <id>`        | Display full session content                     |
| `chasm show path <path>`         | Show sessions for a project path                 |
| `chasm find workspace <pattern>` | Search workspaces by name                        |
| `chasm find session <pattern>`   | Search sessions by content                       |

### Data Management

| Command                        | Description                           |
| ------------------------------ | ------------------------------------- |
| `chasm harvest`                | Scan and import sessions from editors |
| `chasm merge workspace <name>` | Merge sessions from a workspace       |
| `chasm export session <id>`    | Export session to file                |
| `chasm import <file>`          | Import sessions from file             |

### Server

| Command           | Description               |
| ----------------- | ------------------------- |
| `chasm api serve` | Start the REST API server |
| `chasm mcp serve` | Start the MCP tool server |

### Options

```bash
chasm --help          # Show all commands
chasm <cmd> --help    # Show help for a specific command
chasm --version       # Show version
```

## 🔌 API Server

Start the REST API server for integration with web/mobile apps:

```bash
chasm api serve --host 0.0.0.0 --port 8787
```

### Endpoints

| Method | Endpoint                  | Description               |
| ------ | ------------------------- | ------------------------- |
| GET    | `/api/health`             | Health check              |
| GET    | `/api/workspaces`         | List workspaces           |
| GET    | `/api/workspaces/:id`     | Get workspace details     |
| GET    | `/api/sessions`           | List sessions             |
| GET    | `/api/sessions/:id`       | Get session with messages |
| GET    | `/api/sessions/search?q=` | Search sessions           |
| GET    | `/api/stats`              | Database statistics       |
| GET    | `/api/providers`          | List supported providers  |

### Example

```bash
curl http://localhost:8787/api/stats
```

```json
{
  "success": true,
  "data": {
    "totalSessions": 330,
    "totalMessages": 19068,
    "totalWorkspaces": 138,
    "totalToolInvocations": 122712
  }
}
```

## 🤖 MCP Integration

Chasm provides [Model Context Protocol](https://modelcontextprotocol.io/) tools for AI agent integration:

```bash
chasm mcp serve
```

### Available Tools

- `chasm_list_workspaces` - List all workspaces
- `chasm_list_sessions` - List sessions in a workspace
- `chasm_get_session` - Get full session content
- `chasm_search_sessions` - Search across all sessions
- `chasm_get_stats` - Get database statistics

## 🗃️ Supported Providers

### Editor-based
- ✅ GitHub Copilot (VS Code)
- ✅ Cursor
- ✅ Windsurf
- ✅ Continue.dev

### Local LLMs
- ✅ Ollama
- ✅ LM Studio
- ✅ GPT4All
- ✅ LocalAI
- ✅ llama.cpp / llamafile

### Cloud APIs
- ✅ OpenAI / ChatGPT
- ✅ Anthropic / Claude
- ✅ Google / Gemini
- ✅ Perplexity

## 📁 Database

Chasm stores all data in a local SQLite database:

| Platform | Location                                   |
| -------- | ------------------------------------------ |
| Windows  | `%LOCALAPPDATA%\csm\csm.db`                |
| macOS    | `~/Library/Application Support/csm/csm.db` |
| Linux    | `~/.local/share/csm/csm.db`                |

### Schema

```
Workspaces ──< Sessions ──< Messages
                  │
                  ├──< Checkpoints
                  └──< ShareLinks
```

## 🛠️ Development

### Prerequisites

- Rust 1.75+
- Git

### Building

```bash
git clone https://github.com/nervosys/chasm.git
cd chasm
cargo build --release
```

### Running tests

```bash
cargo test
```

### Running the TUI

```bash
cargo run -- tui
```

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## 🔒 Security

For security issues, please see our [Security Policy](SECURITY.md).

## 📞 Support

- 📖 [Documentation](https://docs.rs/chasm)
- 💬 [GitHub Discussions](https://github.com/nervosys/chasm/discussions)
- 🐛 [Issue Tracker](https://github.com/nervosys/chasm/issues)

---

<p align="center">
  Made with ❤️ by <a href="https://nervosys.com">Nervosys</a>
</p>
