# auto-onboarding.ps1
param(
    [string]$AuthorityUrl = "https://dev-dataspaces.dit.upm.es",
    [string]$ConsumerUrl  = "https://eunomia-consumer.dit.upm.es",
    [string]$ProviderUrl  = "https://eunomia-provider.dit.upm.es",
    [string]$DockerAuthorityUrl = "https://dev-dataspaces.dit.upm.es",
    [string]$DockerConsumerUrl  = "https://eunomia-consumer.dit.upm.es",
    [string]$DockerProviderUrl  = "https://eunomia-provider.dit.upm.es"
)

# ----------------------------
# Utility functions
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

function Wait-ForUser {
    param([string]$Message)

    $input = Read-Host "$Message (Enter = continuar | n = parar)"

    if ($input -eq "n") {
        Write-Host "Script detenido por el usuario." -ForegroundColor Yellow
        exit 0
    }
}

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
            Log-Success "SUCCESS: $Method $Url returned $($Response.StatusCode)"
        } else {
            Log-Error "ERROR: $Method $Url returned $($Response.StatusCode)"
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
        Log-Error "The script wont continue executing"
        exit 1
    }
}

Write-Host ""
Write-Host "=== AUTO ONBOARDING SCRIPT ===" -ForegroundColor Cyan

# ----------------------------
# STEP 1 - Authority manual check
# ----------------------------
Log-Step "STEP 1 - Verify Authority wallet is linked"
Wait-ForUser "Comprueba manualmente que la Authority wallet está linkeada."

# ----------------------------
# STEP 2 - Link Consumer wallet
# ----------------------------
Log-Step "STEP 2 - Linking Consumer wallet"
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/link" -ParseJson:$false

# ----------------------------
# STEP 3 - Link Provider wallet
# ----------------------------
Log-Step "STEP 3 - Linking Provider wallet"
Invoke-CurlJson -Method "POST" -Url "$ProviderUrl/api/v1/wallet/link" -ParseJson:$false

# ----------------------------
# STEP 4 - Getting DIDs
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

Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/vc-request/beg" -Body $C_BEG_BODY -ParseJson:$false

Wait-ForUser "Aprueba manualmente la solicitud en Authority antes de continuar."

# ----------------------------
# STEP 6 - Consumer obtains OIDC4VCI URI
# ----------------------------
Log-Step "STEP 6 - Fetching credential URI"

$ALL_AUTHORITY = Invoke-CurlJson -Url "$ConsumerUrl/api/v1/vc-request/all"
$OIDC4VCI_URI = $ALL_AUTHORITY[-1].vc_uri

Log-Success "OIDC4VCI_URI: $OIDC4VCI_URI"

# ----------------------------
# STEP 7 - Consumer processes OIDC4VCI
# ----------------------------
Log-Step "STEP 7 - Consumer processing credential"

Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vci" -Body @{ uri = $OIDC4VCI_URI } -ParseJson:$false

# ----------------------------
# STEP 8 - Consumer requests Provider access
# ----------------------------
Log-Step "STEP 8 - Consumer requesting Provider access"

$OIDC4VP_BODY = @{
    url     = "$DockerProviderUrl/api/v1/gate/access"
    id      = $PROVIDER_DID
    slug    = "provider"
    actions = @("talk")
}

$OIDC4VP_URI = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/onboard/provider" -Body $OIDC4VP_BODY -ParseJson:$false

Log-Success "OIDC4VP_URI: $OIDC4VP_URI"

# ----------------------------
# STEP 9 - Consumer processes OIDC4VP
# ----------------------------
Log-Step "STEP 9 - Consumer processing provider verification"

Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vp" -Body @{ uri = $OIDC4VP_URI } -ParseJson:$false

Write-Host ""
Log-Success "=== ONBOARDING FINISHED SUCCESSFULLY ==="
Write-Host ""