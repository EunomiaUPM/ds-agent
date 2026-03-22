# start-dev.ps1
$ErrorActionPreference = "Stop"

$scriptDir = $PSScriptRoot
Set-Location $scriptDir

Write-Host "=== Eunomia Dev Environment Setup ===" -ForegroundColor Cyan

# 1. Prepare scripts (CRLF -> LF)
Write-Host "[1/4] Preparing Docker scripts..." -ForegroundColor Yellow
try {
    ./prepare-docker-scripts.ps1
}
catch {
    Write-Error "Failed to prepare scripts: $_"
    exit 1
}

$composeFile = "docker-compose.dev.ps1.yaml"
if (-not (Test-Path $composeFile)) {
    Write-Error "Compose file '$composeFile' not found!"
    exit 1
}

# Define command wrapper
function Run-Compose {
    param([string[]]$Arguments)
    if (Get-Command "docker-compose" -ErrorAction SilentlyContinue) {
        docker-compose -f $composeFile @Arguments
    }
    elseif (Get-Command "docker" -ErrorAction SilentlyContinue) {
        docker compose -f $composeFile @Arguments
    }
    else {
        Write-Error "Docker not found."
        exit 1
    }
}

# 2. Start Docker Compose in detached mode
Write-Host "[2/4] Starting containers (Detached)..." -ForegroundColor Yellow
Run-Compose -Arguments @("up", "-d", "--build", "--remove-orphans")

# 3. Wait for Vault Setup
Write-Host "[3/4] Waiting for Vault configuration..." -ForegroundColor Yellow
$maxRetries = 120 # 2 minutes
$retryCount = 0

Write-Host "Waiting up to 2 minutes for 'heimdall_vault_setup' to finish..." -ForegroundColor Gray

while ($retryCount -lt $maxRetries) {
    $status = docker inspect -f '{{.State.Status}}' heimdall_vault_setup 2>$null
    
    if ($status -eq "exited") {
        Write-Host "`nSetup complete!" -ForegroundColor Green
        # Wait a moment for file sync
        Start-Sleep -Seconds 2
        
        Write-Host "Restarting Heimdall to pick up new token..." -ForegroundColor Green
        Run-Compose -Arguments @("restart", "heimdall")
        break
    }
    
    Write-Host "." -NoNewline
    Start-Sleep -Seconds 1
    $retryCount++
}

if ($retryCount -eq $maxRetries) {
    Write-Warning "`nSetup timed out. 'heimdall' might fail to connect."
}

# 4. Attach logs
Write-Host "`n[4/4] Attaching logs (Ctrl+C to stop)..." -ForegroundColor Yellow
Run-Compose -Arguments @("logs", "-f")
