use proc_macro::TokenStream;

mod derive_action;
mod input;
mod input_transition;

/// Accepts `invalidate = true/false` to impl `EnableFilter = IsInputEnabledInvalidate/IsInputEnabled`
#[proc_macro_derive(Action, attributes(action))]
pub fn derive_action(input: TokenStream) -> TokenStream {
    derive_action::derive_action(input)
}

#[proc_macro]
pub fn input(input: TokenStream) -> TokenStream {
    input::input_impl(input)
}

#[proc_macro]
pub fn input_transition(input: TokenStream) -> TokenStream {
    input_transition::input_transition_impl(input)
}
