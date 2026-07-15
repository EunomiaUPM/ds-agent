/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

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
