# Руководство по установке и настройке rg-md

Цель: предоставить разработчикам и автономным агентам инструкции по установке rg-md и развертыванию навыка агента.

## Команды установки

### 1. Linux и macOS (Bash/Zsh)
Выполните в терминале:
```bash
curl -fsSL https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.sh | bash
```
Устанавливает `rg-md` и символическую ссылку `md-rg` в `~/.local/bin/`, выставляет права на запуск и добавляет директорию в PATH.

### 2. Windows (PowerShell)
Выполните в PowerShell:
```powershell
irm https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.ps1 | iex
```
Загружает `rg-md.exe` и `md-rg.exe` в `%LOCALAPPDATA%\Programs\rg-md\bin` и регистрирует каталог в переменной окружения PATH.

### 3. Сборка из исходников через Cargo
```bash
cargo install --git https://github.com/SpIvanM/rg-md.git
```

---

## Пути размещения навыков агентов

Скопируйте `skills/rg-md/SKILL.md` в соответствующий каталог вашего агента:

| ИИ-агент | Путь к навыку (Skill) | Путь к файлу правил |
| :--- | :--- | :--- |
| Google Antigravity (AGY) | `~/.gemini/config/skills/rg-md/SKILL.md` | `~/.gemini/AGENTS.md` |
| Cursor | `.cursor/rules/rg-md.mdc` | `.cursorrules` |
| Claude Code | `~/.claude/skills/rg-md/SKILL.md` | `CLAUDE.md` |
| Windsurf / Cascade | `.windsurf/rules/rg-md.md` | `.windsurfrules` |
| Универсальный агент | `.agents/skills/rg-md/SKILL.md` | `AGENTS.md` |

---

## Решение проблем

### Команда не найдена в активной оболочке
- Linux: Выполните `source ~/.bashrc` или добавьте путь `export PATH="$HOME/.local/bin:$PATH"`.
- Windows: Выполните `$env:Path = [Environment]::GetEnvironmentVariable("Path", "User") + ";$env:Path"` или перезапустите терминал.

### Отсутствие утилиты ripgrep
rg-md использует ripgrep для выполнения сканирования. Установите ripgrep при его отсутствии:
- Ubuntu/Debian: `sudo apt-get install ripgrep`
- macOS: `brew install ripgrep`
- Windows: `winget install BurntSushi.ripgrep.MSVC`
