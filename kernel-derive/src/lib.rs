use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Constructor)]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate the body depending on the kind of data
    let expanded = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            // struct Foo { a: ..., b: ... }
            Fields::Named(ref fields) => {
                let args = fields.named.iter().map(|f| {
                    let ident = f.ident.as_ref().expect("named field should have an ident");
                    let ty = &f.ty;
                    quote! { #ident: #ty }
                });

                let inits = fields.named.iter().map(|f| {
                    let ident = f.ident.as_ref().unwrap();
                    quote! { #ident }
                });

                quote! {
                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn new(#(#args),*) -> Self {
                            Self { #(#inits),* }
                        }
                    }
                }
            }

            Fields::Unnamed(ref fields) => {
                let args = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let ident = format_ident!("arg{}", i);
                    let ty = &f.ty;
                    quote! { #ident: #ty }
                });

                let inits = fields.unnamed.iter().enumerate().map(|(i, _)| {
                    let ident = format_ident!("arg{}", i);
                    quote! { #ident }
                });

                quote! {
                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn new(#(#args),*) -> Self {
                            Self( #(#inits),* )
                        }
                    }
                }
            }

            Fields::Unit => {
                quote! {
                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn new() -> Self {
                            Self
                        }
                    }
                }
            }
        },

        _ => {
            quote! {
                compile_error!("#[derive(Constructor)] is only supported for structs");
            }
        }
    };

    TokenStream::from(expanded)
}
