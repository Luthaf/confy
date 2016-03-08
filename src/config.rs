use syntax::codemap::Span;
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::ext::base::TTMacroExpander;
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::parse::token::InternedString;
use syntax::parse::token;
use syntax::ast::TokenTree;
use syntax::ast;

use toml;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

pub struct Config {
    filename: InternedString,
    data: toml::Value
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    TomlError(String)
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Error::IoError(ref e) => {e.fmt(fmt)},
            &Error::TomlError(ref message) => {write!(fmt, "{}", message)},
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IoError(ref e) => {e.description()},
            &Error::TomlError(ref message) => {&message},
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match self {
            &Error::IoError(ref e) => {e.cause()},
            &Error::TomlError(_) => {None},
        }
    }
}

impl Config {
    pub fn new(path: InternedString) -> Result<Config, Error> {
        let mut file = try!(File::open(&*path));
        let mut buffer = String::new();
        try!(file.read_to_string(&mut buffer));
        let mut parser = toml::Parser::new(&buffer);
        let data = match parser.parse() {
            Some(data) => data,
            None => {
                let mut message = String::from("TOML parsing errors: ");
                for error in &parser.errors {
                    message.push_str(&error.desc);
                    let (line, col) = parser.to_linecol(error.lo);
                    message.push_str(&format!(" at {}:{};", line, col));
                }
                return Err(Error::TomlError(message));
            }
        };

        Ok(Config {
            data: toml::Value::Table(data),
            filename: path
        })
    }
}

fn get_path(cx: &mut ExtCtxt, span: Span, args: &[TokenTree]) -> Option<InternedString> {
    if args.len() == 0 {
        cx.struct_span_err(span, "config! take one string argument").emit();
        return None;
    }

    if let TokenTree::Token(_, token) = args[0].clone() {
        if let token::Token::Literal(lit, _) = token {
            if let token::Lit::Str_(name) = lit {
                return Some(name.as_str());
            }
        }
    }

    cx.struct_span_err(span, "config! take one string argument").emit();
    return None;
}

impl TTMacroExpander for Config {
    fn expand<'cx>(&self, cx: &'cx mut ExtCtxt, span: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
        let path = match get_path(cx, span.clone(), args) {
            Some(path) => path,
            None => {
                return DummyResult::any(span);
            }
        };

        // Convert path to &str
        let path = &*path;
        let value = match self.data.lookup(path) {
            Some(value) => TomlValue(value.clone()),
            None => {
                if args.len() != 3 {
                    cx.struct_span_err(span, &format!("Could not get a value at '{}' in '{}'", path, &*self.filename)).emit();
                    return DummyResult::any(span);
                } else {
                    // args[1] should be a comma
                    let ttre = args[2].clone();
                    // Send default value
                    return MacEager::expr(quote_expr!(cx, $ttre));
                }
            }
        };

        MacEager::expr(quote_expr!(cx, $value))
    }
}

#[derive(Debug)]
struct TomlValue(toml::Value);
impl Deref for TomlValue {
    type Target = toml::Value;
    fn deref(&self) -> &toml::Value {
        &self.0
    }
}

impl ToTokens for TomlValue {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        match self.0.clone() {
            toml::Value::String(value) => value.to_tokens(cx),
            toml::Value::Integer(value) => value.to_tokens(cx),
            toml::Value::Boolean(value) => value.to_tokens(cx),
            toml::Value::Datetime(value) => value.to_tokens(cx),
            toml::Value::Float(value) => {
                let istr = token::intern(&value.to_string()).as_str();
                let lit = ast::Lit{
                    node: ast::LitKind::FloatUnsuffixed(istr),
                    span: DUMMY_SP
                };
                lit.to_tokens(cx)
            },
            toml::Value::Array(value) => {
                let wrapped = value.iter().cloned().map(|t| TomlValue(t)).collect::<Vec<_>>();
                let tokens = wrapped.to_tokens(cx);
                // Create the token tree for a vector
                let mut vector = cx.parse_tts(String::from("vec![1,]"));
                let mut content = match &mut vector[2] {
                    &mut TokenTree::Delimited(_, ref mut content) => Rc::make_mut(content).clone(),
                    _ => panic!("Change in the AST representation")
                };

                // Fill the vector content with the right tokens
                let comma = content.tts[1].clone();
                content.tts.clear();
                for token in tokens {
                    content.tts.push(token);
                    content.tts.push(comma.clone());
                }

                vector[2] = TokenTree::Delimited(DUMMY_SP, Rc::new(content));
                return vector;
            },
            toml::Value::Table(_) => {
                cx.span_err(DUMMY_SP, "Confy can not convert a TOML table to rust");
                return vec![];
            }
        }
    }
}
