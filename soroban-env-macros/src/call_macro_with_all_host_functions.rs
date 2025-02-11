use itertools::iproduct;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    iter,
};
use syn::{Error, LitStr};

use serde::{Deserialize, Serialize};

use crate::path;

pub fn generate(file_lit: LitStr) -> Result<TokenStream, Error> {
    let file_str = file_lit.value();
    let file_path = path::abs_from_rel_to_manifest(&file_str);

    let file = File::open(&file_path).map_err(|e| {
        Error::new(
            file_lit.span(),
            format!("error reading file '{file_str}': {e}"),
        )
    })?;

    let root: Root = serde_json::from_reader(file).map_err(|e| {
        Error::new(
            file_lit.span(),
            format!("error parsing file '{file_str}': {e}"),
        )
    })?;

    let mut export_names = HashMap::<String, String>::new();
    for m in root.modules.iter() {
        // We expect each module in the env interface to label its function
        // export names according to a simple scheme: _ 0-9 a-z A-Z.
        let exp_chars = iter::once('_')
            .chain('0'..='9')
            .chain('a'..='z')
            .chain('A'..='Z')
            .map(|ch| ch.to_string())
            .collect::<Vec<String>>();

        // This forms the sequence of 1-char names above, followed by the 2-char
        // names formed by the cartesian product of that sequence with itself;
        // enough to cover 4032 functions per module, far more than we'll ever
        // have.
        let max_names = exp_chars.len() + (exp_chars.len() * exp_chars.len());
        let expected_fn_export_names = exp_chars
            .iter()
            .map(|x| x.to_owned())
            .chain(iproduct!(exp_chars.iter(), exp_chars.iter()).map(|(a, b)| a.to_owned() + b));

        if m.functions.len() > max_names {
            return Err(Error::new(
                file_lit.span(),
                format!(
                    "too many functions in module '{}' in '{}': have {}, limit is {}",
                    m.name,
                    file_str,
                    m.functions.len(),
                    max_names
                ),
            ));
        }

        for (f, expected) in m.functions.iter().zip(expected_fn_export_names) {
            let path_name = format!("{}.{}", m.name, f.name);
            let export_name = format!("{}.{}", m.export, f.export);

            if f.export != expected {
                return Err(Error::new(
                    file_lit.span(),
                    format!("unexpected host function export-name in '{file_str}': {path_name} uses '{}' but expected '{}'", f.export, expected),
                ));
            }

            match export_names.entry(export_name.clone()) {
                Entry::Occupied(existing) => {
                    let existing_name = existing.get();
                    return Err(Error::new(
                        file_lit.span(),
                        format!("duplicate host function export-name in '{file_str}': '{export_name}' used by both '{path_name}' and '{existing_name}'"),
                    ));
                }
                Entry::Vacant(v) => {
                    v.insert(path_name);
                }
            }
        }
    }

    // Build the 'mod' sections.
    let modules = root.modules.iter().map(|m| {
        let name = format_ident!("{}", &m.name);
        let export = &m.export;

        // Build the 'fn' sections within the 'mod'.
        let functions = m.functions.iter().map(|f| {
            let docs = f.docs.as_deref().unwrap_or_default();
            let export = &f.export;
            let name = format_ident!("{}", &f.name);

            // Build the args for use within the 'fn'.
            let args = f.args.iter().map(|a| {
                let name = format_ident!("{}", &a.name);
                let r#type = format_ident!("{}", &a.r#type);
                quote! { #name: #r#type }
            });

            let r#return = format_ident!("{}", &f.r#return);

            quote! {
                #[doc = #docs]
                { #export, fn #name(#(#args),*) -> #r#return }
            }
        });

        quote! {
            mod #name #export {
                #(#functions)*
            }
        }
    });

    Ok(quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! _call_macro_with_all_host_functions {

            // The x-macro takes a single ident, the name of a macro to call ...
            {$macro_to_call_back:ident} => {

                // ... and just calls it back, passing a single large token-tree.
                $macro_to_call_back! {

                    // The token-tree we pass to the callback is a sequence of
                    // blocks that have the following structure:
                    //
                    //  mod $mod_id:ident $mod_str:literal {
                    //     ...
                    //     { $fn_str:literal, fn $fn_id:ident $args:tt -> $ret:ty }
                    //     ...
                    //  }
                    //
                    // Where the sub token-tree $args:tt is a normal parenthesized
                    // argument list of comma-separated arg:type pairs

                    #(#modules)*
                }
            };
        }
        pub use _call_macro_with_all_host_functions as call_macro_with_all_host_functions;
    })
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub modules: Vec<Module>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub export: String,
    pub functions: Vec<Function>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub export: String,
    pub name: String,
    pub args: Vec<Arg>,
    pub r#return: String,
    pub docs: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arg {
    pub name: String,
    pub r#type: String,
}
