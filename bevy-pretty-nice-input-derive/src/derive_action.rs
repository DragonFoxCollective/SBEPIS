use proc_macro::TokenStream;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Expr, ExprLit, Lit, Token, Type, parse_quote};

pub fn derive_action(input: TokenStream) -> TokenStream {
    match action(syn::parse_macro_input!(input as DeriveInput)) {
        Ok(expr) => expr.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn action(input: DeriveInput) -> syn::Result<syn::ItemImpl> {
    let ident = &input.ident;

    let attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("action"));
    let meta_list = attr
        .map(|attr| attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated))
        .transpose()?;

    let mut invalidate = None;

    if let Some(meta_list) = meta_list {
        for item in meta_list {
            match item {
                Expr::Assign(assign)
                    if assign.left.to_token_stream().to_string() == "invalidate" =>
                {
                    if invalidate.is_some() {
                        return Err(syn::Error::new_spanned(
                            assign.left,
                            "Duplicate `invalidate` attribute",
                        ));
                    }

                    match *assign.right {
                        Expr::Lit(ExprLit {
                            lit: Lit::Bool(bool),
                            ..
                        }) => {
                            invalidate = Some(bool.value);
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                assign.right,
                                "Expected a boolean literal",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(item, "Unexpected item"));
                }
            }
        }
    }

    let enable_filter: Type = if invalidate.unwrap_or(true) {
        parse_quote!(::bevy_pretty_nice_input::IsInputEnabledInvalidate)
    } else {
        parse_quote!(::bevy_pretty_nice_input::IsInputEnabled)
    };

    Ok(parse_quote! {
        impl ::bevy_pretty_nice_input::Action for #ident {
            type EnableFilter = #enable_filter;
        }
    })
}
