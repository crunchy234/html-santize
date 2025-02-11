extern crate proc_macro;

use proc_macro_crate::{crate_name, FoundCrate};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, TypePath};

/// Recursively iterates through all subtypes and stops if the target type is found.
fn iterate_types(ty: &Type, target: &str) -> bool {
    match ty {
        Type::Path(type_path) => {
            let type_str = type_path.path.segments.last().unwrap().ident.to_string();
            if type_str == target {
                return true; // Stop searching
            }

            // Check generics
            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_type) = arg {
                            if iterate_types(inner_type, target) {
                                return true; // Stop searching
                            }
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            return iterate_types(&type_ref.elem, target);
        }
        Type::Tuple(type_tuple) => {
            for elem in &type_tuple.elems {
                if iterate_types(elem, target) {
                    return true;
                }
            }
        }
        Type::Array(type_array) => {
            return iterate_types(&type_array.elem, target);
        }
        Type::Ptr(type_ptr) => {
            return iterate_types(&type_ptr.elem, target);
        }
        Type::BareFn(type_fn) => {
            for input in &type_fn.inputs {
                if iterate_types(&input.ty, target) {
                    return true;
                }
            }
            if let syn::ReturnType::Type(_, return_ty) = &type_fn.output {
                return iterate_types(return_ty, target);
            }
        }
        Type::TraitObject(type_trait) => {
            for bound in &type_trait.bounds {
                if bound.to_token_stream().to_string() == target {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

#[proc_macro_derive(HtmlSanitize)]
pub fn html_sanitize_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let data = input.data;

    let sanitized = if let Data::Struct(data) = data {
        if let Fields::Named(fields) = data.fields {
            fields.named.iter().map(|named_field| {
                let field_name = named_field.ident.as_ref().expect("All fields must be named");

                match &named_field.ty {
                    Type::Path(TypePath { path, .. }) => {
                        let type_string = path.segments.iter().map(|segment| segment.ident.to_string()).collect::<String>();
                        let contains_string = iterate_types(&named_field.ty, "String");

                        match type_string.as_str() {
                            "String" => quote! {
                                #field_name: sanitize_html_string(&self.#field_name),
                            },
                            "Option" => {
                                if let PathArguments::AngleBracketed(angle_bracketed) = &path.segments.last().unwrap().arguments {
                                    if let Some(GenericArgument::Type(Type::Path(type_path))) = angle_bracketed.args.first() {
                                        let inner_type = type_path.path.segments.iter().map(|segment| segment.ident.to_string()).collect::<String>();
                                        if inner_type == "String" {
                                            return quote! {
                                                #field_name: self.#field_name.as_ref().map(|s| sanitize_html_string(s)),
                                            };
                                        } else if contains_string {
                                            let full_type_string = named_field.ty.to_token_stream().to_string();
                                            panic!("Unhandled field type containing String: {full_type_string}");
                                        } else {
                                            let full_type_string = named_field.ty.to_token_stream().to_string();
                                            let doc_string = format!("Field name: {field_name} type: {type_string} inner_type: {inner_type}, full_string: {full_type_string}");
                                            return quote! {
                                                #[doc = #doc_string]
                                                #field_name: self.#field_name.clone(),
                                            };
                                        }
                                    }
                                }
                                quote! {
                                    #field_name: self.#field_name.clone(),
                                }
                            },
                            "Vec" => {
                                if let PathArguments::AngleBracketed(angle_bracketed) = &path.segments.last().unwrap().arguments {
                                    if let Some(GenericArgument::Type(Type::Path(type_path))) = angle_bracketed.args.first() {
                                        let inner_type = type_path.path.segments.iter().map(|segment| segment.ident.to_string()).collect::<String>();
                                        if inner_type == "String" {
                                            return quote! {
                                                #field_name: self.#field_name.iter().map(|s| sanitize_html_string(s)).collect(),
                                            };
                                        } else if contains_string {
                                            let full_type_string = named_field.ty.to_token_stream().to_string();
                                            panic!("Unhandled field type containing String: {full_type_string}");
                                        }
                                    }
                                }
                                quote! {
                                    #field_name: self.#field_name.clone(),
                                }
                            },
                            _ => {
                                if contains_string {
                                    let full_type_string = named_field.ty.to_token_stream().to_string();
                                    panic!("Unhandled field type containing String: {full_type_string}");
                                }
                                quote! {
                                    #field_name: self.#field_name.clone(),
                                }
                            },
                        }
                    }
                    _ => quote! {
                        #field_name: self.#field_name.clone(),
                    },
                }
            }).collect::<Vec<_>>()
        } else {
            panic!("HTMLSanitize can only be derived for structs with named fields");
        }
    } else {
        panic!("HTMLSanitize can only be derived for structs");
    };

    let main_crate_name = match crate_name("html_sanitize") {
        Ok(name) => match name {
            FoundCrate::Itself => "html_sanitize".to_owned(),
            FoundCrate::Name(name) => name,
        },
        Err(_) => "html_sanitize".to_owned(),
    };
    let main_crate_ident = Ident::new(&main_crate_name, proc_macro2::Span::call_site());

    let expanded = quote! {
        impl #main_crate_ident::HtmlSanitize for #ident {
            fn sanitize(&self) -> Self {
                use #main_crate_ident::{sanitize_html_string, sanitize_html_option_string};
                Self {
                    #(#sanitized)*
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}