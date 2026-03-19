use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

fn template_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(.*?)__\s*\}\}").expect("Invalid Regex"))
}

fn response_key_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    // Matches: RUNTIME_{TYPE}_RESPONSE_{jq_expr}
    // Example: RUNTIME_SUB_RESPONSE_{.id}
    //          RUNTIME_AUTH_RESPONSE_{.access_token}
    REGEX.get_or_init(|| {
        Regex::new(r"^RUNTIME_([A-Z_]+)_RESPONSE_\{(.+)\}$").expect("Invalid response key regex")
    })
}

/// Evaluate a jq expression against a JSON value. Returns the first output value, if any.
fn run_jq(expr: &str, value: Value) -> Option<Value> {
    use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};

    let (f, errs) = jaq_parse::parse(expr, jaq_parse::main());
    if !errs.is_empty() {
        tracing::warn!("run_jq: parse errors for {:?}: {:?}", expr, errs);
        return None;
    }
    let f = f?;

    let mut defs = ParseCtx::new(Vec::new());
    let filter = defs.compile(f);

    if !defs.errs.is_empty() {
        tracing::warn!("run_jq: {} compile error(s) for {:?}", defs.errs.len(), expr);
        return None;
    }

    let inputs = RcIter::new(core::iter::empty());
    let result = filter
        .run((Ctx::new([], &inputs), Val::from(value)))
        .next()
        .and_then(|r| r.ok())
        .map(Value::from);

    result
}

pub struct TemplateParametersResolver<'a> {
    values: &'a HashMap<String, Value>,
    context_stack: Vec<String>,
    /// Named response contexts for RUNTIME_*_RESPONSE_{jq} extraction.
    /// Key: uppercase type name (e.g. "SUB", "AUTH")
    /// Value: the JSON response body to run jq against
    response_contexts: HashMap<String, Value>,
}

impl<'a> TemplateParametersResolver<'a> {
    pub fn new(values: &'a HashMap<String, Value>) -> Self {
        Self { values, context_stack: vec![], response_contexts: HashMap::new() }
    }

    /// Attach a named response context used to resolve `RUNTIME_{name}_RESPONSE_{jq}` placeholders.
    /// Call before passing the resolver to `accept_mutator`.
    ///
    /// Example:
    ///   `resolver.with_response_context("SUB", flow_control_value)`
    ///   → `{{__RUNTIME_SUB_RESPONSE_{.id}__}}` resolves via `.id` applied to `flow_control_value`
    pub fn with_response_context(mut self, context_type: &str, value: Value) -> Self {
        self.response_contexts.insert(context_type.to_uppercase(), value);
        self
    }

    /// Resolve a single placeholder key, first from the params map, then via jq extraction.
    fn resolve_key(&self, key: &str) -> Option<Value> {
        if let Some(val) = self.values.get(key) {
            return Some(val.clone());
        }
        self.try_response_extraction(key)
    }

    /// Try to match `RUNTIME_{TYPE}_RESPONSE_{jq}` and evaluate the jq expression.
    fn try_response_extraction(&self, key: &str) -> Option<Value> {
        let caps = response_key_regex().captures(key)?;
        let context_type = caps.get(1)?.as_str(); // e.g. "SUB"
        let jq_expr = caps.get(2)?.as_str(); // e.g. ".id"

        let context_value = self.response_contexts.get(context_type)?;
        run_jq(jq_expr, context_value.clone())
    }

    fn try_exact_replacement(&self, re: &Regex, raw: &str) -> Option<Value> {
        let caps = re.captures(raw)?;
        let full_match = caps.get(0)?.as_str();
        if full_match != raw {
            return None;
        }
        let key = caps.get(1)?.as_str();
        self.resolve_key(key)
    }

    fn try_string_interpolation(&self, re: &Regex, raw: &str) -> Option<Value> {
        if !re.is_match(raw) {
            return None;
        }
        let mut new_string = raw.to_string();
        for caps in re.captures_iter(raw) {
            let full_match = &caps[0];
            let key = &caps[1];

            if let Some(val) = self.resolve_key(key) {
                let replacement_str = self.value_to_string(&val);
                new_string = new_string.replace(full_match, &replacement_str);
            }
        }
        Some(Value::String(new_string))
    }

    fn value_to_string(&self, val: &Value) -> String {
        match val {
            Value::String(s) => s.clone(),
            _ => val.to_string(),
        }
    }
}

/// Drives in-place template resolution by walking a `TemplateMutable` tree.
///
/// Implementors provide the three leaf operations below; the tree structure is
/// handled by each type's `TemplateMutable::accept_mutator` implementation.
///
/// The scope stack (`enter_scope` / `exit_scope`) lets implementors track the
/// JSON path to the field currently being resolved, which is useful for error
/// messages and context-aware resolution.
pub trait ParameterResolverBehavior {
    /// Push a named scope onto the resolution context stack.
    fn enter_scope(&mut self, name: &str);

    /// Pop the innermost scope from the resolution context stack.
    fn exit_scope(&mut self);

    /// Attempt to resolve `raw`.
    ///
    /// - Returns `Some(value)` if `raw` contains a placeholder that maps to a
    ///   known parameter, in which case the caller should replace the field.
    /// - Returns `None` to leave the field unchanged.
    fn resolve(&self, raw: &str) -> Option<Value>;
}

impl ParameterResolverBehavior for TemplateParametersResolver<'_> {
    fn enter_scope(&mut self, name: &str) {
        self.context_stack.push(name.to_string());
    }

    fn exit_scope(&mut self) {
        self.context_stack.pop();
    }

    fn resolve(&self, raw: &str) -> Option<Value> {
        let re = template_regex();
        if let Some(val) = self.try_exact_replacement(re, raw) {
            return Some(val);
        }
        self.try_string_interpolation(re, raw)
    }
}
