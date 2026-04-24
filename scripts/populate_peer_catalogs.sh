#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
# Target Redis container: provider-redis | consumer-redis
REDIS_CONTAINER="${REDIS_CONTAINER:-provider-redis}"

# Redis auth — must match the container's --requirepass / --user config
REDIS_USER="${REDIS_USER:-default}"
REDIS_PASS="${REDIS_PASS:-ds_core_provider_redis}"

# TTL in seconds: PEER_CATALOG_DESIRED_CACHE_TTL = ONE_DAY_TTL * 10
REDIS_TTL=864000

KEY_PREFIX="ds_agent_catalogs:peer-catalog"

# ==========================================
# HELPER
# ==========================================
redis_exec() {
    docker exec "$REDIS_CONTAINER" redis-cli \
        --user "$REDIS_USER" \
        -a "$REDIS_PASS" \
        --no-auth-warning \
        "$@"
}

set_catalog() {
    local participant_id="$1"
    local catalog_json="$2"
    local key="${KEY_PREFIX}:${participant_id}"

    echo "--- Seeding catalog for: $participant_id ---"
    redis_exec DEL "$key"
    redis_exec JSON.SET "$key" '$' "$catalog_json"
    redis_exec EXPIRE "$key" "$REDIS_TTL"
    echo "    key=$key  TTL=${REDIS_TTL}s"
    echo ""
}

# ==========================================
# CATALOG DEFINITIONS
#
# JSON field names match the Rust serde structs exactly:
#   Catalog:      identifier(req), issued(datetime), description(array),
#                 catalog(req,array), dataset(req), service(req)
#   Dataset:      identifier(req), issued(datetime), hasPolicy(array), distribution(req)
#   Distribution: issued(datetime), formats(req,string), accessService(optional object)
#   OdrlOffer:    @id(urn), @type("Offer")
#   Service(min): @type, @id, endpointURL
# ==========================================

# ---- ACME Corp — Manufacturing ----
ACME_CATALOG=$(cat <<'EOF'
{
  "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
  "@type": "dcat:Catalog",
  "@id": "urn:catalog:acme-corp:main",
  "identifier": "urn:catalog:acme-corp:main",
  "issued": "2025-01-10T00:00:00",
  "modified": "2026-03-15T00:00:00",
  "title": "ACME Corp Manufacturing Data Catalog",
  "description": ["Open datasets from ACME Corp manufacturing operations"],
  "participantId": "did:web:agent:acme-corp",
  "catalog": [],
  "dataset": [
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:acme-corp:production-metrics",
      "identifier": "urn:dataset:acme-corp:production-metrics",
      "issued": "2025-01-10T00:00:00",
      "modified": "2026-03-15T00:00:00",
      "title": "Production Line Metrics",
      "description": ["Real-time sensor readings from production lines: throughput, downtime, OEE"],
      "keyword": "manufacturing oee production sensor",
      "hasPolicy": [
        {
          "@id": "urn:offer:acme-corp:production-metrics:free",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:acme-corp:production-metrics:http",
          "issued": "2025-01-10T00:00:00",
          "formats": "application/json",
          "title": "Production Metrics — JSON",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:acme-corp:manufacturing-api",
            "endpointURL": "https://api.acme-corp.example.com/dataspace/v1"
          }
        }
      ]
    },
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:acme-corp:supply-chain",
      "identifier": "urn:dataset:acme-corp:supply-chain",
      "issued": "2025-06-01T00:00:00",
      "modified": "2026-02-28T00:00:00",
      "title": "Supply Chain Logistics",
      "description": ["Inbound and outbound logistics: shipment status, lead times, supplier KPIs"],
      "keyword": "supply-chain logistics shipment manufacturing",
      "hasPolicy": [
        {
          "@id": "urn:offer:acme-corp:supply-chain:agreement",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:acme-corp:supply-chain:csv",
          "issued": "2025-06-01T00:00:00",
          "formats": "text/csv",
          "title": "Supply Chain — CSV",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:acme-corp:manufacturing-api",
            "endpointURL": "https://api.acme-corp.example.com/dataspace/v1"
          }
        }
      ]
    }
  ],
  "service": {
    "@type": "dcat:DataService",
    "@id": "urn:dataservice:acme-corp:manufacturing-api",
    "endpointURL": "https://api.acme-corp.example.com/dataspace/v1",
    "endpointDescription": "urn:openapi:acme-corp:manufacturing-api"
  }
}
EOF
)

# ---- Globex Industries — Energy ----
GLOBEX_CATALOG=$(cat <<'EOF'
{
  "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
  "@type": "dcat:Catalog",
  "@id": "urn:catalog:globex:main",
  "identifier": "urn:catalog:globex:main",
  "issued": "2024-09-01T00:00:00",
  "modified": "2026-04-01T00:00:00",
  "title": "Globex Industries Energy Data Catalog",
  "description": ["Energy production, consumption and grid datasets from Globex Industries"],
  "participantId": "did:web:agent:globex",
  "catalog": [],
  "dataset": [
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:globex:energy-production",
      "identifier": "urn:dataset:globex:energy-production",
      "issued": "2024-09-01T00:00:00",
      "modified": "2026-04-01T00:00:00",
      "title": "Renewable Energy Production",
      "description": ["Hourly renewable energy generation data (solar, wind, hydro) aggregated by plant"],
      "keyword": "energy renewable solar wind production",
      "hasPolicy": [
        {
          "@id": "urn:offer:globex:energy-production:open",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:globex:energy-production:json",
          "issued": "2024-09-01T00:00:00",
          "formats": "application/json",
          "title": "Energy Production — JSON",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:globex:energy-api",
            "endpointURL": "https://data.globex.example.com/dsp/v1"
          }
        },
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:globex:energy-production:parquet",
          "issued": "2024-09-01T00:00:00",
          "formats": "application/vnd.apache.parquet",
          "title": "Energy Production — Parquet",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:globex:energy-api",
            "endpointURL": "https://data.globex.example.com/dsp/v1"
          }
        }
      ]
    },
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:globex:grid-stability",
      "identifier": "urn:dataset:globex:grid-stability",
      "issued": "2025-03-15T00:00:00",
      "modified": "2026-03-30T00:00:00",
      "title": "Grid Stability Indicators",
      "description": ["Frequency deviation, voltage levels and balancing reserve data for European grid segments"],
      "keyword": "grid stability frequency energy europe",
      "hasPolicy": [
        {
          "@id": "urn:offer:globex:grid-stability:restricted",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:globex:grid-stability:json",
          "issued": "2025-03-15T00:00:00",
          "formats": "application/json",
          "title": "Grid Stability — JSON",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:globex:energy-api",
            "endpointURL": "https://data.globex.example.com/dsp/v1"
          }
        }
      ]
    }
  ],
  "service": {
    "@type": "dcat:DataService",
    "@id": "urn:dataservice:globex:energy-api",
    "endpointURL": "https://data.globex.example.com/dsp/v1",
    "endpointDescription": "urn:openapi:globex:energy-api"
  }
}
EOF
)

# ---- Initech Solutions — Finance ----
INITECH_CATALOG=$(cat <<'EOF'
{
  "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
  "@type": "dcat:Catalog",
  "@id": "urn:catalog:initech:main",
  "identifier": "urn:catalog:initech:main",
  "issued": "2025-02-01T00:00:00",
  "modified": "2026-04-10T00:00:00",
  "title": "Initech Solutions Financial Data Catalog",
  "description": ["Anonymised financial datasets from Initech Solutions for research and compliance use"],
  "participantId": "did:web:agent:initech",
  "catalog": [],
  "dataset": [
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:initech:transaction-stats",
      "identifier": "urn:dataset:initech:transaction-stats",
      "issued": "2025-02-01T00:00:00",
      "modified": "2026-04-10T00:00:00",
      "title": "Anonymised Transaction Statistics",
      "description": ["Aggregated and anonymised payment transaction statistics by region and category (monthly)"],
      "keyword": "finance transactions anonymised statistics",
      "hasPolicy": [
        {
          "@id": "urn:offer:initech:transaction-stats:research",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:initech:transaction-stats:csv",
          "issued": "2025-02-01T00:00:00",
          "formats": "text/csv",
          "title": "Transaction Stats — CSV",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:initech:finance-api",
            "endpointURL": "https://ds.initech.example.com/api/v2"
          }
        }
      ]
    },
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:initech:risk-indicators",
      "identifier": "urn:dataset:initech:risk-indicators",
      "issued": "2025-07-15T00:00:00",
      "modified": "2026-03-01T00:00:00",
      "title": "Credit Risk Indicators",
      "description": ["Sector-level credit risk scores and default rate trends derived from internal models"],
      "keyword": "finance credit risk indicators",
      "hasPolicy": [
        {
          "@id": "urn:offer:initech:risk-indicators:commercial",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:initech:risk-indicators:json",
          "issued": "2025-07-15T00:00:00",
          "formats": "application/json",
          "title": "Risk Indicators — JSON",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:initech:finance-api",
            "endpointURL": "https://ds.initech.example.com/api/v2"
          }
        }
      ]
    }
  ],
  "service": {
    "@type": "dcat:DataService",
    "@id": "urn:dataservice:initech:finance-api",
    "endpointURL": "https://ds.initech.example.com/api/v2",
    "endpointDescription": "urn:openapi:initech:finance-api"
  }
}
EOF
)

# ---- Umbrella Data Co — Healthcare ----
UMBRELLA_CATALOG=$(cat <<'EOF'
{
  "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
  "@type": "dcat:Catalog",
  "@id": "urn:catalog:umbrella:main",
  "identifier": "urn:catalog:umbrella:main",
  "issued": "2024-11-01T00:00:00",
  "modified": "2026-01-20T00:00:00",
  "title": "Umbrella Data Co Healthcare Catalog",
  "description": ["Pseudonymised clinical and population health datasets from Umbrella Data Co"],
  "participantId": "did:web:agent:umbrella",
  "catalog": [],
  "dataset": [
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:umbrella:patient-outcomes",
      "identifier": "urn:dataset:umbrella:patient-outcomes",
      "issued": "2024-11-01T00:00:00",
      "modified": "2026-01-20T00:00:00",
      "title": "Pseudonymised Patient Outcome Registry",
      "description": ["De-identified longitudinal patient outcomes for cardiovascular and respiratory conditions (2018-2025)"],
      "keyword": "healthcare clinical outcomes pseudonymised longitudinal",
      "hasPolicy": [
        {
          "@id": "urn:offer:umbrella:patient-outcomes:research",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:umbrella:patient-outcomes:fhir",
          "issued": "2024-11-01T00:00:00",
          "formats": "application/fhir+json",
          "title": "Patient Outcomes — FHIR R4",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:umbrella:health-api",
            "endpointURL": "https://data.umbrella.example.com/dsp/v1"
          }
        }
      ]
    },
    {
      "@type": "dcat:Dataset",
      "@id": "urn:dataset:umbrella:population-health",
      "identifier": "urn:dataset:umbrella:population-health",
      "issued": "2025-05-01T00:00:00",
      "modified": "2026-04-01T00:00:00",
      "title": "Population Health Indicators",
      "description": ["Aggregated public health KPIs by region: prevalence rates, vaccination coverage, hospital admission trends"],
      "keyword": "healthcare population public-health kpi aggregated",
      "hasPolicy": [
        {
          "@id": "urn:offer:umbrella:population-health:open",
          "@type": "Offer"
        }
      ],
      "distribution": [
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:umbrella:population-health:json",
          "issued": "2025-05-01T00:00:00",
          "formats": "application/json",
          "title": "Population Health — JSON",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:umbrella:health-api",
            "endpointURL": "https://data.umbrella.example.com/dsp/v1"
          }
        },
        {
          "@type": "dcat:Distribution",
          "@id": "urn:distribution:umbrella:population-health:csv",
          "issued": "2025-05-01T00:00:00",
          "formats": "text/csv",
          "title": "Population Health — CSV",
          "description": [],
          "accessService": {
            "@type": "dcat:DataService",
            "@id": "urn:dataservice:umbrella:health-api",
            "endpointURL": "https://data.umbrella.example.com/dsp/v1"
          }
        }
      ]
    }
  ],
  "service": {
    "@type": "dcat:DataService",
    "@id": "urn:dataservice:umbrella:health-api",
    "endpointURL": "https://data.umbrella.example.com/dsp/v1",
    "endpointDescription": "urn:openapi:umbrella:health-api"
  }
}
EOF
)

# ==========================================
# MAIN
# ==========================================
main() {
    echo "=== SEEDING PEER CATALOGS INTO REDIS ==="
    echo "    container : $REDIS_CONTAINER"
    echo "    user      : $REDIS_USER"
    echo "    TTL       : ${REDIS_TTL}s ($(( REDIS_TTL / 86400 )) days)"
    echo ""

    # Verify container is running
    if ! docker inspect --format '{{.State.Running}}' "$REDIS_CONTAINER" 2>/dev/null | grep -q "true"; then
        echo "ERROR: Container '$REDIS_CONTAINER' is not running." >&2
        exit 1
    fi

    # Verify redis-cli can connect
    if ! redis_exec PING 2>/dev/null | grep -q "PONG"; then
        echo "ERROR: Cannot reach redis-cli in '$REDIS_CONTAINER' with the given credentials." >&2
        exit 1
    fi

    set_catalog "did:web:agent:acme-corp"  "$ACME_CATALOG"
    set_catalog "did:web:agent:globex"     "$GLOBEX_CATALOG"
    set_catalog "did:web:agent:initech"    "$INITECH_CATALOG"
    set_catalog "did:web:agent:umbrella"   "$UMBRELLA_CATALOG"

    echo "=== DONE. 4 peer catalogs seeded. ==="
    echo ""
    echo "Verify with:"
    echo "  docker exec $REDIS_CONTAINER redis-cli --user $REDIS_USER -a $REDIS_PASS --no-auth-warning KEYS 'ds_agent_catalogs:peer-catalog:*'"
}

main "$@"
