param(
    [string]$Mode = "serve",
    [string]$ConfigPath
)

$ErrorActionPreference = "Stop"

switch ($Mode) {
    "serve" { }
    "doctor" { }
    "bootstrap-local" { }
    default {
        [Console]::Error.WriteLine("unsupported mode: $Mode")
        [Console]::Error.WriteLine("usage: pwsh -File .\scripts\agent-llm-mm.ps1 [serve|doctor|bootstrap-local] [config_path]")
        exit 2
    }
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path

Push-Location $projectRoot
try {
    if ($Mode -eq "bootstrap-local") {
        $targetPath = if ($ConfigPath) { $ConfigPath } else { "agent-llm-mm.local.toml" }
        $sourcePath = Join-Path $projectRoot "examples/agent-llm-mm.dev.example.toml"
        $targetParent = Split-Path -Parent $targetPath
        if (-not $targetParent) {
            $targetParent = "."
        }

        if (Test-Path -LiteralPath $targetPath) {
            [Console]::Error.WriteLine("target already exists; refusing to overwrite: $targetPath")
            exit 1
        }
        if (-not (Test-Path -LiteralPath $targetParent -PathType Container)) {
            [Console]::Error.WriteLine("parent directory does not exist: $targetParent")
            exit 1
        }

        try {
            [System.IO.File]::Copy($sourcePath, $targetPath, $false)
        }
        catch [System.IO.IOException] {
            [Console]::Error.WriteLine("target already exists; refusing to overwrite: $targetPath")
            exit 1
        }
        $quotedTargetPath = "'" + ($targetPath -replace "'", "''") + "'"
        Write-Output "created local config: $targetPath"
        Write-Output "Next commands:"
        Write-Output "  pwsh -File .\scripts\agent-llm-mm.ps1 doctor $quotedTargetPath"
        Write-Output "  pwsh -File .\scripts\agent-llm-mm.ps1 serve $quotedTargetPath"
        exit 0
    }

    if ($ConfigPath) {
        $resolvedConfigPath = (Resolve-Path $ConfigPath).Path
        $env:AGENT_LLM_MM_CONFIG = $resolvedConfigPath
    }

    & cargo run --quiet --bin agent_llm_mm -- $Mode
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
