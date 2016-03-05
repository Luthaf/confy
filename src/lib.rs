#![crate_name = "confy"]
#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private, quote)]
extern crate toml;

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use syntax::ast::MetaItemKind;
use syntax::ast::LitKind;
use syntax::codemap::Spanned;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token;
use syntax::parse::token::intern;

use rustc::session::Session;
use rustc_plugin::Registry;

use std::error::Error;

mod config;
use config::Config;

/// Send a fatal error about the plugin call
fn plugin_error(session: &Session, message: &str) {
    session.struct_fatal("Confy plugin declaration must look like: `#![plugin(confy(file=\"file.toml\"))]`").emit();
    session.fatal(message);
}

/// Check arguments of the plugin call, and get the configuration file
fn check_arguments(args: &[syntax::ptr::P<Spanned<MetaItemKind>>], session: &Session) -> token::InternedString {
    if args.len() != 1 {
        plugin_error(session, "There is no argument");
    }

    if let MetaItemKind::NameValue(name, lit) = args[0].clone().unwrap().node {
        if name != "file" {
            plugin_error(session, "Missing `file` argument");
        };

        if let LitKind::Str(value, _) = lit.node {
            return value;
        } else {
            plugin_error(session, "`file` argument is not a string");
        }
    } else {
        plugin_error(session, "Bad plugin call syntax");
    }
    return token::InternedString::new("");
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let file = check_arguments(reg.args(), reg.sess);

    let config = match Config::new(file.clone()) {
        Ok(val) => val,
        Err(err) => {
            reg.sess.fatal(&format!("Error while reading {}: {:?}", file, err.description()));
        }
    };

    let extension = SyntaxExtension::NormalTT(Box::new(config), None, false);
    reg.register_syntax_extension(intern("config"), extension);
}
