//! Code formatting utilities for generating well-formatted Rust code.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

/// Format a token stream into a well-formatted Rust code string.
///
/// This function uses prettyplease to format the generated code according
/// to rustfmt defaults.
pub fn format_token_stream(tokens: TokenStream) -> Result<String, syn::Error> {
    // Parse the token stream into a syn::File
    let syntax_tree: syn::File = syn::parse2(tokens)?;

    // Format using prettyplease
    let formatted = prettyplease::unparse(&syntax_tree);

    Ok(formatted)
}

/// Generate a struct definition with the given name and fields.
///
/// # Arguments
/// * `name` - The name of the struct
/// * `fields` - A vector of (field_name, field_type) tuples
/// * `derives` - A vector of trait names to derive
/// * `doc_comment` - Optional documentation comment
///
/// # Returns
/// A formatted Rust struct definition as a string
pub fn generate_struct(
    name: &str,
    fields: Vec<(String, Type)>,
    derives: Vec<&str>,
    doc_comment: Option<&str>,
) -> Result<String, syn::Error> {
    let struct_name = Ident::new(name, proc_macro2::Span::call_site());

    // Build derive attributes
    let derive_idents: Vec<Ident> = derives
        .iter()
        .map(|d| Ident::new(d, proc_macro2::Span::call_site()))
        .collect();

    // Build fields
    let field_defs: Vec<_> = fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
            quote! {
                pub #field_ident: #field_type
            }
        })
        .collect();

    // Build the struct with optional doc comment
    let tokens = if let Some(doc) = doc_comment {
        quote! {
            #[doc = #doc]
            #[derive(#(#derive_idents),*)]
            pub struct #struct_name {
                #(#field_defs),*
            }
        }
    } else {
        quote! {
            #[derive(#(#derive_idents),*)]
            pub struct #struct_name {
                #(#field_defs),*
            }
        }
    };

    format_token_stream(tokens)
}

/// Generate an impl block for a type.
///
/// # Arguments
/// * `type_name` - The name of the type to implement for
/// * `methods` - A vector of method token streams
///
/// # Returns
/// A formatted Rust impl block as a string
pub fn generate_impl_block(
    type_name: &str,
    methods: Vec<TokenStream>,
) -> Result<String, syn::Error> {
    let type_ident = Ident::new(type_name, proc_macro2::Span::call_site());

    let tokens = quote! {
        impl #type_ident {
            #(#methods)*
        }
    };

    format_token_stream(tokens)
}

/// Generate a trait implementation block.
///
/// # Arguments
/// * `trait_name` - The name of the trait to implement
/// * `type_name` - The name of the type to implement for
/// * `methods` - A vector of method token streams
///
/// # Returns
/// A formatted Rust trait impl block as a string
pub fn generate_trait_impl(
    trait_name: &str,
    type_name: &str,
    methods: Vec<TokenStream>,
) -> Result<String, syn::Error> {
    let trait_ident = Ident::new(trait_name, proc_macro2::Span::call_site());
    let type_ident = Ident::new(type_name, proc_macro2::Span::call_site());

    let tokens = quote! {
        impl #trait_ident for #type_ident {
            #(#methods)*
        }
    };

    format_token_stream(tokens)
}

/// Generate an enum definition with the given name and variants.
///
/// # Arguments
/// * `name` - The name of the enum
/// * `variants` - A vector of (variant_name, optional_data_type) tuples
/// * `derives` - A vector of trait names to derive
/// * `doc_comment` - Optional documentation comment
///
/// # Returns
/// A formatted Rust enum definition as a string
pub fn generate_enum(
    name: &str,
    variants: Vec<(String, Option<Type>)>,
    derives: Vec<&str>,
    doc_comment: Option<&str>,
) -> Result<String, syn::Error> {
    let enum_name = Ident::new(name, proc_macro2::Span::call_site());

    // Build derive attributes
    let derive_idents: Vec<Ident> = derives
        .iter()
        .map(|d| Ident::new(d, proc_macro2::Span::call_site()))
        .collect();

    // Build variants
    let variant_defs: Vec<_> = variants
        .iter()
        .map(|(variant_name, data_type)| {
            let variant_ident = Ident::new(variant_name, proc_macro2::Span::call_site());
            if let Some(ty) = data_type {
                quote! { #variant_ident(#ty) }
            } else {
                quote! { #variant_ident }
            }
        })
        .collect();

    // Build the enum with optional doc comment
    let tokens = if let Some(doc) = doc_comment {
        quote! {
            #[doc = #doc]
            #[derive(#(#derive_idents),*)]
            pub enum #enum_name {
                #(#variant_defs),*
            }
        }
    } else {
        quote! {
            #[derive(#(#derive_idents),*)]
            pub enum #enum_name {
                #(#variant_defs),*
            }
        }
    };

    format_token_stream(tokens)
}

/// Generate a type alias.
///
/// # Arguments
/// * `alias_name` - The name of the type alias
/// * `target_type` - The type being aliased
/// * `doc_comment` - Optional documentation comment
///
/// # Returns
/// A formatted Rust type alias as a string
pub fn generate_type_alias(
    alias_name: &str,
    target_type: Type,
    doc_comment: Option<&str>,
) -> Result<String, syn::Error> {
    let alias_ident = Ident::new(alias_name, proc_macro2::Span::call_site());

    let tokens = if let Some(doc) = doc_comment {
        quote! {
            #[doc = #doc]
            pub type #alias_ident = #target_type;
        }
    } else {
        quote! {
            pub type #alias_ident = #target_type;
        }
    };

    format_token_stream(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_format_token_stream() {
        let tokens = quote! {
            pub struct Foo { pub bar: i32, pub baz: String }
        };

        let result = format_token_stream(tokens);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(formatted.contains("pub struct Foo"));
        assert!(formatted.contains("pub bar: i32"));
        assert!(formatted.contains("pub baz: String"));
    }

    #[test]
    fn test_generate_struct_simple() {
        let fields = vec![
            ("name".to_string(), parse_quote!(String)),
            ("age".to_string(), parse_quote!(u32)),
        ];

        let result = generate_struct("Person", fields, vec!["Debug", "Clone"], None);

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("pub struct Person"));
        assert!(code.contains("pub name: String"));
        assert!(code.contains("pub age: u32"));
        assert!(code.contains("#[derive(Debug, Clone)]"));
    }

    #[test]
    fn test_generate_struct_with_doc() {
        let fields = vec![("value".to_string(), parse_quote!(i32))];

        let result = generate_struct(
            "Counter",
            fields,
            vec!["Debug"],
            Some("A simple counter type"),
        );

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("A simple counter type"));
        assert!(code.contains("pub struct Counter"));
    }

    #[test]
    fn test_generate_enum_simple() {
        let variants = vec![
            ("Red".to_string(), None),
            ("Green".to_string(), None),
            ("Blue".to_string(), None),
        ];

        let result = generate_enum("Color", variants, vec!["Debug", "Clone"], None);

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("pub enum Color"));
        assert!(code.contains("Red"));
        assert!(code.contains("Green"));
        assert!(code.contains("Blue"));
    }

    #[test]
    fn test_generate_enum_with_data() {
        let variants = vec![
            ("Some".to_string(), Some(parse_quote!(i32))),
            ("None".to_string(), None),
        ];

        let result = generate_enum("MaybeInt", variants, vec!["Debug"], None);

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("pub enum MaybeInt"));
        assert!(code.contains("Some(i32)"));
        assert!(code.contains("None"));
    }

    #[test]
    fn test_generate_impl_block() {
        let methods = vec![
            quote! {
                pub fn new(value: i32) -> Self {
                    Self { value }
                }
            },
            quote! {
                pub fn get(&self) -> i32 {
                    self.value
                }
            },
        ];

        let result = generate_impl_block("Counter", methods);

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("impl Counter"));
        assert!(code.contains("pub fn new"));
        assert!(code.contains("pub fn get"));
    }

    #[test]
    fn test_generate_trait_impl() {
        let methods = vec![quote! {
            fn default() -> Self {
                Self { value: 0 }
            }
        }];

        let result = generate_trait_impl("Default", "Counter", methods);

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("impl Default for Counter"));
        assert!(code.contains("fn default"));
    }

    #[test]
    fn test_generate_type_alias() {
        let result = generate_type_alias(
            "IntList",
            parse_quote!(Vec<i32>),
            Some("A list of integers"),
        );

        assert!(result.is_ok());
        let code = result.unwrap();

        assert!(code.contains("pub type IntList = Vec<i32>"));
        assert!(code.contains("A list of integers"));
    }
}
