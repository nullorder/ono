//! Tiny expression evaluator for `pass = { foo = "${expr}" }` bindings.
//!
//! Input forms supported:
//! * `"literal"` — any string not surrounded by `${…}` is a literal string.
//! * `"${ident}"` — value of a parent param.
//! * `"${width - 20}"` — arithmetic over idents and number literals.
//!
//! Grammar:
//! ```text
//! expr   = term (('+' | '-') term)*
//! term   = factor (('*' | '/') factor)*
//! factor = NUMBER | IDENT | '(' expr ')' | '-' factor
//! ```

use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("unknown identifier `{0}`")]
    UnknownIdent(String),
    #[error("cannot {op} {lhs} and {rhs}")]
    TypeMismatch {
        op: &'static str,
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("division by zero")]
    DivByZero,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unterminated `${{…}}` expression")]
    Unterminated,
}

pub type ParamCtx = BTreeMap<String, Value>;

/// Evaluate a `pass`-style string. Returns `Value::Str` for literals and
/// whatever the inner arithmetic yields for `${…}` expressions.
pub fn eval_pass(input: &str, ctx: &ParamCtx) -> Result<Value, ExprError> {
    if let Some(rest) = input.strip_prefix("${") {
        let inner = rest
            .strip_suffix('}')
            .ok_or(ExprError::Unterminated)?;
        eval_expr(inner, ctx)
    } else {
        Ok(Value::Str(input.to_string()))
    }
}

/// Evaluate a raw expression (no surrounding `${…}`).
pub fn eval_expr(src: &str, ctx: &ParamCtx) -> Result<Value, ExprError> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens: &tokens, pos: 0 };
    let v = p.parse_expr(ctx)?;
    if p.pos != tokens.len() {
        return Err(ExprError::Parse(format!("trailing tokens in `{src}`")));
    }
    Ok(v)
}

/// Collect identifiers referenced inside `${…}` — used for validation.
pub fn idents_in_pass(input: &str) -> Vec<String> {
    let Some(rest) = input.strip_prefix("${") else { return vec![] };
    let Some(inner) = rest.strip_suffix('}') else { return vec![] };
    let mut out = vec![];
    let Ok(tokens) = tokenize(inner) else { return out };
    for t in tokens {
        if let Token::Ident(s) = t {
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

// --- tokenizer ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64, bool), // (value, is_int)
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(src: &str) -> Result<Vec<Token>, ExprError> {
    let bytes = src.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            ' ' | '\t' | '\n' => i += 1,
            '+' => { out.push(Token::Plus); i += 1; }
            '-' => { out.push(Token::Minus); i += 1; }
            '*' => { out.push(Token::Star); i += 1; }
            '/' => { out.push(Token::Slash); i += 1; }
            '(' => { out.push(Token::LParen); i += 1; }
            ')' => { out.push(Token::RParen); i += 1; }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                let mut is_int = true;
                while i < bytes.len() {
                    let ch = bytes[i] as char;
                    if ch == '.' { is_int = false; i += 1; continue; }
                    if ch.is_ascii_digit() { i += 1; continue; }
                    break;
                }
                let lit = &src[start..i];
                let v: f64 = lit
                    .parse()
                    .map_err(|_| ExprError::Parse(format!("bad number `{lit}`")))?;
                out.push(Token::Num(v, is_int));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < bytes.len() {
                    let ch = bytes[i] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' { i += 1; continue; }
                    break;
                }
                out.push(Token::Ident(src[start..i].to_string()));
            }
            other => return Err(ExprError::Parse(format!("unexpected char `{other}`"))),
        }
    }
    Ok(out)
}

// --- parser ------------------------------------------------------------------

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() { self.pos += 1; }
        t
    }

    fn parse_expr(&mut self, ctx: &ParamCtx) -> Result<Value, ExprError> {
        let mut lhs = self.parse_term(ctx)?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Plus => "+",
                Token::Minus => "-",
                _ => break,
            };
            self.bump();
            let rhs = self.parse_term(ctx)?;
            lhs = apply(op, lhs, rhs)?;
        }
        Ok(lhs)
    }

    fn parse_term(&mut self, ctx: &ParamCtx) -> Result<Value, ExprError> {
        let mut lhs = self.parse_factor(ctx)?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Star => "*",
                Token::Slash => "/",
                _ => break,
            };
            self.bump();
            let rhs = self.parse_factor(ctx)?;
            lhs = apply(op, lhs, rhs)?;
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self, ctx: &ParamCtx) -> Result<Value, ExprError> {
        match self.bump() {
            Some(Token::Num(v, is_int)) => Ok(if is_int {
                Value::Int(v as i64)
            } else {
                Value::Float(v)
            }),
            Some(Token::Ident(name)) => ctx
                .get(&name)
                .cloned()
                .ok_or(ExprError::UnknownIdent(name)),
            Some(Token::LParen) => {
                let v = self.parse_expr(ctx)?;
                match self.bump() {
                    Some(Token::RParen) => Ok(v),
                    _ => Err(ExprError::Parse("missing `)`".into())),
                }
            }
            Some(Token::Minus) => {
                let v = self.parse_factor(ctx)?;
                apply("-", Value::Int(0), v)
            }
            other => Err(ExprError::Parse(format!("unexpected token `{other:?}`"))),
        }
    }
}

fn apply(op: &str, lhs: Value, rhs: Value) -> Result<Value, ExprError> {
    let op_name = match op {
        "+" => "add",
        "-" => "subtract",
        "*" => "multiply",
        "/" => "divide",
        _ => unreachable!(),
    };
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => {
                if b == 0 { return Err(ExprError::DivByZero); }
                a / b
            }
            _ => unreachable!(),
        })),
        (a, b) if is_num(&a) && is_num(&b) => {
            let x = to_f64(&a);
            let y = to_f64(&b);
            Ok(Value::Float(match op {
                "+" => x + y,
                "-" => x - y,
                "*" => x * y,
                "/" => {
                    if y == 0.0 { return Err(ExprError::DivByZero); }
                    x / y
                }
                _ => unreachable!(),
            }))
        }
        (a, b) => Err(ExprError::TypeMismatch {
            op: op_name,
            lhs: type_name(&a),
            rhs: type_name(&b),
        }),
    }
}

fn is_num(v: &Value) -> bool {
    matches!(v, Value::Int(_) | Value::Float(_))
}

fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => unreachable!("checked by is_num"),
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        Value::Bool(_) => "bool",
    }
}
