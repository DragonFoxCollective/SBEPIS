use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, parse_quote};

pub fn input_transition_impl(input: TokenStream) -> TokenStream {
    match input_transition(syn::parse_macro_input!(input as InputTransition)) {
        Ok(expr) => expr.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn input_transition(input: InputTransition) -> syn::Result<syn::Expr> {
    match input.arrow {
        TransitionArrow::Right => {
            let left = match input.left {
                LeftTransitionSide::Multiple(ref types) => types,
                LeftTransitionSide::MultipleBack(first, rest) => {
                    if !input.conditions.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &input.conditions[0],
                            "Cannot have conditions with bidirectional transitions",
                        ));
                    }
                    let left = input_transition(InputTransition {
                        action: input.action.clone(),
                        left: LeftTransitionSide::Single(first),
                        arrow: TransitionArrow::Left,
                        right: input.right.clone(),
                        bindings: input.bindings.clone(),
                        conditions: input.conditions.clone(),
                    })?;
                    let right = input_transition(InputTransition {
                        action: input.action.clone(),
                        left: LeftTransitionSide::Multiple(rest),
                        arrow: TransitionArrow::Right,
                        right: input.right.clone(),
                        bindings: input.bindings.clone(),
                        conditions: input.conditions.clone(),
                    })?;
                    return Ok(parse_quote! { ( #left, #right ) });
                }
                LeftTransitionSide::Single(ty) => &vec![ty],
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
            let transition =
                build_right_transition(&input.action, left, right, input.conditions.clone())?;
            Ok(build_output(
                &transition.action,
                &input.bindings,
                &transition.conditions,
                &transition.observers,
            ))
        }
        TransitionArrow::Left => {
            let right = match input.right {
                RightTransitionSide::Multiple(ref types) => types,
                RightTransitionSide::MultipleBack(rest, last) => {
                    if !input.conditions.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &input.conditions[0],
                            "Cannot have conditions with bidirectional transitions",
                        ));
                    }
                    let left = input_transition(InputTransition {
                        action: input.action.clone(),
                        left: input.left.clone(),
                        arrow: TransitionArrow::Left,
                        right: RightTransitionSide::Multiple(rest),
                        bindings: input.bindings.clone(),
                        conditions: input.conditions.clone(),
                    })?;
                    let right = input_transition(InputTransition {
                        action: input.action.clone(),
                        left: input.left.clone(),
                        arrow: TransitionArrow::Right,
                        right: RightTransitionSide::Single(last),
                        bindings: input.bindings.clone(),
                        conditions: input.conditions.clone(),
                    })?;
                    return Ok(parse_quote! { ( #left, #right ) });
                }
                RightTransitionSide::Single(ty) => &vec![ty],
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
            let transition =
                build_left_transition(&input.action, left, right, input.conditions.clone())?;
            Ok(build_output(
                &transition.action,
                &input.bindings,
                &transition.conditions,
                &transition.observers,
            ))
        }
        TransitionArrow::Both => {
            if !input.conditions.is_empty() {
                return Err(syn::Error::new_spanned(
                    &input.conditions[0],
                    "Cannot have conditions with bidirectional transitions",
                ));
            }
            let left = input_transition(InputTransition {
                action: input.action.clone(),
                left: input.left.clone(),
                arrow: TransitionArrow::Left,
                right: input.right.clone(),
                bindings: input.bindings.clone(),
                conditions: input.conditions.clone(),
            })?;
            let right = input_transition(InputTransition {
                action: input.action.clone(),
                left: input.left.clone(),
                arrow: TransitionArrow::Right,
                right: input.right.clone(),
                bindings: input.bindings.clone(),
                conditions: input.conditions.clone(),
            })?;
            Ok(parse_quote! { ( #left, #right ) })
        }
    }
}

fn build_output(
    action: &syn::Type,
    bindings: &[syn::Expr],
    conditions: &[syn::Expr],
    observers: &[syn::Expr],
) -> syn::Expr {
    parse_quote! {
        (
            ::bevy_pretty_nice_input::input!(
                #action,
                [#( #bindings ),*],
                [#( #conditions ),*],
            ),
            #( #observers ),*
        )
    }
}

fn build_filter(from: &[syn::Type]) -> syn::Expr {
    if from.len() == 1 {
        let from = &from[0];
        parse_quote! {
            ::bevy_pretty_nice_input::Filter::<::bevy::prelude::With<#from>>::default()
        }
    } else {
        parse_quote! {
            ::bevy_pretty_nice_input::Filter::<::bevy::prelude::Or<(#( ::bevy::prelude::With<#from> ,)*)>>::default()
        }
    }
}

fn build_observers(
    action: &syn::Type,
    from: &[syn::Type],
    to: &syn::Type,
) -> syn::Result<Vec<syn::Expr>> {
    if from.is_empty() {
        return Err(syn::Error::new_spanned(
            action,
            "Expected at least one 'from' type",
        ));
    }

    Ok(from.iter()
		.map(|f| {
			parse_quote! {
				::bevy_pretty_nice_input::bundles::observe(::bevy_pretty_nice_input::transition::<#action, #f, #to>)
			}
		})
		.collect())
}

struct TransitionOutput {
    action: syn::Type,
    conditions: Vec<syn::Expr>,
    observers: Vec<syn::Expr>,
}

fn build_right_transition(
    action: &syn::Type,
    left: &[syn::Type],
    right: Option<&syn::Type>,
    mut conditions: Vec<syn::Expr>,
) -> syn::Result<TransitionOutput> {
    conditions.insert(0, build_filter(left));
    let observers = if let Some(right) = right {
        build_observers(action, left, right)?
    } else {
        Vec::new()
    };
    Ok(TransitionOutput {
        action: action.clone(),
        conditions,
        observers,
    })
}

fn build_left_transition(
    action: &syn::Type,
    left: Option<&syn::Type>,
    right: &[syn::Type],
    mut conditions: Vec<syn::Expr>,
) -> syn::Result<TransitionOutput> {
    let action = parse_quote! { ::bevy_pretty_nice_input::Off<#action> };
    conditions.insert(0, build_filter(right));
    conditions.insert(0, parse_quote! { ::bevy_pretty_nice_input::Invert });
    let observers = if let Some(left) = left {
        build_observers(&action, right, left)?
    } else {
        Vec::new()
    };
    Ok(TransitionOutput {
        action,
        conditions,
        observers,
    })
}

struct InputTransition {
    action: syn::Type,
    left: LeftTransitionSide,
    arrow: TransitionArrow,
    right: RightTransitionSide,
    bindings: Vec<syn::Expr>,
    conditions: Vec<syn::Expr>,
}

impl Parse for InputTransition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action = input.parse::<syn::Type>()?;
        input.parse::<Token![:]>()?;
        let left = input.parse::<LeftTransitionSide>()?;
        let arrow = input.parse::<TransitionArrow>()?;
        let right = input.parse::<RightTransitionSide>()?;
        let bindings = input.parse::<ExprList>()?.0;
        let conditions = input.parse::<ExprList>().unwrap_or_default().0;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

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
    Both,
    Right,
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

#[derive(Default)]
struct ExprList(Vec<syn::Expr>);

impl Parse for ExprList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![,]>()?;
        let content;
        syn::bracketed!(content in input);
        let exprs = content.parse_terminated(syn::Expr::parse, Token![,])?;
        Ok(ExprList(exprs.into_iter().collect()))
    }
}
