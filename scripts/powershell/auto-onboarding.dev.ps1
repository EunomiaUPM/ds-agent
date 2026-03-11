# auto-onboarding.ps1
param(
    [string]$AuthorityUrl = "http://127.0.0.1:1500",
    [string]$ConsumerUrl  = "http://127.0.0.1:1100",
    [string]$ProviderUrl  = "http://127.0.0.1:1200",
    [string]$DockerAuthorityUrl = "http://127.0.0.1:1500",
    [string]$DockerConsumerUrl  = "http://127.0.0.1:1100",
    [string]$DockerProviderUrl  = "http://127.0.0.1:1200"
)

function Invoke-CurlJson {
    param(
        [string]$Method = "GET",
        [string]$Url,
        [object]$Body = $null,
        [bool]$ParseJson = $true  # parse to JSON only if JSON is expected
    )

    try {
        $Params = @{
            Method      = $Method
            Uri         = $Url
            ContentType = "application/json"
            ErrorAction = 'Stop'
        }

        if ($Body) { $Params.Body = $Body | ConvertTo-Json -Compress }

        $Response = Invoke-WebRequest @Params

        if ($Response.StatusCode -ge 200 -and $Response.StatusCode -lt 300) {
            Write-Host "SUCCESS: $Method $Url returned $($Response.StatusCode)"
        } else {
            Write-Host "ERROR: $Method $Url returned $($Response.StatusCode)"
            exit 1
        }

        if ($ParseJson -and $Response.Content) {
            return $Response.Content | ConvertFrom-Json
        } else {
            return $Response.Content
        }

    } catch {
        Write-Host "ERROR: Request to $Url failed"
        Write-Host $_.Exception.Message
        Write-Host "The script wont continue executing"
        Write-Host ""
        exit 1
    }
}

Write-Host "Starting auto-onboarding script..."

# ----------------------------
# Onboarding Authority / Consumer / Provider (no JSON parsing)
# ----------------------------
Invoke-CurlJson -Method "POST" -Url "$AuthorityUrl/api/v1/wallet/link" -ParseJson:$false
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/link" -ParseJson:$false
Invoke-CurlJson -Method "POST" -Url "$ProviderUrl/api/v1/wallet/link" -ParseJson:$false

# ----------------------------
# Getting DIDs
# ----------------------------
$AUTH_DID     = (Invoke-CurlJson -Url "$AuthorityUrl/.well-known/did.json").id
Write-Host "Authority DID: $AUTH_DID"
$CONSUMER_DID = (Invoke-CurlJson -Url "$ConsumerUrl/.well-known/did.json").id
Write-Host "Consumer DID: $CONSUMER_DID"
$PROVIDER_DID = (Invoke-CurlJson -Url "$ProviderUrl/.well-known/did.json").id
Write-Host "Provider DID: $PROVIDER_DID"

# ----------------------------
# Consumer begins request for credential
# ----------------------------
$C_BEG_BODY = @{
    url = "$DockerAuthorityUrl/api/v1/gate/access"
    id  = $AUTH_DID
    slug = "authority"
    vc_type = "DataspaceParticipant_jwt_vc_json"
    method = "cross-user"
}
$C_BEG_RESPONSE = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/vc-request/beg" -Body $C_BEG_BODY -ParseJson:$false
Write-Host "Consumer request completed."

# ----------------------------
# Get all requests from Authority
# ----------------------------
$ALL_REQUESTS = Invoke-CurlJson -Url "$AuthorityUrl/api/v1/approver/all"
$PETITION_ID = $ALL_REQUESTS[-1].id
Write-Host "Petition ID: $PETITION_ID"

# ----------------------------
# Authority approves request
# ----------------------------
$APPROVE_BODY = @{ approve = $true }
Invoke-CurlJson -Method "POST" -Url "$AuthorityUrl/api/v1/approver/$PETITION_ID" -Body $APPROVE_BODY -ParseJson:$false
Write-Host "Request approved."

# ----------------------------
# Get all authority requests for Consumer
# ----------------------------
$ALL_AUTHORITY = Invoke-CurlJson -Url "$ConsumerUrl/api/v1/vc-request/all"
$OIDC4VCI_URI = $ALL_AUTHORITY[-1].vc_uri
Write-Host "OIDC4VCI_URI: $OIDC4VCI_URI"

# ----------------------------
# Consumer processes OIDC4VCI
# ----------------------------
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vci" -Body @{ uri = $OIDC4VCI_URI } -ParseJson:$false
Write-Host "OIDC4VCI processed."

# ----------------------------
# Consumer requests grant from Provider
# ----------------------------
$OIDC4VP_BODY = @{
    url = "$DockerProviderUrl/api/v1/gate/access"
    id  = $PROVIDER_DID
    slug = "provider"
    actions = "talk"
}
$OIDC4VP_URI = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/onboard/provider" -Body $OIDC4VP_BODY -ParseJson:$false
Write-Host "OIDC4VP_URI: $OIDC4VP_URI"

# ----------------------------
# Consumer processes OIDC4VP
# ----------------------------
Write-Host "Consumer processes OIDC4VP..."
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/oidc4vp" -Body @{ uri = $OIDC4VP_URI } -ReturnJson:$false
Write-Host "OIDC4VP processed."

Write-Host "Onboarding script finished successfully!"
Write-Host ""