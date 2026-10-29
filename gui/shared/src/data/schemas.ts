// JSON Schemas as the schema editor shows them: a tree of fields over an object root. Keywords
// the tree does not show (enum, minimum, pattern…) are kept per field and written back.

export const JSON_SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema";

export type FieldType = "string" | "integer" | "number" | "boolean" | "object" | "array";

export const FIELD_TYPES: FieldType[] = [
  "string",
  "integer",
  "number",
  "boolean",
  "object",
  "array",
];

/** String formats the editor offers; any other is kept as written. */
export const STRING_FORMATS = ["date-time", "date", "time", "email", "uri", "uuid"];

export interface SchemaField {
  /** Local key for React; not part of the schema. */
  key: string;
  name: string;
  type: FieldType;
  required: boolean;
  nullable: boolean;
  description: string;
  format: string;
  /** Properties of an object, or of the objects in an array. */
  children: SchemaField[];
  /** Item type of an array. */
  itemType: FieldType;
  /** Keywords the tree does not edit, written back unchanged. */
  extra: Record<string, unknown>;
}

/** A schema as the editor holds it. */
export interface SchemaTree {
  title: string;
  description: string;
  fields: SchemaField[];
  /** Root keywords the tree does not edit. */
  extra: Record<string, unknown>;
}

type JsonSchema = Record<string, unknown>;

// Keywords each part of the tree owns; everything else goes to `extra`.
const FIELD_KEYS = ["type", "description", "format", "properties", "required", "items"];
const ROOT_KEYS = [...FIELD_KEYS, "$schema", "$id", "title"];

let counter = 0;

const isObject = (value: unknown): value is JsonSchema =>
  !!value && typeof value === "object" && !Array.isArray(value);

const without = (schema: JsonSchema, keys: string[]) =>
  Object.fromEntries(Object.entries(schema).filter(([k]) => !keys.includes(k)));

export const SchemaTrees = {
  newKey(): string {
    counter += 1;
    return `f${counter}`;
  },

  field(name = "", type: FieldType = "string"): SchemaField {
    return {
      key: SchemaTrees.newKey(),
      name,
      type,
      required: false,
      nullable: false,
      description: "",
      format: "",
      children: [],
      itemType: "string",
      extra: {},
    };
  },

  empty(): SchemaTree {
    return { title: "", description: "", fields: [], extra: {} };
  },

  /** The tree of `schema`, or null when its root is not an object the tree can show. */
  fromJsonSchema(schema: unknown): SchemaTree | null {
    if (!isObject(schema)) return null;
    const types = SchemaTrees.types(schema.type);
    if (types.length > 0 && !(types.length === 1 && types[0] === "object")) return null;
    if (schema.properties !== undefined && !isObject(schema.properties)) return null;
    return {
      title: typeof schema.title === "string" ? schema.title : "",
      description: typeof schema.description === "string" ? schema.description : "",
      fields: SchemaTrees.readProperties(schema),
      extra: without(schema, ROOT_KEYS),
    };
  },

  toJsonSchema(tree: SchemaTree): JsonSchema {
    const schema: JsonSchema = { $schema: JSON_SCHEMA_DIALECT };
    if (tree.title.trim()) schema.title = tree.title.trim();
    if (tree.description.trim()) schema.description = tree.description.trim();
    Object.assign(schema, SchemaTrees.writeObject(tree.fields), tree.extra);
    return schema;
  },

  /** Names that are empty or repeated among siblings, by field key. */
  problems(fields: SchemaField[]): Record<string, string> {
    const problems: Record<string, string> = {};
    const seen = new Set<string>();
    for (const field of fields) {
      const name = field.name.trim();
      if (!name) problems[field.key] = "Every field needs a name.";
      else if (seen.has(name)) problems[field.key] = `"${name}" appears twice.`;
      seen.add(name);
      if (SchemaTrees.hasChildren(field))
        Object.assign(problems, SchemaTrees.problems(field.children));
    }
    return problems;
  },

  /** Whether the field holds properties: an object, or an array of objects. */
  hasChildren(field: SchemaField): boolean {
    return field.type === "object" || (field.type === "array" && field.itemType === "object");
  },

  /** How many fields the tree has, nested ones included. */
  count(fields: SchemaField[]): number {
    return fields.reduce((n, f) => n + 1 + SchemaTrees.count(f.children), 0);
  },

  types(type: unknown): string[] {
    if (typeof type === "string") return [type];
    if (Array.isArray(type)) return type.filter((t): t is string => typeof t === "string");
    return [];
  },

  readProperties(schema: JsonSchema): SchemaField[] {
    const properties = isObject(schema.properties) ? schema.properties : {};
    const required = Array.isArray(schema.required) ? schema.required : [];
    return Object.entries(properties).map(([name, value]) =>
      SchemaTrees.readField(name, isObject(value) ? value : {}, required.includes(name)),
    );
  },

  readField(name: string, schema: JsonSchema, required: boolean): SchemaField {
    const types = SchemaTrees.types(schema.type);
    const nullable = types.includes("null");
    const main = (types.find((t) => t !== "null") ?? "string") as FieldType;
    const field: SchemaField = {
      ...SchemaTrees.field(name, FIELD_TYPES.includes(main) ? main : "string"),
      required,
      nullable,
      description: typeof schema.description === "string" ? schema.description : "",
      format: typeof schema.format === "string" ? schema.format : "",
      extra: without(schema, FIELD_KEYS),
    };
    if (!FIELD_TYPES.includes(main)) field.extra.type = schema.type;
    if (main === "object") field.children = SchemaTrees.readProperties(schema);
    if (main === "array" && isObject(schema.items)) {
      const itemTypes = SchemaTrees.types(schema.items.type);
      const itemType = (itemTypes.find((t) => t !== "null") ?? "string") as FieldType;
      field.itemType = FIELD_TYPES.includes(itemType) ? itemType : "string";
      if (field.itemType === "object") field.children = SchemaTrees.readProperties(schema.items);
      const itemExtra = without(schema.items, FIELD_KEYS);
      if (Object.keys(itemExtra).length > 0) field.extra.items = itemExtra;
    }
    return field;
  },

  writeObject(fields: SchemaField[]): JsonSchema {
    const properties: JsonSchema = {};
    for (const field of fields) properties[field.name.trim()] = SchemaTrees.writeField(field);
    const required = fields.filter((f) => f.required).map((f) => f.name.trim());
    return {
      type: "object",
      properties,
      ...(required.length > 0 ? { required } : {}),
    };
  },

  writeField(field: SchemaField): JsonSchema {
    const { items: itemExtra, ...extra } = field.extra as JsonSchema & { items?: JsonSchema };
    const schema: JsonSchema = {
      type: field.nullable ? [field.type, "null"] : field.type,
    };
    if (field.description.trim()) schema.description = field.description.trim();
    if (field.type === "string" && field.format) schema.format = field.format;
    if (field.type === "object") Object.assign(schema, SchemaTrees.writeObject(field.children));
    if (field.type === "array") {
      schema.items =
        field.itemType === "object"
          ? { ...SchemaTrees.writeObject(field.children), ...(itemExtra ?? {}) }
          : { type: field.itemType, ...(itemExtra ?? {}) };
    }
    if (field.type === "object" && field.nullable) schema.type = ["object", "null"];
    return { ...schema, ...extra };
  },
};
