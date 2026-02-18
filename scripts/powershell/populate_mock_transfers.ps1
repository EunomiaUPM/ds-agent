# PowerShell Script: populate_mock_transfers.ps1

# ==========================================
# CONFIGURATION
# ==========================================
$AGENT_URL = "http://127.0.0.1:1200"

# ==========================================
# HELPER FUNCTIONS
# ==========================================

function Invoke-Curl {
    param (
        [string]$Method,
        [string]$Url,
        [string]$Body,
        [string]$Description
    )

    Write-Host "--- $Description ---"
    Write-Host "Request: $Method $Url"

    $Headers = @{ "Content-Type" = "application/json" }

    try {
        if (-not [string]::IsNullOrEmpty($Body)) {
            $Response = Invoke-RestMethod -Uri $Url -Method $Method -Body $Body -Headers $Headers -ErrorAction Stop
        } else {
            $Response = Invoke-RestMethod -Uri $Url -Method $Method -Headers $Headers -ErrorAction Stop
        }
        
        # Preview response
        $ResponseJson = $Response | ConvertTo-Json -Depth 10 -Compress
        Write-Host "Response (preview): $($ResponseJson.Substring(0, [math]::Min(100, $ResponseJson.Length)))..."
        Write-Host "---------------------------------------------------"
        
        return $Response
    } catch {
        Write-Host "Error: $_"
        return $null
    }
}

function Get-Timestamp {
    return [datetimeOffset]::Now.ToUnixTimeSeconds().ToString()
}

# ==========================================
# TRANSFER AGENT FUNCTIONS
# ==========================================

# 1. Create Transfer Process
function Create-Transfer-Process {
    param ($ConsumerDid, $AgreementId, $State)

    $Now = Get-Timestamp
    $Body = @{
        state = $State
        associatedAgentPeer = $ConsumerDid
        protocol = "dataspace-protocol-http"
        transferDirection = "OUTGOING"
        agreementId = $AgreementId
        role = "PROVIDER"
        identifiers = @{
            localId = "urn:uuid:$Now"
            remoteId = "urn:uuid:$Now-remote"
        }
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/transfer-agent/transfer-processes" -Body $Body -Description "Creating Transfer Process ($State)"
    return $Resp.id
}

# 2. Update Transfer Process State
function Update-Transfer-State {
    param ($ProcessId, $NewState)

    $Body = @{ state = $NewState } | ConvertTo-Json
    Invoke-Curl -Method "PUT" -Url "$AGENT_URL/api/v1/transfer-agent/transfer-processes/$ProcessId" -Body $Body -Description "Updating Transfer State -> $NewState" | Out-Null
}

# 3. Create Transfer Message
function Create-Transfer-Message {
    param ($ProcessId, $Type, $FromState, $ToState)

    $Direction = "OUTGOING"
    if ($Type -eq "TRANSFER_REQUEST") {
        $Direction = "INCOMING"
    }

    $Body = @{
        transferAgentProcessId = $ProcessId
        direction = $Direction
        protocol = "dataspace-protocol-http"
        messageType = $Type
        stateTransitionFrom = $FromState
        stateTransitionTo = $ToState
        payload = @{
            dataAddress = @{ properties = @{ endpoint = "http://dummy/data" } }
        }
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/transfer-agent/transfer-messages" -Body $Body -Description "Msg: $Type ($FromState -> $ToState)"
    return $Resp.id
}


# ==========================================
# SIMULATION SCENARIOS
# ==========================================

# SCENARIO A: SUCCESSFUL
function Simulate-Successful-Transfer {
    param ($Consumer, $Agreement, $Label)

    Write-Host ">>> Starting Transfer Scenario: SUCCESSFUL ($Label)"

    # 1. REQUESTED (Consumer asks for data)
    $PID_PROC = Create-Transfer-Process -ConsumerDid $Consumer -AgreementId $Agreement -State "REQUESTED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_REQUEST" -FromState "INITIAL" -ToState "REQUESTED" | Out-Null

    # 2. STARTED (Provider starts sending data)
    Update-Transfer-State -ProcessId $PID_PROC -NewState "STARTED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_START" -FromState "REQUESTED" -ToState "STARTED" | Out-Null

    # 3. COMPLETED (Data transfer finished)
    Update-Transfer-State -ProcessId $PID_PROC -NewState "COMPLETED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_COMPLETION" -FromState "STARTED" -ToState "COMPLETED" | Out-Null

    Write-Host ">>> Finished Transfer: SUCCESSFUL (Process: $PID_PROC)`n"
}

# SCENARIO B: TERMINATED
function Simulate-Terminated-Transfer {
    param ($Consumer, $Agreement, $Label)

    Write-Host ">>> Starting Transfer Scenario: TERMINATED ($Label)"

    # 1. REQUESTED
    $PID_PROC = Create-Transfer-Process -ConsumerDid $Consumer -AgreementId $Agreement -State "REQUESTED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_REQUEST" -FromState "INITIAL" -ToState "REQUESTED" | Out-Null

    # 2. STARTED
    Update-Transfer-State -ProcessId $PID_PROC -NewState "STARTED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_START" -FromState "REQUESTED" -ToState "STARTED" | Out-Null

    # 3. TERMINATED (Something went wrong)
    Update-Transfer-State -ProcessId $PID_PROC -NewState "TERMINATED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_TERMINATION" -FromState "STARTED" -ToState "TERMINATED" | Out-Null

    Write-Host ">>> Finished Transfer: TERMINATED (Process: $PID_PROC)`n"
}

# SCENARIO C: SUSPENDED
function Simulate-Suspended-Transfer {
    param ($Consumer, $Agreement, $Label)

    Write-Host ">>> Starting Transfer Scenario: SUSPENDED ($Label)"

    $PID_PROC = Create-Transfer-Process -ConsumerDid $Consumer -AgreementId $Agreement -State "STARTED"
    Create-Transfer-Message -ProcessId $PID_PROC -Type "TRANSFER_START" -FromState "REQUESTED" -ToState "STARTED" | Out-Null

    Update-Transfer-State -ProcessId $PID_PROC -NewState "SUSPENDED"

    Write-Host ">>> Finished Transfer: SUSPENDED (Process: $PID_PROC)`n"
}

# ==========================================
# MAIN EXECUTION
# ==========================================

function Main {
    param ($AgreementId)

    Write-Host "=== GENERATING MOCK TRANSFER DATA ==="

    # --- MOCK DATA ---
    $CONSUMER_A = "did:web:consumer:company-a"
    if (-not $AgreementId) {
        $AgreementId = "urn:agreement:mock-agreement-001"
    }

    Write-Host "Using Agreement ID: $AgreementId"

    # --- EXECUTE SCENARIOS ---

    # 1. Successful Transfer
    Simulate-Successful-Transfer -Consumer $CONSUMER_A -Agreement $AgreementId -Label "Daily Batch Download"

    # 2. Failed Transfer
    Simulate-Terminated-Transfer -Consumer $CONSUMER_A -Agreement $AgreementId -Label "Real-time Stream (Failed)"

    # 3. Suspended Transfer
    Simulate-Suspended-Transfer -Consumer $CONSUMER_A -Agreement $AgreementId -Label "Large File Upload (Paused)"

    Write-Host "=== DONE. GENERATED TRANSFER PROCESSES ==="
}

# Use arguments if provided
Main -AgreementId $args[0]
