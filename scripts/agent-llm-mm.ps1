param(
    [ValidateSet("serve", "doctor")]
    [string]$Mode = "serve",
    [string]$ConfigPath
)

$ErrorActionPreference = "Stop"

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
