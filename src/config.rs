use syntax::codemap::Span;
use syntax::ext::base::TTMacroExpander;
use syntax::parse::token::InternedString;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::ast::TokenTree;
use syntax::parse::token::Token;
use syntax::parse::token::Lit;

use toml;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::error::Error as ErrorTrait;
use std::fmt;

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

fn get_argument(cx: &mut ExtCtxt, span: Span, args: &[TokenTree]) -> Option<InternedString> {
    if args.len() != 1 {
        cx.struct_span_err(span, "config! take one string argument").emit();
        return None;
    }

    if let TokenTree::Token(_, token) = args[0].clone() {
        if let Token::Literal(lit, _) = token {
            if let Lit::Str_(name) = lit {
                return Some(name.as_str());
            }
        }
    }

    cx.struct_span_err(span, "config! take one string argument").emit();
    return None;
}

impl TTMacroExpander for Config {
    fn expand<'cx>(&self, cx: &'cx mut ExtCtxt, span: Span, args: &[TokenTree]) -> Box<MacResult + 'cx> {
        let path = match get_argument(cx, span.clone(), args) {
            Some(path) => path,
            None => {
                return DummyResult::any(span);
            }
        };

        // Convert path to &str
        let path = &*path;
        let value = match self.data.lookup(path) {
            Some(value) => value,
            None => {
                cx.struct_span_err(span, &format!("Could not a value at '{}' in '{}'", path, &*self.filename)).emit();
                return DummyResult::any(span);
            }
        };

        let string = match value.as_str() {
            Some(value) => value,
            None => {
                cx.struct_span_err(span, &format!("The value at {} is not a string", path)).emit();
                return DummyResult::any(span);
            }
        };

        MacEager::expr(quote_expr!(cx, $string))
    }
}
