param(
    [string]$Mode = "serve",
    [string]$ConfigPath
)

$ErrorActionPreference = "Stop"

switch ($Mode) {
    "serve" { }
    "doctor" { }
    default {
        [Console]::Error.WriteLine("unsupported mode: $Mode")
        [Console]::Error.WriteLine("usage: pwsh -File .\scripts\agent-llm-mm.ps1 [serve|doctor] [config_path]")
        exit 2
    }
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path

Push-Location $projectRoot
try {
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
