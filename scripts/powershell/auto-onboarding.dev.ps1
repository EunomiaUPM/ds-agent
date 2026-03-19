# auto-onboarding.ps1
param(
    [string]$AuthorityUrl = "http://127.0.0.1:1500",
    [string]$ConsumerUrl  = "http://127.0.0.1:1100",
    [string]$ProviderUrl  = "http://127.0.0.1:1200",
    [string]$DockerAuthorityUrl = "http://127.0.0.1:1500",
    [string]$DockerConsumerUrl  = "http://127.0.0.1:1100",
    [string]$DockerProviderUrl  = "http://127.0.0.1:1200"
)

# ----------------------------
# Logging helpers
# ----------------------------

function Log-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host $Message -ForegroundColor Cyan
}

function Log-Success {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Green
}

function Log-Error {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Red
}

function Log-Info {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

# ----------------------------
# HTTP helper
# ----------------------------

function Invoke-CurlJson {
    param(
        [string]$Method = "GET",
        [string]$Url,
        [object]$Body = $null,
        [bool]$ParseJson = $true
    )

    try {

        $Params = @{
            Method      = $Method
            Uri         = $Url
            ContentType = "application/json"
            ErrorAction = 'Stop'
        }

        if ($Body) {
            $Params.Body = $Body | ConvertTo-Json -Compress
        }

        $Response = Invoke-WebRequest @Params

        if ($Response.StatusCode -ge 200 -and $Response.StatusCode -lt 300) {
            Log-Success "SUCCESS: $Method $Url -> $($Response.StatusCode)"
        } else {
            Log-Error "ERROR: $Method $Url -> $($Response.StatusCode)"
            exit 1
        }

        if ($ParseJson -and $Response.Content) {
            return $Response.Content | ConvertFrom-Json
        } else {
            return $Response.Content
        }

    } catch {
        Log-Error "ERROR: Request to $Url failed"
        Log-Error $_.Exception.Message
        Log-Error "The script won't continue executing"
        exit 1
    }
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "      AUTO ONBOARDING SCRIPT" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan


# ----------------------------
# STEP 1 - Link Authority Wallet
# ----------------------------

Log-Step "STEP 1 - Linking Authority wallet"
Invoke-CurlJson -Method "POST" -Url "$AuthorityUrl/api/v1/wallet/link" -ParseJson:$false


# ----------------------------
# STEP 2 - Link Consumer Wallet
# ----------------------------

Log-Step "STEP 2 - Linking Consumer wallet"
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/link" -ParseJson:$false


# ----------------------------
# STEP 3 - Link Provider Wallet
# ----------------------------

Log-Step "STEP 3 - Linking Provider wallet"
Invoke-CurlJson -Method "POST" -Url "$ProviderUrl/api/v1/wallet/link" -ParseJson:$false


# ----------------------------
# STEP 4 - Retrieve DIDs
# ----------------------------

Log-Step "STEP 4 - Retrieving DIDs"

$AUTH_DID = (Invoke-CurlJson -Url "$AuthorityUrl/.well-known/did.json").id
Log-Success "Authority DID: $AUTH_DID"

$CONSUMER_DID = (Invoke-CurlJson -Url "$ConsumerUrl/.well-known/did.json").id
Log-Success "Consumer DID: $CONSUMER_DID"

$PROVIDER_DID = (Invoke-CurlJson -Url "$ProviderUrl/.well-known/did.json").id
Log-Success "Provider DID: $PROVIDER_DID"


# ----------------------------
# STEP 5 - Consumer requests credential
# ----------------------------

Log-Step "STEP 5 - Consumer requests credential from Authority"

$C_BEG_BODY = @{
    url     = "$DockerAuthorityUrl/api/v1/gate/access"
    id      = $AUTH_DID
    slug    = "authority"
    vc_type = "DataspaceParticipant_jwt_vc_json"
    method  = "cert"
}

$C_BEG_RESPONSE = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/vc-request/beg" -Body $C_BEG_BODY -ParseJson:$false
Log-Success "Consumer credential request sent"


# ----------------------------
# STEP 6 - Authority retrieves requests
# ----------------------------

Log-Step "STEP 6 - Authority retrieving pending requests"

$ALL_REQUESTS = Invoke-CurlJson -Url "$AuthorityUrl/api/v1/approver/all"

$PETITION_ID = $ALL_REQUESTS[-1].id
Log-Info "Petition ID: $PETITION_ID"


# ----------------------------
# STEP 7 - Authority approves request
# ----------------------------

Log-Step "STEP 7 - Authority approving request"

$APPROVE_BODY = @{ approve = $true }

Invoke-CurlJson -Method "POST" -Url "$AuthorityUrl/api/v1/approver/$PETITION_ID" -Body $APPROVE_BODY -ParseJson:$false

Log-Success "Request approved"


# ----------------------------
# STEP 8 - Consumer retrieves credential URI
# ----------------------------

Log-Step "STEP 8 - Consumer retrieving OIDC4VCI URI"

$ALL_AUTHORITY = Invoke-CurlJson -Url "$ConsumerUrl/api/v1/vc-request/all"

$OIDC4VCI_URI = $ALL_AUTHORITY[-1].vc_uri

Log-Info "OIDC4VCI URI:"
Write-Host $OIDC4VCI_URI


# ----------------------------
# STEP 9 - Consumer processes OIDC4VCI
# ----------------------------

Log-Step "STEP 9 - Consumer processing credential"

Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vci" -Body @{ uri = $OIDC4VCI_URI } -ParseJson:$false

Log-Success "OIDC4VCI processed"


# ----------------------------
# STEP 10 - Consumer requests Provider access
# ----------------------------

Log-Step "STEP 10 - Consumer requesting Provider access"

$OIDC4VP_BODY = @{
    url     = "$DockerProviderUrl/api/v1/gate/access"
    id      = $PROVIDER_DID
    slug    = "provider"
    actions = @("talk")
}

$OIDC4VP_URI = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/onboard/provider" -Body $OIDC4VP_BODY -ParseJson:$false

Log-Info "OIDC4VP URI:"
Write-Host $OIDC4VP_URI


# ----------------------------
# STEP 11 - Consumer processes OIDC4VP
# ----------------------------

Log-Step "STEP 11 - Consumer processing OIDC4VP"

Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vp" -Body @{ uri = $OIDC4VP_URI } -ParseJson:$false

Log-Success "OIDC4VP processed"


Write-Host ""
Write-Host "======================================" -ForegroundColor Green
Write-Host "   ONBOARDING FINISHED SUCCESSFULLY" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""