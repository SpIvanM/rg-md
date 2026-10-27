#!/usr/bin/env bash
# Install script for rg-md on Linux and macOS
# Implements idempotent marker replacement and agent environment detection.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.sh | bash

set -euo pipefail

REPO="SpIvanM/rg-md"
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

echo -e "\033[0;36m==========================================================\033[0m"
echo -e "\033[0;36m   rg-md — Markdown-Formatted Code Search Engine Installer\033[0m"
echo -e "\033[0;36m==========================================================\033[0m"

ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

if [ "${OS}" != "linux" ] && [ "${OS}" != "darwin" ]; then
    echo "Unsupported OS: ${OS}. Please build from source using cargo."
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
REPO_ROOT=""
if [ -n "${SCRIPT_DIR}" ] && [ -f "${SCRIPT_DIR}/../Cargo.toml" ]; then
    REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
elif [ -f "./Cargo.toml" ]; then
    REPO_ROOT="$(pwd)"
fi

echo -e "\033[0;90m[1/3] Installing rg-md binary...\033[0m"

INSTALLED=0

# Priority A: Local build if running inside repo
if [ -n "${REPO_ROOT}" ]; then
    if [ -f "${REPO_ROOT}/target/release/rg-md" ]; then
        cp "${REPO_ROOT}/target/release/rg-md" "${INSTALL_DIR}/rg-md"
        INSTALLED=1
        echo -e "\033[0;32m      Installed from local target/release/rg-md\033[0m"
    elif [ -f "${REPO_ROOT}/target/debug/rg-md" ]; then
        cp "${REPO_ROOT}/target/debug/rg-md" "${INSTALL_DIR}/rg-md"
        INSTALLED=1
        echo -e "\033[0;32m      Installed from local target/debug/rg-md\033[0m"
    fi
fi

# Priority B: Download precompiled binary from GitHub Releases
if [ "${INSTALLED}" -eq 0 ]; then
    TAR_URL="https://github.com/${REPO}/releases/latest/download/rg-md-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
    TMP_DIR="$(mktemp -d)"
    echo -e "\033[0;90m      Attempting download from ${TAR_URL}...\033[0m"
    if curl -fsSL "${TAR_URL}" -o "${TMP_DIR}/rg-md.tar.gz" 2>/dev/null; then
        tar -xzf "${TMP_DIR}/rg-md.tar.gz" -C "${TMP_DIR}"
        cp "${TMP_DIR}/rg-md" "${INSTALL_DIR}/rg-md"
        rm -rf "${TMP_DIR}"
        INSTALLED=1
        echo -e "\033[0;32m      Downloaded and installed release binary.\033[0m"
    else
        rm -rf "${TMP_DIR}"
    fi
fi

# Priority C: Build and install via Cargo
if [ "${INSTALLED}" -eq 0 ]; then
    if command -v cargo >/dev/null 2>&1; then
        if [ -n "${REPO_ROOT}" ] && [ -f "${REPO_ROOT}/Cargo.toml" ]; then
            echo -e "\033[0;90m      Building release binary via local cargo...\033[0m"
            cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml"
            cp "${REPO_ROOT}/target/release/rg-md" "${INSTALL_DIR}/rg-md"
            INSTALLED=1
        else
            echo -e "\033[0;90m      Installing from remote repository via cargo...\033[0m"
            cargo install --git "https://github.com/${REPO}.git"
            if [ -f "${HOME}/.cargo/bin/rg-md" ]; then
                cp "${HOME}/.cargo/bin/rg-md" "${INSTALL_DIR}/rg-md"
                INSTALLED=1
            fi
        fi
        echo -e "\033[0;32m      Built and installed via cargo.\033[0m"
    fi
fi

if [ "${INSTALLED}" -eq 0 ]; then
    echo "Failed to install rg-md binary. Please install Rust (https://rustup.rs) or build manually."
    exit 1
fi

chmod +x "${INSTALL_DIR}/rg-md"
ln -sf "${INSTALL_DIR}/rg-md" "${INSTALL_DIR}/md-rg"
echo -e "\033[0;32m      Linked alias: ${INSTALL_DIR}/md-rg -> rg-md\033[0m"

# Ensure INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    if [ -f "${HOME}/.bashrc" ]; then
        echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "${HOME}/.bashrc"
    fi
    if [ -f "${HOME}/.zshrc" ]; then
        echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "${HOME}/.zshrc"
    fi
    export PATH="${INSTALL_DIR}:${PATH}"
    echo -e "\033[0;32m      Added ${INSTALL_DIR} to PATH.\033[0m"
else
    echo -e "\033[0;90m      ${INSTALL_DIR} is already in PATH.\033[0m"
fi

echo -e "\033[0;90m[2/3] Detecting agent environments and deploying skills...\033[0m"

# Resolve skill content (local or remote)
SKILL_CONTENT=""
if [ -n "${REPO_ROOT}" ] && [ -f "${REPO_ROOT}/skills/rg-md/SKILL.md" ]; then
    SKILL_CONTENT="$(cat "${REPO_ROOT}/skills/rg-md/SKILL.md")"
else
    SKILL_CONTENT="$(curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/skills/rg-md/SKILL.md" 2>/dev/null || true)"
fi

deploy_skill() {
    local target_path="$1"
    local dir
    dir="$(dirname "${target_path}")"
    mkdir -p "${dir}"
    if [ -n "${SKILL_CONTENT}" ]; then
        echo "${SKILL_CONTENT}" > "${target_path}"
        echo -e "\033[0;32m      Deployed skill: ${target_path}\033[0m"
    fi
}

# 1. Google Antigravity / Gemini
if [ -d "${HOME}/.gemini" ]; then
    deploy_skill "${HOME}/.gemini/config/skills/rg-md/SKILL.md"
fi

# 2. Generic Agents directory
if [ -d "${HOME}/.agents" ]; then
    deploy_skill "${HOME}/.agents/skills/rg-md/SKILL.md"
fi
if [ -d "./.agents" ]; then
    deploy_skill "./.agents/skills/rg-md/SKILL.md"
fi

# 3. Cursor
if [ -d "${HOME}/.cursor" ] || [ -d "./.cursor" ] || [ -f "./.cursorrules" ]; then
    if [ -d "./.cursor" ]; then
        deploy_skill "./.cursor/rules/rg-md.mdc"
    elif [ -d "${HOME}/.cursor" ]; then
        deploy_skill "${HOME}/.cursor/rules/rg-md.mdc"
    fi
fi

# 4. Claude Code
if [ -d "${HOME}/.claude" ]; then
    deploy_skill "${HOME}/.claude/skills/rg-md/SKILL.md"
fi

# 5. Windsurf / Codeium
if [ -d "${HOME}/.windsurf" ] || [ -d "${HOME}/.codeium" ] || [ -d "./.windsurf" ]; then
    if [ -d "./.windsurf" ]; then
        deploy_skill "./.windsurf/rules/rg-md.md"
    elif [ -d "${HOME}/.windsurf" ]; then
        deploy_skill "${HOME}/.windsurf/rules/rg-md.md"
    fi
fi

echo -e "\033[0;90m[3/3] Performing idempotent rule injection...\033[0m"

RULES_BLOCK="# >>> rg-md navigation rules >>>
## Codebase Navigation Directives
- Run rg-md \"<pattern>\" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- ‼ If locating identifiers, blind recursive grep without path scope is strictly prohibited.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
# <<< rg-md navigation rules <<<"

inject_rules() {
    local target_file="$1"
    local dir
    dir="$(dirname "${target_file}")"
    if [ ! -d "${dir}" ]; then
        return 0
    fi

    if [ -f "${target_file}" ]; then
        if grep -q "# >>> rg-md navigation rules >>>" "${target_file}"; then
            awk -v start="# >>> rg-md navigation rules >>>" \
                -v end="# <<< rg-md navigation rules <<<" \
                -v block="${RULES_BLOCK}" '
                BEGIN { printing = 1 }
                $0 ~ start { printing = 0; print block; next }
                $0 ~ end { printing = 1; next }
                printing { print }
            ' "${target_file}" > "${target_file}.tmp"
            cat "${target_file}.tmp" > "${target_file}"
            rm -f "${target_file}.tmp"
            echo -e "\033[0;32m      Updated rg-md rules in ${target_file}\033[0m"
        else
            printf "\n%s\n" "${RULES_BLOCK}" >> "${target_file}"
            echo -e "\033[0;32m      Injected rg-md rules into ${target_file}\033[0m"
        fi
    elif [ -d "${dir}" ]; then
        printf "%s\n" "${RULES_BLOCK}" > "${target_file}"
        echo -e "\033[0;32m      Created and injected rg-md rules into ${target_file}\033[0m"
    fi
}

# Inject into detected agent rule files
if [ -d "${HOME}/.gemini" ]; then
    inject_rules "${HOME}/.gemini/AGENTS.md"
fi

if [ -d "${HOME}/.agents" ]; then
    inject_rules "${HOME}/.agents/AGENTS.md"
fi

if [ -f "${HOME}/.cursorrules" ]; then
    inject_rules "${HOME}/.cursorrules"
fi
if [ -f "./.cursorrules" ]; then
    inject_rules "./.cursorrules"
fi

if [ -f "${HOME}/.claude/CLAUDE.md" ]; then
    inject_rules "${HOME}/.claude/CLAUDE.md"
elif [ -d "${HOME}/.claude" ]; then
    inject_rules "${HOME}/.claude/CLAUDE.md"
fi
if [ -f "./CLAUDE.md" ]; then
    inject_rules "./CLAUDE.md"
fi

if [ -f "${HOME}/.windsurfrules" ]; then
    inject_rules "${HOME}/.windsurfrules"
fi
if [ -f "./.windsurfrules" ]; then
    inject_rules "./.windsurfrules"
fi

# Inject into current project AGENTS.md if present (avoid duplicating if it's the rg-md repo itself which already has it)
if [ -f "./AGENTS.md" ]; then
    inject_rules "./AGENTS.md"
fi

echo -e "\033[0;32m==========================================================\033[0m"
echo -e "\033[0;32mInstallation and rule integration completed successfully!\033[0m"
echo -e "Try running: rg-md \"fn main\" ."