# PowerShell Script: populate_mock_contracts.ps1

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
# API WRAPPERS (ATOMIC OPERATIONS)
# ==========================================

# 1. Create Process
function Create-Process {
    param ($ConsumerDid, $State)

    $Now = Get-Timestamp
    $Body = @{
        state = $State
        associatedAgentPeer = $ConsumerDid
        protocol = "dataspace-protocol-http"
        role = "PROVIDER"
        identifiers = @{
            localId = "urn:uuid:$Now"
            remoteId = "urn:uuid:$Now-remote"
        }
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/negotiation-agent/negotiation-processes" -Body $Body -Description "Creating Process ($State)"
    return $Resp.id
}

# 2. Update Process State
function Update-Process-State {
    param ($ProcessId, $NewState)

    $Body = @{ state = $NewState } | ConvertTo-Json
    Invoke-Curl -Method "PUT" -Url "$AGENT_URL/api/v1/negotiation-agent/negotiation-processes/$ProcessId" -Body $Body -Description "Updating Process State -> $NewState" | Out-Null
}

# 3. Create Message
function Create-Message {
    param ($ProcessId, $Type, $FromState, $ToState)

    $Direction = "OUTGOING"
    if ($Type -eq "CONTRACT_REQUEST" -or $Type -eq "CONTRACT_AGREEMENT_VERIFICATION") {
        $Direction = "INCOMING"
    }

    $Now = Get-Timestamp
    $Body = @{
        negotiationAgentProcessId = $ProcessId
        direction = $Direction
        protocol = "dataspace-protocol-http"
        messageType = $Type
        stateTransitionFrom = $FromState
        stateTransitionTo = $ToState
        payload = @{ mock_timestamp = $Now }
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/negotiation-agent/negotiation-messages" -Body $Body -Description "Msg: $Type ($FromState -> $ToState)"
    return $Resp.id
}

# 4. Create Offer Object
function Create-Offer {
    param ($ProcessId, $MessageId, $TargetId)

    $OfferUid = "urn:offer:$([datetimeOffset]::Now.ToUnixTimeMilliseconds())" # Using milliseconds for better uniqueness like date +%s%N
    
    $Body = @{
        negotiationAgentProcessId = $ProcessId
        negotiationAgentMessageId = $MessageId
        offerId = $OfferUid
        offerContent = @{
            uid = $OfferUid
            target = $TargetId
            assigner = "did:web:provider"
            permission = @(
                @{ action = "use"; target = $TargetId }
            )
        }
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/negotiation-agent/offers" -Body $Body -Description "Creating Offer Object"
    return $Resp.id
}

# 5. Create Agreement Object
function Create-Agreement {
    param ($ProcessId, $MessageId, $Consumer, $Provider, $Target)

    $Now = Get-Timestamp
    $Body = @{
        negotiationAgentProcessId = $ProcessId
        negotiationAgentMessageId = $MessageId
        consumerParticipantId = $Consumer
        providerParticipantId = $Provider
        target = $Target
        agreementContent = @{
            uid = "urn:agreement:$Now"
            target = $Target
            permission = @(
                @{ action = "use"; target = $Target }
            )
        }
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/negotiation-agent/agreements" -Body $Body -Description "Creating Agreement Object"
    return $Resp.id
}


# ==========================================
# SCENARIO SIMULATIONS
# ==========================================

# SCENARIO A: SUCCESSFUL
function Simulate-Successful-Negotiation {
    param ($Consumer, $Dataset, $Label)

    Write-Host ">>> Starting Scenario: SUCCESSFUL ($Label)"

    # 1. State: REQUESTED
    $PID_PROC = Create-Process -ConsumerDid $Consumer -State "REQUESTED"
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_REQUEST" -FromState "INITIAL" -ToState "REQUESTED" | Out-Null

    # 2. State: OFFERED
    Update-Process-State -ProcessId $PID_PROC -NewState "OFFERED"
    $MID_OFFER = Create-Message -ProcessId $PID_PROC -Type "CONTRACT_OFFER" -FromState "REQUESTED" -ToState "OFFERED"
    Create-Offer -ProcessId $PID_PROC -MessageId $MID_OFFER -TargetId $Dataset | Out-Null

    # 3. State: ACCEPTED
    Update-Process-State -ProcessId $PID_PROC -NewState "ACCEPTED"
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_NEGOTIATION_EVENT" -FromState "OFFERED" -ToState "ACCEPTED" | Out-Null

    # 4. State: AGREED
    Update-Process-State -ProcessId $PID_PROC -NewState "AGREED"
    $MID_AGREE = Create-Message -ProcessId $PID_PROC -Type "CONTRACT_AGREEMENT" -FromState "ACCEPTED" -ToState "AGREED"
    # Mock provider DID
    Create-Agreement -ProcessId $PID_PROC -MessageId $MID_AGREE -Consumer $Consumer -Provider "did:web:provider:mock" -Target $Dataset | Out-Null

    # 5. State: FINALIZED
    Update-Process-State -ProcessId $PID_PROC -NewState "FINALIZED"
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_NEGOTIATION_EVENT" -FromState "AGREED" -ToState "FINALIZED" | Out-Null

    Write-Host ">>> Finished Scenario: SUCCESSFUL (Process: $PID_PROC)`n"
}

# SCENARIO B: TERMINATED
function Simulate-Terminated-Negotiation {
    param ($Consumer, $Dataset, $Label)

    Write-Host ">>> Starting Scenario: TERMINATED ($Label)"

    # 1. State: REQUESTED
    $PID_PROC = Create-Process -ConsumerDid $Consumer -State "REQUESTED"
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_REQUEST" -FromState "INITIAL" -ToState "REQUESTED" | Out-Null

    # 2. State: OFFERED
    Update-Process-State -ProcessId $PID_PROC -NewState "OFFERED"
    $MID_OFFER = Create-Message -ProcessId $PID_PROC -Type "CONTRACT_OFFER" -FromState "REQUESTED" -ToState "OFFERED"
    Create-Offer -ProcessId $PID_PROC -MessageId $MID_OFFER -TargetId $Dataset | Out-Null

    # 3. State: TERMINATED
    Update-Process-State -ProcessId $PID_PROC -NewState "TERMINATED"
    # Event Message for termination
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_NEGOTIATION_EVENT" -FromState "OFFERED" -ToState "TERMINATED" | Out-Null

    Write-Host ">>> Finished Scenario: TERMINATED (Process: $PID_PROC)`n"
}

# SCENARIO C: PENDING
function Simulate-Pending-Negotiation {
    param ($Consumer, $Dataset, $Label)

    Write-Host ">>> Starting Scenario: PENDING ($Label)"

    # 1. State: REQUESTED
    $PID_PROC = Create-Process -ConsumerDid $Consumer -State "REQUESTED"
    Create-Message -ProcessId $PID_PROC -Type "CONTRACT_REQUEST" -FromState "INITIAL" -ToState "REQUESTED" | Out-Null

    # 2. State: OFFERED
    Update-Process-State -ProcessId $PID_PROC -NewState "OFFERED"
    $MID_OFFER = Create-Message -ProcessId $PID_PROC -Type "CONTRACT_OFFER" -FromState "REQUESTED" -ToState "OFFERED"
    Create-Offer -ProcessId $PID_PROC -MessageId $MID_OFFER -TargetId $Dataset | Out-Null

    Write-Host ">>> Finished Scenario: PENDING (Process: $PID_PROC)`n"
}

# ==========================================
# MAIN
# ==========================================

function Main {
    Write-Host "=== GENERATING MOCK NEGOTIATION DATA ==="

    # MOCK DATASETS & PARTICIPANTS
    $CONSUMER_A = "did:web:consumer:company-a"
    $CONSUMER_B = "did:web:consumer:company-b"
    $DS_WEATHER = "urn:dataset:weather-2024"
    $DS_TRAFFIC = "urn:dataset:traffic-logs"
    $DS_SENSITIVE = "urn:dataset:sensitive-financials"

    # --- BATCH 1: Successful Agreements ---
    Simulate-Successful-Negotiation -Consumer $CONSUMER_A -Dataset $DS_WEATHER -Label "Weather Data Purchase - Completed"
    Simulate-Successful-Negotiation -Consumer $CONSUMER_B -Dataset $DS_TRAFFIC -Label "Traffic Data Access - Completed"
    Simulate-Successful-Negotiation -Consumer $CONSUMER_A -Dataset $DS_TRAFFIC -Label "Traffic Data Access (Repeated) - Completed"

    # --- BATCH 2: Failed / Terminated ---
    Simulate-Terminated-Negotiation -Consumer $CONSUMER_B -Dataset $DS_SENSITIVE -Label "Sensitive Data Access - Rejected by User"

    # --- BATCH 3: Ongoing / Pending ---
    Simulate-Pending-Negotiation -Consumer $CONSUMER_A -Dataset $DS_SENSITIVE -Label "Sensitive Data Access - Waiting for Approval"

    Write-Host "=== DONE. GENERATED 5 NEGOTIATION PROCESSES ==="
}

Main
