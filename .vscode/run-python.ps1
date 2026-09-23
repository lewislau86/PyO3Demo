param(
    [Parameter(Mandatory = $true)]
    [string]$PythonFile
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$python = Join-Path $projectRoot ".venv\Scripts\python.exe"

if (-not (Test-Path $python)) {
    throw "Project Python environment not found: $python"
}

Push-Location $projectRoot
$condaPrefix = $env:CONDA_PREFIX
try {
    $env:CONDA_PREFIX = $null

    & $python -m maturin develop
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    & $python $PythonFile
    exit $LASTEXITCODE
}
finally {
    if ($null -eq $condaPrefix) {
        Remove-Item Env:CONDA_PREFIX -ErrorAction SilentlyContinue
    } else {
        $env:CONDA_PREFIX = $condaPrefix
    }

    Pop-Location
}
