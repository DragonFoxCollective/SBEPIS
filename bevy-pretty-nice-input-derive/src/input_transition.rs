use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, parse_quote};

use crate::input::{Bindings, Conditions};

pub fn input_transition_impl(input: TokenStream) -> TokenStream {
    match input_transition(syn::parse_macro_input!(input as InputTransition)) {
        Ok(expr) => expr.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn input_transition(input: InputTransition) -> syn::Result<syn::Expr> {
    match input.arrow {
        TransitionArrow::Right => {
            let (left, direction) = match input.left {
                LeftTransitionSide::Multiple(ref types) => (types, ObserverArrow::Right),
                LeftTransitionSide::MultipleBack(ref first, rest) => {
                    if !input.conditions.conditions.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &input.conditions.conditions[0],
                            "Cannot have conditions with bidirectional transitions",
                        ));
                    }
                    (
                        &[first.clone()].into_iter().chain(rest).collect::<Vec<_>>(),
                        ObserverArrow::RightBack(first),
                    )
                }
                LeftTransitionSide::Single(ty) => (&vec![ty], ObserverArrow::Right),
                LeftTransitionSide::Manual => {
                    return Err(syn::Error::new_spanned(
                        &input.left,
                        "Cannot transition from manual",
                    ));
                }
            };
            let right = match input.right {
                RightTransitionSide::Single(ref ty) => Some(ty),
                RightTransitionSide::Multiple(_) | RightTransitionSide::MultipleBack(_, _) => {
                    return Err(syn::Error::new_spanned(
                        &input.right,
                        "Cannot transition to multiple states",
                    ));
                }
                RightTransitionSide::Manual => None,
            };
            let transition = build_transition(
                &input.action,
                left,
                right,
                input.conditions.clone(),
                direction,
            )?;
            Ok(build_output(
                &transition.action,
                &input.bindings,
                &transition.conditions,
                &transition.observers,
            ))
        }
        TransitionArrow::Left => {
            let (right, direction) = match input.right {
                RightTransitionSide::Multiple(ref types) => (types, ObserverArrow::Left),
                RightTransitionSide::MultipleBack(rest, ref last) => {
                    if !input.conditions.conditions.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &input.conditions.conditions[0],
                            "Cannot have conditions with bidirectional transitions",
                        ));
                    }
                    (
                        &rest.into_iter().chain([last.clone()]).collect::<Vec<_>>(),
                        ObserverArrow::LeftBack(last),
                    )
                }
                RightTransitionSide::Single(ty) => (&vec![ty], ObserverArrow::Left),
                RightTransitionSide::Manual => {
                    return Err(syn::Error::new_spanned(
                        &input.right,
                        "Cannot transition from manual",
                    ));
                }
            };
            let left = match input.left {
                LeftTransitionSide::Single(ref ty) => Some(ty),
                LeftTransitionSide::Multiple(_) | LeftTransitionSide::MultipleBack(_, _) => {
                    return Err(syn::Error::new_spanned(
                        &input.left,
                        "Cannot transition to multiple states",
                    ));
                }
                LeftTransitionSide::Manual => None,
            };
            let transition = build_transition(
                &input.action,
                right,
                left,
                input.conditions.clone(),
                direction,
            )?;
            Ok(build_output(
                &transition.action,
                &input.bindings,
                &transition.conditions,
                &transition.observers,
            ))
        }
        TransitionArrow::Both => {
            if !input.conditions.conditions.is_empty() {
                return Err(syn::Error::new_spanned(
                    &input.conditions.conditions[0],
                    "Cannot have conditions with bidirectional transitions",
                ));
            }
            let left = match input.left {
                LeftTransitionSide::Single(ty) => ty,
                LeftTransitionSide::Multiple(_) | LeftTransitionSide::MultipleBack(_, _) => {
                    return Err(syn::Error::new_spanned(
                        &input.left,
                        "Cannot transition to multiple states",
                    ));
                }
                LeftTransitionSide::Manual => {
                    return Err(syn::Error::new_spanned(
                        &input.left,
                        "Cannot transition from manual",
                    ));
                }
            };
            let right = match input.right {
                RightTransitionSide::Single(ty) => ty,
                RightTransitionSide::Multiple(_) | RightTransitionSide::MultipleBack(_, _) => {
                    return Err(syn::Error::new_spanned(
                        &input.right,
                        "Cannot transition to multiple states",
                    ));
                }
                RightTransitionSide::Manual => {
                    return Err(syn::Error::new_spanned(
                        &input.right,
                        "Cannot transition from manual",
                    ));
                }
            };
            let transition = build_transition(
                &input.action,
                std::slice::from_ref(&left),
                Some(&right),
                input.conditions.clone(),
                ObserverArrow::RightBack(&left),
            )?;
            Ok(build_output(
                &transition.action,
                &input.bindings,
                &transition.conditions,
                &transition.observers,
            ))
        }
    }
}

fn build_output(
    action: &syn::Type,
    bindings: &Bindings,
    conditions: &Conditions,
    observers: &[syn::Expr],
) -> syn::Expr {
    parse_quote! {
        (
            ::bevy_pretty_nice_input::input!(
                #action,
                #bindings,
                #conditions,
            ),
            #( #observers ),*
        )
    }
}

fn build_filter(from: &[syn::Type]) -> syn::Expr {
    if from.len() == 1 {
        let from = &from[0];
        parse_quote! {
            ::bevy_pretty_nice_input::InvalidatingFilter::<::bevy::prelude::With<#from>>::default()
        }
    } else {
        parse_quote! {
            ::bevy_pretty_nice_input::InvalidatingFilter::<::bevy::prelude::Or<(#( ::bevy::prelude::With<#from> ,)*)>>::default()
        }
    }
}

fn build_observers(
    action: &syn::Type,
    from: &[syn::Type],
    to: &syn::Type,
    direction: ObserverArrow,
) -> syn::Result<Vec<syn::Expr>> {
    if from.is_empty() {
        return Err(syn::Error::new_spanned(
            action,
            "Expected at least one 'from' type",
        ));
    }

    let transition: syn::Expr = match direction {
        ObserverArrow::Left => parse_quote! { ::bevy_pretty_nice_input::transition_off },
        ObserverArrow::Right => parse_quote! { ::bevy_pretty_nice_input::transition_on },
        ObserverArrow::LeftBack(back) => {
            return Ok([
                build_observers(action, from, to, ObserverArrow::Left)?,
                build_observers(action, std::slice::from_ref(to), back, ObserverArrow::Right)?,
            ]
            .into_iter()
            .flatten()
            .collect());
        }
        ObserverArrow::RightBack(back) => {
            return Ok([
                build_observers(action, from, to, ObserverArrow::Right)?,
                build_observers(action, std::slice::from_ref(to), back, ObserverArrow::Left)?,
            ]
            .into_iter()
            .flatten()
            .collect());
        }
    };

    #[cfg(not(feature = "debug_graph"))]
    return Ok(from
        .iter()
        .map(|f| {
            parse_quote! {
                ::bevy_pretty_nice_input::bundles::observe(#transition::<#action, #f, #to>)
            }
        })
        .collect());
    #[cfg(feature = "debug_graph")]
    return Ok(from
        .iter()
        .flat_map(|f| {
            [
                parse_quote! {
                    ::bevy_pretty_nice_input::bundles::observe(#transition::<#action, #f, #to>)
                },
                parse_quote! {
                    ::bevy_pretty_nice_input::debug_graph::add_graph_edge::<#f, #to, #action>()
                },
            ]
        })
        .collect());
}

#[derive(Clone)]
enum ObserverArrow<'a> {
    Left,
    Right,
    LeftBack(&'a syn::Type),
    RightBack(&'a syn::Type),
}

struct TransitionOutput {
    action: syn::Type,
    conditions: Conditions,
    observers: Vec<syn::Expr>,
}

fn build_transition(
    action: &syn::Type,
    from: &[syn::Type],
    to: Option<&syn::Type>,
    mut conditions: Conditions,
    direction: ObserverArrow,
) -> syn::Result<TransitionOutput> {
    let filters = from.iter().chain(to).cloned().collect::<Vec<_>>();
    conditions.conditions.insert(0, build_filter(&filters));
    let observers =
        if let Some(to) = to {
            build_observers(action, from, to, direction)?
        } else {
            #[cfg(feature = "debug_graph")]
			let empty = from.iter().map(|f| parse_quote! {
				::bevy_pretty_nice_input::debug_graph::add_graph_edge::<#f, #action, #action>()
			}).collect::<Vec<_>>();
            #[cfg(not(feature = "debug_graph"))]
            let empty = vec![];
            empty
        };
    Ok(TransitionOutput {
        action: action.clone(),
        conditions,
        observers,
    })
}

struct InputTransition {
    action: syn::Type,
    left: LeftTransitionSide,
    arrow: TransitionArrow,
    right: RightTransitionSide,
    bindings: Bindings,
    conditions: Conditions,
}

impl Parse for InputTransition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action = input.parse::<syn::Type>()?;
        input.parse::<Token![:]>()?;
        let left = input.parse::<LeftTransitionSide>()?;
        let arrow = input.parse::<TransitionArrow>()?;
        let right = input.parse::<RightTransitionSide>()?;
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

        Ok(InputTransition {
            action,
            left,
            arrow,
            right,
            bindings,
            conditions,
        })
    }
}

#[derive(Clone)]
enum LeftTransitionSide {
    Single(syn::Type),
    Multiple(Vec<syn::Type>),
    MultipleBack(syn::Type, Vec<syn::Type>),
    Manual,
}

impl Parse for LeftTransitionSide {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let types = content.parse_terminated(LeftArrowType::parse, Token![,])?;
            if types.is_empty() {
                Err(syn::Error::new_spanned(
                    &types,
                    "Expected at least one type inside parentheses",
                ))
            } else if let LeftArrowType::ArrowType(ty) = types.first().unwrap().clone() {
                let rest = types
                    .into_iter()
                    .skip(1)
                    .map(|t| match t {
                        LeftArrowType::Type(ty) => Ok(ty),
                        LeftArrowType::ArrowType(_) => Err(syn::Error::new_spanned(
                            &t,
                            "Only the first type can have an arrow",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                Ok(LeftTransitionSide::MultipleBack(ty, rest))
            } else {
                let types = types
                    .into_iter()
                    .map(|t| match t {
                        LeftArrowType::Type(ty) => Ok(ty),
                        LeftArrowType::ArrowType(_) => Err(syn::Error::new_spanned(
                            &t,
                            "Only the first type can have an arrow",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                Ok(LeftTransitionSide::Multiple(types))
            }
        } else if lookahead.peek(syn::Ident) || lookahead.peek(Token![<]) {
            let ty = input.parse::<syn::Type>()?;
            Ok(LeftTransitionSide::Single(ty))
        } else if lookahead.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(LeftTransitionSide::Manual)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for LeftTransitionSide {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            LeftTransitionSide::Single(ty) => {
                ty.to_tokens(tokens);
            }
            LeftTransitionSide::Multiple(types) => {
                tokens.extend(quote! { ( #(#types),* ) });
            }
            LeftTransitionSide::MultipleBack(first, rest) => {
                tokens.extend(quote! { ( #first <= , #(#rest),* ) });
            }
            LeftTransitionSide::Manual => {
                tokens.extend(quote! { * });
            }
        }
    }
}

#[derive(Clone)]
enum LeftArrowType {
    Type(syn::Type),
    ArrowType(syn::Type),
}

impl Parse for LeftArrowType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<syn::Type>()?;
        if input.peek(Token![<=]) {
            input.parse::<Token![<=]>()?;
            Ok(LeftArrowType::ArrowType(ty))
        } else {
            Ok(LeftArrowType::Type(ty))
        }
    }
}

impl ToTokens for LeftArrowType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            LeftArrowType::Type(ty) => {
                ty.to_tokens(tokens);
            }
            LeftArrowType::ArrowType(ty) => {
                tokens.extend(quote! { #ty <= });
            }
        }
    }
}

#[derive(Clone)]
enum TransitionArrow {
    Left,
    Right,
    Both,
}

impl Parse for TransitionArrow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) && input.peek2(Token![=]) && input.peek3(Token![>]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![=]>()?;
            input.parse::<Token![>]>()?;
            Ok(TransitionArrow::Both)
        } else if input.peek(Token![<]) && input.peek2(Token![=]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![=]>()?;
            Ok(TransitionArrow::Left)
        } else if input.peek(Token![=]) && input.peek2(Token![>]) {
            input.parse::<Token![=]>()?;
            input.parse::<Token![>]>()?;
            Ok(TransitionArrow::Right)
        } else {
            Err(input.error("Expected one of `<=`, `=>`, or `<=>`"))
        }
    }
}

impl ToTokens for TransitionArrow {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TransitionArrow::Left => {
                tokens.extend(quote! { <= });
            }
            TransitionArrow::Both => {
                tokens.extend(quote! { <=> });
            }
            TransitionArrow::Right => {
                tokens.extend(quote! { => });
            }
        }
    }
}

#[derive(Clone)]
enum RightTransitionSide {
    Single(syn::Type),
    Multiple(Vec<syn::Type>),
    MultipleBack(Vec<syn::Type>, syn::Type),
    Manual,
}

impl Parse for RightTransitionSide {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let types = content.parse_terminated(RightArrowType::parse, Token![,])?;
            if types.is_empty() {
                Err(syn::Error::new_spanned(
                    &types,
                    "Expected at least one type inside parentheses",
                ))
            } else if let RightArrowType::ArrowType(ty) = types.last().unwrap().clone() {
                let len = types.len();
                let rest = types
                    .into_iter()
                    .take(len - 1)
                    .map(|t| match t {
                        RightArrowType::Type(ty) => Ok(ty),
                        RightArrowType::ArrowType(_) => Err(syn::Error::new_spanned(
                            &t,
                            "Only the last type can have an arrow",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                Ok(RightTransitionSide::MultipleBack(rest, ty))
            } else {
                let types = types
                    .into_iter()
                    .map(|t| match t {
                        RightArrowType::Type(ty) => Ok(ty),
                        RightArrowType::ArrowType(_) => Err(syn::Error::new_spanned(
                            &t,
                            "Only the last type can have an arrow",
                        )),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                Ok(RightTransitionSide::Multiple(types))
            }
        } else if lookahead.peek(syn::Ident) || lookahead.peek(Token![<]) {
            let ty = input.parse::<syn::Type>()?;
            Ok(RightTransitionSide::Single(ty))
        } else if lookahead.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(RightTransitionSide::Manual)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for RightTransitionSide {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            RightTransitionSide::Single(ty) => {
                ty.to_tokens(tokens);
            }
            RightTransitionSide::Multiple(types) => {
                tokens.extend(quote! { ( #(#types),* ) });
            }
            RightTransitionSide::MultipleBack(rest, last) => {
                tokens.extend(quote! { ( #(#rest),* , => #last ) });
            }
            RightTransitionSide::Manual => {
                tokens.extend(quote! { * });
            }
        }
    }
}

#[derive(Clone)]
enum RightArrowType {
    Type(syn::Type),
    ArrowType(syn::Type),
}

impl Parse for RightArrowType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let ty = input.parse::<syn::Type>()?;
            Ok(RightArrowType::ArrowType(ty))
        } else {
            let ty = input.parse::<syn::Type>()?;
            Ok(RightArrowType::Type(ty))
        }
    }
}

impl ToTokens for RightArrowType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            RightArrowType::Type(ty) => {
                ty.to_tokens(tokens);
            }
            RightArrowType::ArrowType(ty) => {
                tokens.extend(quote! { => #ty });
            }
        }
    }
}
