# PowerShell Script: populate_mock_data.ps1

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

# ==========================================
# CORE LOGIC FUNCTIONS
# ==========================================

# 1. Retrieve DIDs
function Get-Dids {
    Write-Host "Retrieving DIDs..."

    try {
        # Provider
        $ProviderDidRaw = Invoke-RestMethod -Uri "$AGENT_URL/api/v1/mates/myself" -Method Get -ErrorAction SilentlyContinue
        if ($ProviderDidRaw -is [array]) {
             $script:PROVIDER_DID = $ProviderDidRaw[0].participant_id
        } elseif ($ProviderDidRaw.error_code -eq 3120) {
             $script:PROVIDER_DID = $null
        } else {
             $script:PROVIDER_DID = $ProviderDidRaw.participant_id
        }

        # Consumer
        $ConsumerDidRaw = Invoke-RestMethod -Uri "$AGENT_URL/api/v1/mates/all" -Method Get -ErrorAction SilentlyContinue
        if ($ConsumerDidRaw -is [array]) {
            $OtherMates = $ConsumerDidRaw | Where-Object { $_.is_me -eq $false }
            # Get the last one like jq 'last'
            if ($OtherMates) {
                if ($OtherMates -is [array]) {
                    $script:CONSUMER_DID = $OtherMates[-1].participant_id
                } else {
                    $script:CONSUMER_DID = $OtherMates.participant_id
                }
            } else {
                $script:CONSUMER_DID = $null
            }
        } else {
             $script:CONSUMER_DID = $null
        }

        if ([string]::IsNullOrEmpty($script:PROVIDER_DID) -or [string]::IsNullOrEmpty($script:CONSUMER_DID)) {
            Write-Host "Warning: Could not retrieve DIDs. Using placeholders."
            $script:PROVIDER_DID = "urn:did:provider:placeholder"
            $script:CONSUMER_DID = "urn:did:consumer:placeholder"
        }

        Write-Host "Provider DID: $script:PROVIDER_DID"
        Write-Host "Consumer DID: $script:CONSUMER_DID"
    } catch {
        Write-Host "Error retrieving DIDs: $_"
    }
}

# 2. Create Catalog
function Create-Catalog {
    # Check existence
    try {
        $Check = Invoke-RestMethod -Uri "$AGENT_URL/api/v1/catalog-agent/catalogs/main" -Method Get -ErrorAction SilentlyContinue
        if ($Check -and $Check.id) {
             Write-Host "Log: Catalog already exists."
             return $Check.id
        }
    } catch {}

    $Body = @{
        foafHomePage = "superfoaf"
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/catalog-agent/catalogs" -Body $Body -Description "Creating Catalog"
    return $Resp.id
}

function Get-Main-Catalog {
    try {
        $Check = Invoke-RestMethod -Uri "$AGENT_URL/api/v1/catalog-agent/catalogs/main" -Method Get -ErrorAction SilentlyContinue
        if ($Check -and $Check.id) {
             Write-Host "Log: Catalog already exists."
             return $Check.id
        }
    } catch {
        return $null
    }
    return $null
}

# 3. Create Data Service
function Create-DataService {
    param ($CatalogId)
    $Body = @{
        dcatEndpointUrl = "superfoaf"
        catalogId = $CatalogId
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/catalog-agent/data-services" -Body $Body -Description "Creating Data Service"
    return $Resp.id
}

# 4. Create Dataset
function Create-Dataset {
    param ($Title, $CatalogId)
    $Body = @{
        dctTitle = $Title
        catalogId = $CatalogId
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/catalog-agent/datasets" -Body $Body -Description "Creating Dataset ($Title)"
    return $Resp.id
}

# 5. Create Distribution
function Create-Distribution {
    param ($Title, $DatasetId, $DataserviceId, $Format)
    $Body = @{
        dctTitle = $Title
        dcatAccessService = $DataserviceId
        datasetId = $DatasetId
        dctFormats = $Format
    } | ConvertTo-Json

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/catalog-agent/distributions" -Body $Body -Description "Creating Distribution ($Title)"
    return $Resp.id
}

# 6. Create Connector Template (Blueprint)
function Create-Blueprint {
    $Body = @{
        authentication = @{ type = "NO_AUTH" }
        interaction = @{
            mode = "PULL"
            dataAccess = @{
                protocol = "HTTP"
                urlTemplate = "{{__ACCESS_URL__}}"
                method = "{{__ALLOWED_METHODS__}}"
                headers = "{{__ALLOWED_HEADERS__}}"
            }
        }
        parameters = @(
            @{ paramType = "STRING"; name = "ACCESS_URL"; title = "Target URL"; required = $true }
            @{ paramType = "VEC<STRING>"; name = "ALLOWED_METHODS"; title = "Methods"; required = $true }
            @{ paramType = "MAP<STRING,STRING>"; name = "ALLOWED_HEADERS"; title = "Headers"; required = $true }
        )
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/connector/templates" -Body $Body -Description "Creating Connector Template"
    
    return @{
        name = $Resp.name
        version = $Resp.version
        id = $Resp.id
    }
}

# 7. Create Connector Instance
function Create-Connector-Instance {
    param ($DistId, $TargetUrl, $TName, $TVer)
    
    $Body = @{
        templateName = $TName
        templateVersion = $TVer
        distributionId = $DistId
        metadata = @{ description = "Auto-generated instance"; ownerId = "provider-admin" }
        dryRun = $false
        parameters = @{
            ACCESS_URL = $TargetUrl
            ALLOWED_METHODS = @("GET", "HEAD")
            ALLOWED_HEADERS = @{ "Content-Type" = "application/json" }
        }
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/connector/instances" -Body $Body -Description "Linking Connector to Dist $DistId"
    return $Resp.id
}

# 8. Create Policy
function Create-Policy {
    param ($EntityId)
    $Body = @{
        odrlOffer = @{
            permission = @( @{ action = "use" } )
        }
        entityId = $EntityId
        entityType = "Dataset"
    } | ConvertTo-Json -Depth 5

    $Resp = Invoke-Curl -Method "POST" -Url "$AGENT_URL/api/v1/catalog-agent/odrl-policies" -Body $Body -Description "Creating Policy for Dataset/Entity"
    return $Resp.id
}

# ==========================================
# MAIN EXECUTION
# ==========================================

function Main {
    Write-Host "=== STARTING MOCK DATA POPULATION ==="

    # 1. Retrieve DIDs
    Get-Dids

    # 2. Create Catalog
    $CATALOG_ID = Get-Main-Catalog
    if (-not $CATALOG_ID) {
        $CATALOG_ID = Create-Catalog
    }
    
    if (-not $CATALOG_ID) {
        Write-Host "Error: Failed to get or create Catalog ID."
        exit 1
    }
    Write-Host ">> Catalog ID: $CATALOG_ID"

    # 3. Create Data Service
    $DATA_SERVICE_ID = Create-DataService -CatalogId $CATALOG_ID
    Write-Host ">> Data Service ID: $DATA_SERVICE_ID"

    # 4. Create Datasets
    $DS1_ID = Create-Dataset -Title "Weather Data 2024" -CatalogId $CATALOG_ID
    $DS2_ID = Create-Dataset -Title "Traffic Sensor Logs" -CatalogId $CATALOG_ID
    $DS3_ID = Create-Dataset -Title "Public Transport Schedules" -CatalogId $CATALOG_ID
    Write-Host ">> Datasets Created: $DS1_ID, $DS2_ID, $DS3_ID"

    # 5. Create Distributions
    $DI1_ID = Create-Distribution -Title "Weather Data - JSON" -DatasetId $DS1_ID -DataserviceId $DATA_SERVICE_ID -Format "http+pull"
    $DI2_ID = Create-Distribution -Title "Traffic Logs - JSON" -DatasetId $DS1_ID -DataserviceId $DATA_SERVICE_ID -Format "http+push"
    $DI3_ID = Create-Distribution -Title "Public Transport - JSON" -DatasetId $DS2_ID -DataserviceId $DATA_SERVICE_ID -Format "http+pull"
    $DI4_ID = Create-Distribution -Title "Weather Data Alt - JSON" -DatasetId $DS3_ID -DataserviceId $DATA_SERVICE_ID -Format "http+pull"
    $DI5_ID = Create-Distribution -Title "Traffic Logs Alt - JSON" -DatasetId $DS3_ID -DataserviceId $DATA_SERVICE_ID -Format "http+push"
    $DI6_ID = Create-Distribution -Title "Public Transport Alt - JSON" -DatasetId $DS3_ID -DataserviceId $DATA_SERVICE_ID -Format "http+pull"
    Write-Host ">> Distributions Created."

    # 6. Create Blueprint (Template)
    Write-Host ">> Creating Blueprint..."
    $BP_DATA = Create-Blueprint
    $BP_NAME = $BP_DATA.name
    $BP_VER = $BP_DATA.version
    $BP_ID = $BP_DATA.id
    Write-Host ">> Blueprint Created: $BP_NAME v$BP_VER (ID: $BP_ID)"

    # 7. Create Connector Instances
    if ($DI1_ID) {
        $CI1 = Create-Connector-Instance -DistId $DI1_ID -TargetUrl "https://jsonplaceholder.typicode.com/posts" -TName $BP_NAME -TVer $BP_VER
        Write-Host ">> Connector Instance for DI1: $CI1"
    }
    if ($DI3_ID) {
        $CI3 = Create-Connector-Instance -DistId $DI3_ID -TargetUrl "https://jsonplaceholder.typicode.com/albums" -TName $BP_NAME -TVer $BP_VER
        Write-Host ">> Connector Instance for DI3: $CI3"
    }
    if ($DI6_ID) {
        $CI6 = Create-Connector-Instance -DistId $DI6_ID -TargetUrl "https://jsonplaceholder.typicode.com/users" -TName $BP_NAME -TVer $BP_VER
        Write-Host ">> Connector Instance for DI6: $CI6"
    }

    # 8. Create Policies
    $POLICY_ID1 = Create-Policy -EntityId $DS1_ID
    Write-Host ">> Policy for DS1 (1): $POLICY_ID1"

    $POLICY_ID11 = Create-Policy -EntityId $DS1_ID
    Write-Host ">> Policy for DS1 (2): $POLICY_ID11"

    $POLICY_ID12 = Create-Policy -EntityId $DS1_ID
    Write-Host ">> Policy for DS1 (3): $POLICY_ID12"

    $POLICY_ID13 = Create-Policy -EntityId $DS1_ID
    Write-Host ">> Policy for DS1 (4): $POLICY_ID13"

    $POLICY_ID2 = Create-Policy -EntityId $DS2_ID
    Write-Host ">> Policy for DS2: $POLICY_ID2"

    $POLICY_ID3 = Create-Policy -EntityId $DS3_ID
    Write-Host ">> Policy for DS3: $POLICY_ID3"

    Write-Host "=== MOCK DATA POPULATION FINISHED ==="
}

Main
