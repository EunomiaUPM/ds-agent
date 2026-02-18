# scripts/powershell/dev.ps1
param (
    [string]$Role = "provider",
    [string]$Cmd = "setup"
)

$ErrorActionPreference = "Stop"

# Use relative paths assuming this script is called from the project root or sub-module
# matching the behavior of the bash script which uses "../static"
# If running from 'rainbow-monolith', $PSScriptRoot is '.../rainbow/scripts/powershell'
# We want to resolve relative to the EXECUTION LOCATION just like bash usually does if relative paths are used inside.
# However, bash script defines: CONFIG_FILE="../static..."
# If run from `rainbow-monolith`, `..` is `rainbow`. Correct.

$ConfigFile = "..\static\environment\config\core.${Role}.yaml"
$EnvFile = "..\static\vault\${Role}\data\vault.env"

# Validations
if ($Role -notin "provider", "consumer") {
    Write-Error "[ERROR] Invalid role. Usage: dev.ps1 [provider|consumer] [setup|start]"
    exit 1
}

if ($Cmd -notin "setup", "start") {
    Write-Error "[ERROR] Invalid command. Usage: dev.ps1 [provider|consumer] [setup|start]"
    exit 1
}

if (-not (Test-Path $ConfigFile)) {
    Write-Error "[ERROR] Config file not found: $ConfigFile"
    exit 1
}

if (-not (Test-Path $EnvFile)) {
    Write-Error "[ERROR] Secrets file not found: $EnvFile"
    Write-Host "        Ensure Docker container is running and Vault is initialized."
    exit 1
}

# Execution
Write-Host "Running [$Cmd] for [$Role]..."

# Load Environment Variables from vault.env
# Format is VAR=VALUE. We need to handle comments and export them to process scope.
Get-Content $EnvFile | ForEach-Object {
    $line = $_.Trim()
    if ($line -and -not $line.StartsWith("#")) {
        $parts = $line -split "=", 2
        if ($parts.Count -eq 2) {
            $name = $parts[0].Trim()
            $value = $parts[1].Trim()
            # Remove potential quotes around value
            $value = $value -replace '^"|"$', ''
            
            # Set environment variable for the current process
            [System.Environment]::SetEnvironmentVariable($name, $value, [System.EnvironmentVariableTarget]::Process)
        }
    }
}

# Run Cargo
# We use cmd /c or direct execution. Direct execution is better in PS.
# Arguments: cargo run <cmd> -e <config_file>
Write-Host "Executing: cargo run -- $Cmd -e $ConfigFile"
cargo run -- $Cmd -e $ConfigFile
