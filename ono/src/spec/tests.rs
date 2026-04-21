//! Tests against the three anchor specs.

use super::*;
use crate::theme::Theme;

fn catalog() -> Catalog {
    Catalog::load().expect("catalog parses")
}

#[test]
fn catalog_loads_full_catalog() {
    let cat = catalog();
    let expected_elements =
        ["box", "percentage", "progress", "sparkline", "spinner", "typewriter"];
    let expected_components = ["boot", "dashboard", "map", "splash", "statusbar"];
    for name in expected_elements.iter().chain(expected_components.iter()) {
        assert!(cat.get(name).is_some(), "{name} is present");
    }
    assert_eq!(cat.len(), expected_elements.len() + expected_components.len());
}

#[test]
fn progress_is_element_with_no_compose() {
    let cat = catalog();
    let p = cat.get("progress").unwrap();
    assert_eq!(p.component.kind, Kind::Element);
    assert!(p.compose.is_empty());
    assert_eq!(p.params.len(), 5);
    assert_eq!(p.params.get("percent").unwrap().ty, ParamType::Float);
    assert_eq!(p.params.get("style").unwrap().ty, ParamType::Enum);
}

#[test]
fn statusbar_composes_three_elements() {
    let cat = catalog();
    let s = cat.get("statusbar").unwrap();
    assert_eq!(s.component.kind, Kind::Component);
    let children: Vec<_> = s.compose.iter().map(|c| c.child().unwrap()).collect();
    assert_eq!(children, vec!["spinner", "progress", "percentage"]);
}

#[test]
fn deps_for_element_is_just_itself() {
    let cat = catalog();
    let d = deps(&cat, "progress").unwrap();
    assert_eq!(d, vec!["progress"]);
}

#[test]
fn deps_for_unknown_errors() {
    let cat = catalog();
    let err = deps(&cat, "nope").unwrap_err();
    assert!(matches!(err, ResolveError::Unknown(_)));
}

#[test]
fn expr_literal_string() {
    let ctx = ParamCtx::new();
    assert_eq!(eval_pass("unicode", &ctx).unwrap(), Value::Str("unicode".into()));
}

#[test]
fn expr_ident_lookup() {
    let mut ctx = ParamCtx::new();
    ctx.insert("percent".into(), Value::Float(0.42));
    assert_eq!(eval_pass("${percent}", &ctx).unwrap(), Value::Float(0.42));
}

#[test]
fn expr_int_arithmetic() {
    let mut ctx = ParamCtx::new();
    ctx.insert("width".into(), Value::Int(60));
    assert_eq!(eval_pass("${width - 20}", &ctx).unwrap(), Value::Int(40));
}

#[test]
fn expr_precedence_and_parens() {
    let ctx = ParamCtx::new();
    assert_eq!(eval_expr("2 + 3 * 4", &ctx).unwrap(), Value::Int(14));
    assert_eq!(eval_expr("(2 + 3) * 4", &ctx).unwrap(), Value::Int(20));
}

#[test]
fn expr_promotes_int_to_float() {
    let ctx = ParamCtx::new();
    let v = eval_expr("1 + 0.5", &ctx).unwrap();
    assert_eq!(v, Value::Float(1.5));
}

#[test]
fn expr_unknown_ident_errors() {
    let ctx = ParamCtx::new();
    let err = eval_expr("missing", &ctx).unwrap_err();
    assert!(matches!(err, ExprError::UnknownIdent(_)));
}

#[test]
fn expr_div_by_zero() {
    let ctx = ParamCtx::new();
    assert!(matches!(eval_expr("1 / 0", &ctx), Err(ExprError::DivByZero)));
}

#[test]
fn idents_in_pass_extracts_names() {
    let names = expr::idents_in_pass("${width - 20}");
    assert_eq!(names, vec!["width"]);
    let names = expr::idents_in_pass("${a + b * a}");
    assert_eq!(names, vec!["a", "b"]);
    assert!(expr::idents_in_pass("unicode").is_empty());
}

#[test]
fn full_catalog_validates_clean() {
    let cat = catalog();
    let errs = validate_all(&cat);
    assert!(errs.is_empty(), "unexpected validation errors: {errs:?}");
}

#[test]
fn class_color_resolves_to_palette() {
    let cat = catalog();
    let p = cat.get("progress").unwrap();
    let palette = Theme::Forest.palette();
    // progress.classes.fill = "primary"
    assert_eq!(class_color(p, palette, "fill"), Some(palette.primary));
    assert_eq!(class_color(p, palette, "track"), Some(palette.dim));
    assert_eq!(class_color(p, palette, "bogus"), None);
}

#[test]
fn statusbar_pass_evaluates_against_parent_params() {
    let cat = catalog();
    let s = cat.get("statusbar").unwrap();
    let progress_compose = s
        .compose
        .iter()
        .find(|c| c.child() == Some("progress"))
        .unwrap();

    let mut ctx = ParamCtx::new();
    ctx.insert("percent".into(), Value::Float(0.25));
    ctx.insert("width".into(), Value::Int(60));

    let percent = eval_pass(&progress_compose.pass["percent"], &ctx).unwrap();
    let width = eval_pass(&progress_compose.pass["width"], &ctx).unwrap();
    let style = eval_pass(&progress_compose.pass["style"], &ctx).unwrap();

    assert_eq!(percent, Value::Float(0.25));
    assert_eq!(width, Value::Int(40));
    assert_eq!(style, Value::Str("unicode".into()));
}
