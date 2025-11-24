use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, parse_quote};

pub fn input_impl(input: TokenStream) -> TokenStream {
    match input_(syn::parse_macro_input!(input as Input)) {
        Ok(expr) => expr.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn input_(mut input: Input) -> syn::Result<syn::Expr> {
    let action = &input.action;
    input.conditions.conditions.insert(
        0,
        parse_quote!(<#action as ::bevy_pretty_nice_input::Action>::EnableFilter::default()),
    );
    let bindings = build_bindings(action, &input.bindings);
    let conditions = build_conditions(action, &input.conditions);
    let actions = build_actions(action, &input.bindings.dim, &bindings, &conditions);
    let output = parse_quote! {
        (
            #actions,
            ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action_enable::<#action>),
        )
    };
    Ok(output)
}

fn build_actions(
    action: &syn::Type,
    binding_dim: &BindingDim,
    bindings: &syn::Expr,
    conditions: &syn::Expr,
) -> syn::Expr {
    parse_quote! {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Actions<#action>[(
            ::bevy::prelude::Name::new(format!("Action of {}", ::bevy::prelude::ShortName::of::<#action>())),
            ::bevy_pretty_nice_input::PrevActionData(::bevy_pretty_nice_input::ActionData::#binding_dim(Default::default())),
            ::bevy_pretty_nice_input::PrevAction2Data::default(),
            ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action::<#action>),
            ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action_2::<#action>),
            ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::action_2_invalidate::<#action>),

            #bindings,
            #conditions,
        )])
    }
}

fn build_bindings(action: &syn::Type, bindings: &Bindings) -> syn::Expr {
    let bindings = &bindings.bindings;
    parse_quote! {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Bindings[#((
            ::bevy::prelude::Name::new(format!("Binding of {}", ::bevy::prelude::ShortName::of::<#action>())),
            ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::binding),
            ::bevy_pretty_nice_input::BindingParts::spawn(#bindings),
        )),*])
    }
}

fn build_conditions(action: &syn::Type, conditions: &Conditions) -> syn::Expr {
    let conditions = &conditions.conditions;
    parse_quote! {
        ::bevy::prelude::related!(::bevy_pretty_nice_input::Conditions[#((
            ::bevy::prelude::Name::new(format!("Condition of {}", ::bevy::prelude::ShortName::of::<#action>())),
            {
                use ::bevy_pretty_nice_input::Condition;
                let condition = #conditions;
                (
                    condition.bundle::<#action>(),
                    condition,
                    ::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::invalidate_pass),
                )
            }
        )),*])
    }
}

struct Input {
    action: syn::Type,
    bindings: Bindings,
    conditions: Conditions,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action = input.parse::<syn::Type>()?;
        input.parse::<Token![,]>()?;
        let bindings = input.parse::<Bindings>()?;
        let conditions = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let conditions = input.parse::<Conditions>().unwrap_or_default();
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            conditions
        } else {
            Conditions::default()
        };

        Ok(Input {
            action,
            bindings,
            conditions,
        })
    }
}

#[derive(Clone)]
pub struct Bindings {
    pub dim: BindingDim,
    pub bindings: Vec<syn::Expr>,
}

impl Parse for Bindings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dim = input.parse::<BindingDim>()?;
        let content;
        syn::bracketed!(content in input);
        let bindings = content
            .parse_terminated(syn::Expr::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Bindings { dim, bindings })
    }
}

impl ToTokens for Bindings {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let dim = &self.dim;
        let bindings = &self.bindings;
        tokens.extend(quote! {
            #dim [ #( #bindings ),* ]
        });
    }
}

#[derive(Clone)]
pub enum BindingDim {
    Axis1D,
    Axis2D,
    Axis3D,
}

impl Parse for BindingDim {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        match ident.to_string().as_str() {
            "Axis1D" => Ok(BindingDim::Axis1D),
            "Axis2D" => Ok(BindingDim::Axis2D),
            "Axis3D" => Ok(BindingDim::Axis3D),
            _ => Err(syn::Error::new_spanned(
                ident,
                "Expected one of `Axis1D`, `Axis2D`, or `Axis3D`",
            )),
        }
    }
}

impl ToTokens for BindingDim {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            BindingDim::Axis1D => tokens.extend(quote! { Axis1D }),
            BindingDim::Axis2D => tokens.extend(quote! { Axis2D }),
            BindingDim::Axis3D => tokens.extend(quote! { Axis3D }),
        }
    }
}

#[derive(Clone, Default)]
pub struct Conditions {
    pub conditions: Vec<syn::Expr>,
}

impl Parse for Conditions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let conditions = content
            .parse_terminated(syn::Expr::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Conditions { conditions })
    }
}

impl ToTokens for Conditions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let conditions = &self.conditions;
        tokens.extend(quote! {
            [ #( #conditions ),* ]
        });
    }
}
