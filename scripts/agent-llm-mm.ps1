param(
    [ValidateSet("serve", "doctor")]
    [string]$Mode = "serve"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path

Push-Location $projectRoot
try {
    & cargo run --quiet -- $Mode
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
