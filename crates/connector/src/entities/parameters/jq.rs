use serde_json::Value;

/// Evaluate a jq expression against a JSON value. Returns the first output, or `None` on
/// parse/compile/runtime error (warnings are emitted via `tracing`).
pub(crate) fn run_jq(expr: &str, value: Value) -> Option<Value> {
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
        tracing::warn!(
            "run_jq: {} compile error(s) for {:?}",
            defs.errs.len(),
            expr
        );
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
