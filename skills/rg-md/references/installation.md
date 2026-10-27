# rg-md Installation and Configuration Guide

Goal: guide developers and autonomous agents on installing rg-md and configuring agent skills when the binary is not present in the environment.

## Installation Commands

### 1. Linux and macOS (Bash/Zsh)
Run in terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.sh | bash
```
Installs `rg-md` and symlink `md-rg` into `~/.local/bin/`, marks them executable, and registers `~/.local/bin` in PATH.

### 2. Windows (PowerShell)
Run in PowerShell:
```powershell
irm https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.ps1 | iex
```
Installs `rg-md.exe` and `md-rg.exe` into `%LOCALAPPDATA%\Programs\rg-md\bin` and registers the directory in User PATH.

### 3. Build from Source via Cargo
```bash
cargo install --git https://github.com/SpIvanM/rg-md.git
```

---

## Agent Skill Deployment Locations

Copy `skills/rg-md/SKILL.md` to your agent configuration:

| Coding Agent | Skill Location | Rule File Location |
| :--- | :--- | :--- |
| Google Antigravity (AGY) | `~/.gemini/config/skills/rg-md/SKILL.md` | `~/.gemini/AGENTS.md` |
| Cursor | `.cursor/rules/rg-md.mdc` | `.cursorrules` |
| Claude Code | `~/.claude/skills/rg-md/SKILL.md` | `CLAUDE.md` |
| Windsurf / Cascade | `.windsurf/rules/rg-md.md` | `.windsurfrules` |
| Generic Agent | `.agents/skills/rg-md/SKILL.md` | `AGENTS.md` |

---

## Troubleshooting

### Command Not Found in Shell
- Linux: Run `source ~/.bashrc` or export `export PATH="$HOME/.local/bin:$PATH"`.
- Windows: Run `$env:Path = [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:Path"` or restart terminal.

### Missing Ripgrep Dependency
rg-md invokes ripgrep under the hood. Install ripgrep if missing:
- Ubuntu/Debian: `sudo apt-get install ripgrep`
- macOS: `brew install ripgrep`
- Windows: `winget install BurntSushi.ripgrep.MSVC`
