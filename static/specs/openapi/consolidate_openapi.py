import yaml
import json
import copy
import os

# Helper to load file (handles json and yaml)
def load_spec(path):
    with open(path, 'r') as f:
        if path.endswith('.json'):
            return json.load(f)
        else:
            return yaml.safe_load(f)

# Load ALL specs
print("Loading specs...")
base_path = '/Users/apabook/Desktop/ds-protocol/static/specs/openapi/'
catalog_dsp = load_spec(os.path.join(base_path, 'catalog/catalog_dsp.json'))
catalog_agent = load_spec(os.path.join(base_path, 'catalog/catalog_agent.json'))
negotiation_agent = load_spec(os.path.join(base_path, 'contracts/negotiation_agent.json'))
negotiation_dsp = load_spec(os.path.join(base_path, 'contracts/negotiation_dsp.yaml'))
transfer_dsp = load_spec(os.path.join(base_path, 'transfer/transfer_dsp.yaml'))
transfer_agent = load_spec(os.path.join(base_path, 'transfer/transfer_agent.yaml'))
fe_gateway = load_spec(os.path.join(base_path, 'fe_gateway.yaml'))

final_spec = copy.deepcopy(fe_gateway)

# --- 1. Consolidate Schemas ---
print("Consolidating Schemas...")
if 'components' not in final_spec:
    final_spec['components'] = {}
if 'schemas' not in final_spec['components']:
    final_spec['components']['schemas'] = {}

# Helper to merge schemas (overwrite if exists)
def merge_schemas(source_spec, prefix=""):
    if 'components' in source_spec and 'schemas' in source_spec['components']:
        for name, schema in source_spec['components']['schemas'].items():
            # If explicit DTOs requested, we prioritize them
            final_spec['components']['schemas'][name] = schema

# Merge Catalog Schemas
merge_schemas(catalog_agent) 
merge_schemas(catalog_dsp)

# Generate Aliases for Catalog
if 'NewCatalogDto' in final_spec['components']['schemas']:
    final_spec['components']['schemas']['CatalogNewDto'] = final_spec['components']['schemas']['NewCatalogDto']
if 'EditCatalogDto' in final_spec['components']['schemas']:
    final_spec['components']['schemas']['CatalogEditDto'] = final_spec['components']['schemas']['EditCatalogDto']
for entity in ['DataService', 'Dataset', 'Distribution']:
    if f'New{entity}Dto' in final_spec['components']['schemas']:
        final_spec['components']['schemas'][f'{entity}NewDto'] = final_spec['components']['schemas'][f'New{entity}Dto']
    if f'Edit{entity}Dto' in final_spec['components']['schemas']:
        final_spec['components']['schemas'][f'{entity}EditDto'] = final_spec['components']['schemas'][f'Edit{entity}Dto']

# Merge Negotiation Schemas (Strict extraction)
merge_schemas(negotiation_agent)
merge_schemas(negotiation_dsp)

# Merge Transfer Schemas
merge_schemas(transfer_agent)
merge_schemas(transfer_dsp)

# Ensure ErrorInfo (from catalog_dsp usually)
if 'ErrorInfo' in catalog_dsp.get('components', {}).get('schemas', {}):
    final_spec['components']['schemas']['ErrorInfo'] = catalog_dsp['components']['schemas']['ErrorInfo']
elif 'ErrorInfo' not in final_spec['components']['schemas']:
    # Fallback definition if not found
    final_spec['components']['schemas']['ErrorInfo'] = {
        "type": "object",
        "required": ["message", "error_code", "cause"],
        "properties": {
            "message": {"type": "string"},
            "error_code": {"type": "integer"},
            "details": {"type": "string"},
            "cause": {"type": "string"}
        }
    }

# --- 1.5 Add Missing OdrlInfo ---
print("Adding missing OdrlInfo schema...")
# Extracted from user Rust struct and existing catalog_dsp schemas
odrl_info_schema = {
    "type": "object",
    "properties": {
        "profile": {
            "oneOf": [
                {"type": "string"},
                {"type": "object"}, # allowing object too as per oneOf in OdrlOffer might be tricky, usually string or array of strings. 
                # Keep flexible for now:
                {"type": "array", "items": {"type": "string"}}
            ]
        },
        "permission": {
            "type": "array",
            "items": {"$ref": "#/components/schemas/OdrlPermission"}
        },
        "obligation": {
            "type": "array",
            "items": {"$ref": "#/components/schemas/OdrlObligation"}
        },
        "prohibition": {
            "type": "array",
            "items": {"$ref": "#/components/schemas/OdrlObligation"}
        }
    }
}
final_spec['components']['schemas']['OdrlInfo'] = odrl_info_schema
# Also add OdrlPolicyInfo as alias if needed (NewOdrlPolicyDto uses it)
final_spec['components']['schemas']['OdrlPolicyInfo'] = odrl_info_schema


# --- 2. Fix RPC Operation IDs ---
print("Fixing RPC Operation IDs...")

# Helper to find RPC operationId in source by suffix match
def find_source_rpc_op_id(path_suffix, source_spec):
    # path_suffix like '/rpc/setup-catalog-request'
    for path, path_item in source_spec.get('paths', {}).items():
        if path.endswith(path_suffix):
            if 'post' in path_item:
                return path_item['post'].get('operationId')
    return None

# Iterate Gateway paths
for path, path_item in final_spec['paths'].items():
    if '/rpc/' in path:
        suffix = path.split('/rpc/')[-1]
        full_suffix = '/rpc/' + suffix
        
        new_op_id = None
        # Determine source SPEC based on path prefix
        if path.startswith('/catalogs/'):
             new_op_id = find_source_rpc_op_id(full_suffix, catalog_dsp)
        elif path.startswith('/negotiations/'):
             new_op_id = find_source_rpc_op_id(full_suffix, negotiation_dsp)
        elif path.startswith('/transfers/'):
             new_op_id = find_source_rpc_op_id(full_suffix, transfer_dsp)
        
        # If not found by prefix (or unexpected path), fall back to checking all (carefully)
        if not new_op_id and path.startswith('/rpc/'):
             # Generic RPC paths?
             pass 

        if new_op_id:
            print(f"Update {path}: {path_item.get('post', {}).get('operationId')} -> {new_op_id}")
            if 'post' in path_item:
                path_item['post']['operationId'] = new_op_id

# --- 3. Add Error Responses ---
print("Adding Error Responses...")
standard_errors = {
    "400": {"description": "Bad Request", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorInfo"}}}},
    "401": {"description": "Unauthorized", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorInfo"}}}},
    "403": {"description": "Forbidden", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorInfo"}}}},
    "404": {"description": "Not Found", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorInfo"}}}},
    "500": {"description": "Internal Server Error", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorInfo"}}}}
}

for path, path_item in final_spec['paths'].items():
    for method, op in path_item.items():
        if method in ['get', 'post', 'put', 'delete', 'patch']:
            if 'responses' not in op:
                op['responses'] = {}
            for code, response in standard_errors.items():
                if code not in op['responses']:
                    # Use deepcopy to avoid YAML anchors/aliases in output
                    op['responses'][code] = copy.deepcopy(response)
                else:
                    # Update existing error response to use ErrorInfo if it doesn't
                    op['responses'][code]['content'] = {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/ErrorInfo"}
                        }
                    }

# --- 3.5 Fix OpenAPI 3.1 Compatibility (Nullable & Examples) ---
print("Fixing OpenAPI 3.1 Compatibility...")

def fix_schema_node(schema):
    if not isinstance(schema, dict):
        return

    # Fix nullable
    if schema.get('nullable') is True:
        schema.pop('nullable')
        current_type = schema.get('type')
        if current_type:
            if isinstance(current_type, str):
                schema['type'] = [current_type, 'null']
            elif isinstance(current_type, list) and 'null' not in current_type:
                schema['type'] = current_type + ['null']
        # If no type (e.g. any type or OneOf), nullable implies 'null' is allowed.
        # But without a type, it's hard to express in simple [T, null]. 
        # Usually type exists if nullable is true.

    # Fix example -> examples (Schema Object)
    if 'example' in schema:
        example_val = schema.pop('example')
        # Only add examples if it doesn't exist? Or merge? 
        # Spec says examples is array.
        if 'examples' not in schema:
            schema['examples'] = [example_val]
    
    # Recursion
    for key, val in schema.items():
        if key == 'properties' and isinstance(val, dict):
            for prop_schema in val.values():
                fix_schema_node(prop_schema)
        elif key == 'items' and isinstance(val, dict):
            fix_schema_node(val)
        elif key in ['allOf', 'oneOf', 'anyOf'] and isinstance(val, list):
            for sub_schema in val:
                fix_schema_node(sub_schema)
        elif key == 'additionalProperties' and isinstance(val, dict):
            fix_schema_node(val)

# 1. Fix Components Schemas
if 'components' in final_spec and 'schemas' in final_spec['components']:
    for s in final_spec['components']['schemas'].values():
        fix_schema_node(s)

# 2. Fix Paths
if 'paths' in final_spec:
    for path_item in final_spec['paths'].values():
        for op_name, op_val in path_item.items():
            if op_name in ['get', 'post', 'put', 'delete', 'patch', 'head', 'options', 'trace']:
                # Parameters
                if 'parameters' in op_val:
                    for param in op_val['parameters']:
                        if 'schema' in param:
                            fix_schema_node(param['schema'])
                
                # Request Body
                if 'requestBody' in op_val and 'content' in op_val['requestBody']:
                    for media in op_val['requestBody']['content'].values():
                        if 'schema' in media:
                            fix_schema_node(media['schema'])
                
                # Responses
                if 'responses' in op_val:
                    for resp in op_val['responses'].values():
                        if 'content' in resp:
                            for media in resp['content'].values():
                                if 'schema' in media:
                                    fix_schema_node(media['schema'])

# --- 4. Write Output ---
print("Writing output...")
output_path = '/Users/apabook/Desktop/ds-protocol/static/specs/openapi/openapi_consolidated.yaml'

class NoAliasDumper(yaml.SafeDumper):
    def ignore_aliases(self, data):
        return True

with open(output_path, 'w') as f:
    # Use SafeDumper and turn off aliases entirely to be safe
    yaml.dump(final_spec, f, Dumper=NoAliasDumper, sort_keys=False, default_flow_style=False)

print("Done.")
