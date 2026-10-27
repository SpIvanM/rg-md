# Install script for rg-md on Windows
# Implements idempotent marker replacement and agent environment detection.
# Usage:
#   irm https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "SpIvanM/rg-md"
$installDir = "$env:LOCALAPPDATA\Programs\rg-md\bin"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   rg-md — Markdown-Formatted Code Search Engine          " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path -ErrorAction SilentlyContinue
$repoRoot = $null
if ($scriptDir -and (Test-Path "$scriptDir\..\Cargo.toml")) {
    $repoRoot = (Resolve-Path "$scriptDir\..").Path
} elseif (Test-Path ".\Cargo.toml") {
    $repoRoot = (Resolve-Path ".").Path
}

Write-Host "[1/3] Installing rg-md binary..." -ForegroundColor Gray
$installed = $false

# Priority A: Local build
if ($repoRoot) {
    if (Test-Path "$repoRoot\target\release\rg-md.exe") {
        Copy-Item "$repoRoot\target\release\rg-md.exe" "$installDir\rg-md.exe" -Force
        Copy-Item "$repoRoot\target\release\rg-md.exe" "$installDir\md-rg.exe" -Force
        $installed = $true
        Write-Host "      Installed from local target\release\rg-md.exe" -ForegroundColor Green
    } elseif (Test-Path "$repoRoot\target\debug\rg-md.exe") {
        Copy-Item "$repoRoot\target\debug\rg-md.exe" "$installDir\rg-md.exe" -Force
        Copy-Item "$repoRoot\target\debug\rg-md.exe" "$installDir\md-rg.exe" -Force
        $installed = $true
        Write-Host "      Installed from local target\debug\rg-md.exe" -ForegroundColor Green
    }
}

# Priority B: Download release archive
if (-not $installed) {
    $zipUrl = "https://github.com/$repo/releases/latest/download/rg-md-v0.1.0-x86_64-pc-windows-msvc.zip"
    $tmpZip = "$env:TEMP\rg-md.zip"
    $tmpExtract = "$env:TEMP\rg-md-extract"
    try {
        Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing -ErrorAction Stop
        Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
        Copy-Item "$tmpExtract\rg-md.exe" "$installDir\rg-md.exe" -Force
        Copy-Item "$tmpExtract\rg-md.exe" "$installDir\md-rg.exe" -Force
        Remove-Item -Recurse -Force $tmpExtract, $tmpZip -ErrorAction SilentlyContinue
        $installed = $true
        Write-Host "      Downloaded and installed release binary." -ForegroundColor Green
    } catch {
        Remove-Item -Recurse -Force $tmpExtract, $tmpZip -ErrorAction SilentlyContinue
    }
}

# Priority C: Cargo install
if (-not $installed) {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        if ($repoRoot -and (Test-Path "$repoRoot\Cargo.toml")) {
            Write-Host "      Building release binary via cargo..." -ForegroundColor Gray
            cargo build --release --manifest-path "$repoRoot\Cargo.toml"
            Copy-Item "$repoRoot\target\release\rg-md.exe" "$installDir\rg-md.exe" -Force
            Copy-Item "$repoRoot\target\release\rg-md.exe" "$installDir\md-rg.exe" -Force
            $installed = $true
        } else {
            Write-Host "      Installing from remote repository via cargo..." -ForegroundColor Gray
            cargo install --git "https://github.com/$repo.git"
            $cargoBin = "$env:USERPROFILE\.cargo\bin\rg-md.exe"
            if (Test-Path $cargoBin) {
                Copy-Item $cargoBin "$installDir\rg-md.exe" -Force
                Copy-Item $cargoBin "$installDir\md-rg.exe" -Force
                $installed = $true
            }
        }
        Write-Host "      Built and installed via cargo." -ForegroundColor Green
    }
}

if (-not $installed) {
    Write-Error "Failed to install rg-md binary. Please install Rust (https://rustup.rs) or download prebuilt binary."
}

# Update User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "      Added $installDir to User PATH." -ForegroundColor Green
} else {
    Write-Host "      $installDir is already in PATH." -ForegroundColor Gray
}

Write-Host "[2/3] Detecting agent environments and deploying skills..." -ForegroundColor Gray

$skillContent = $null
if ($repoRoot -and (Test-Path "$repoRoot\skills\rg-md\SKILL.md")) {
    $skillContent = Get-Content "$repoRoot\skills\rg-md\SKILL.md" -Raw
} else {
    try {
        $skillContent = (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/skills/rg-md/SKILL.md" -UseBasicParsing).Content
    } catch {}
}

function Deploy-Skill($targetPath) {
    $parentDir = Split-Path -Parent $targetPath
    if (-not (Test-Path $parentDir)) {
        New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
    }
    if ($skillContent) {
        Set-Content -Path $targetPath -Value $skillContent -Encoding UTF8 -NoNewline
        Write-Host "      Deployed skill: $targetPath" -ForegroundColor Green
    }
}

# 1. Google Antigravity / Gemini
$geminiDir = "$env:USERPROFILE\.gemini"
if (Test-Path $geminiDir) {
    Deploy-Skill "$geminiDir\config\skills\rg-md\SKILL.md"
}

# 2. Generic Agents
$agentsDir = "$env:USERPROFILE\.agents"
if (Test-Path $agentsDir) {
    Deploy-Skill "$agentsDir\skills\rg-md\SKILL.md"
}
if (Test-Path ".\.agents") {
    Deploy-Skill ".\.agents\skills\rg-md\SKILL.md"
}

# 3. Cursor
if ((Test-Path "$env:USERPROFILE\.cursor") -or (Test-Path ".\.cursor") -or (Test-Path ".\.cursorrules")) {
    if (Test-Path ".\.cursor") {
        Deploy-Skill ".\.cursor\rules\rg-md.mdc"
    } elseif (Test-Path "$env:USERPROFILE\.cursor") {
        Deploy-Skill "$env:USERPROFILE\.cursor\rules\rg-md.mdc"
    }
}

# 4. Claude Code
if (Test-Path "$env:USERPROFILE\.claude") {
    Deploy-Skill "$env:USERPROFILE\.claude\skills\rg-md\SKILL.md"
}

# 5. Windsurf / Codeium
if ((Test-Path "$env:USERPROFILE\.windsurf") -or (Test-Path "$env:USERPROFILE\.codeium") -or (Test-Path ".\.windsurf")) {
    if (Test-Path ".\.windsurf") {
        Deploy-Skill ".\.windsurf\rules\rg-md.md"
    } elseif (Test-Path "$env:USERPROFILE\.windsurf") {
        Deploy-Skill "$env:USERPROFILE\.windsurf\rules\rg-md.md"
    }
}

Write-Host "[3/3] Performing idempotent rule injection..." -ForegroundColor Gray

$rulesBlock = @"
# >>> rg-md navigation rules >>>
## Codebase Navigation Directives
- Run rg-md "<pattern>" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- ‼ If locating identifiers, blind recursive grep without path scope is strictly prohibited.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
# <<< rg-md navigation rules <<<
"@

function Inject-Rules($targetFile) {
    $parentDir = Split-Path -Parent $targetFile
    if (-not (Test-Path $parentDir)) {
        return
    }

    $startMarker = "# >>> rg-md navigation rules >>>"
    $endMarker = "# <<< rg-md navigation rules <<<"

    if (Test-Path $targetFile) {
        $content = Get-Content $targetFile -Raw
        if ($content -match "(?s)$startMarker.*?$endMarker") {
            $newContent = [regex]::Replace($content, "(?s)$startMarker.*?$endMarker", $rulesBlock.Trim())
            Set-Content -Path $targetFile -Value $newContent -Encoding UTF8 -NoNewline
            Write-Host "      Updated rg-md rules in $targetFile" -ForegroundColor Green
        } else {
            $newContent = $content.TrimEnd() + "`r`n`r`n" + $rulesBlock.Trim() + "`r`n"
            Set-Content -Path $targetFile -Value $newContent -Encoding UTF8 -NoNewline
            Write-Host "      Injected rg-md rules into $targetFile" -ForegroundColor Green
        }
    } else {
        Set-Content -Path $targetFile -Value ($rulesBlock.Trim() + "`r`n") -Encoding UTF8 -NoNewline
        Write-Host "      Created and injected rg-md rules into $targetFile" -ForegroundColor Green
    }
}

# Rule injection targets
if (Test-Path $geminiDir) {
    Inject-Rules "$geminiDir\AGENTS.md"
}
if (Test-Path $agentsDir) {
    Inject-Rules "$agentsDir\AGENTS.md"
}
if (Test-Path "$env:USERPROFILE\.cursorrules") {
    Inject-Rules "$env:USERPROFILE\.cursorrules"
}
if (Test-Path ".\.cursorrules") {
    Inject-Rules ".\.cursorrules"
}
if (Test-Path "$env:USERPROFILE\.claude") {
    Inject-Rules "$env:USERPROFILE\.claude\CLAUDE.md"
}
if (Test-Path ".\CLAUDE.md") {
    Inject-Rules ".\CLAUDE.md"
}
if (Test-Path "$env:USERPROFILE\.windsurfrules") {
    Inject-Rules "$env:USERPROFILE\.windsurfrules"
}
if (Test-Path ".\.windsurfrules") {
    Inject-Rules ".\.windsurfrules"
}
if (Test-Path ".\AGENTS.md") {
    Inject-Rules ".\AGENTS.md"
}

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "Installation and rule integration completed successfully! " -ForegroundColor Green
Write-Host "Try running: rg-md `"fn main`" ."
