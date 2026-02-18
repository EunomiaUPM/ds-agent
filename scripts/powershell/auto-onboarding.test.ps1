# auto-onboarding.ps1
param(
    [string]$AuthorityUrl = "http://127.0.0.1:1500",
    [string]$ConsumerUrl  = "http://127.0.0.1:1100",
    [string]$ProviderUrl  = "http://127.0.0.1:1200"
)

function Invoke-CurlJson {
    param(
        [string]$Method = "GET",
        [string]$Url,
        [object]$Body = $null,
        [bool]$ParseJson = $true  # parsea a JSON solo si se espera JSON
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
            Write-Host "SUCCESS: $Method $Url returned $($Response.StatusCode)" -ForegroundColor Green
        } else {
            Write-Host "ERROR: $Method $Url returned $($Response.StatusCode)" -ForegroundColor Red
            exit 1
        }

        if ($ParseJson -and $Response.Content) {
            return $Response.Content | ConvertFrom-Json
        } else {
            return $Response.Content
        }

    } catch {
        Write-Host "ERROR: Request to $Url failed" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
        Write-Host "The script wont continue executing" -ForegroundColor Red
        Write-Host ""
        exit 1
    }
}

Write-Host "Starting auto-onboarding script..."

# ----------------------------
# Onboarding Authority / Consumer / Provider (no se parsea JSON)
# ----------------------------
Invoke-CurlJson -Method "POST" -Url "$AuthorityUrl/api/v1/wallet/onboard" -ParseJson:$false
Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/wallet/onboard" -ParseJson:$false
Invoke-CurlJson -Method "POST" -Url "$ProviderUrl/api/v1/wallet/onboard" -ParseJson:$false

# ----------------------------
# Getting DIDs
# ----------------------------
$AUTH_DID     = (Invoke-CurlJson -Url "$AuthorityUrl/api/v1/wallet/did.json").id
Write-Host "Authority DID: $AUTH_DID"
$CONSUMER_DID = (Invoke-CurlJson -Url "$ConsumerUrl/api/v1/wallet/did.json").id
Write-Host "Consumer DID: $CONSUMER_DID"
$PROVIDER_DID = (Invoke-CurlJson -Url "$ProviderUrl/api/v1/wallet/did.json").id
Write-Host "Provider DID: $PROVIDER_DID"

# ----------------------------
# Consumer begins request for credential
# ----------------------------
$C_BEG_BODY = @{
    url = "$AuthorityUrl/api/v1/gate/access"
    id  = $AUTH_DID
    slug = "authority"
    vc_type = "DataspaceParticipant"
}
$C_BEG_RESPONSE = Invoke-CurlJson -Method "POST" -Url "$ConsumerUrl/api/v1/vc-request/beg/cross-user" -Body $C_BEG_BODY -ParseJson:$false
Write-Host "Consumer request completed."

