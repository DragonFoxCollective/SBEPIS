use proc_macro::TokenStream;
use quote::quote;

// #[proc_macro_derive(Actionlike, attributes(control_kind))]
// pub fn derive_actionlike(input: TokenStream) -> TokenStream {
//     let input = syn::parse_macro_input!(input as syn::DeriveInput);
//     let ident = &input.ident;
//     let control_kind = {
//         let control_kind_attr = input
//             .attrs
//             .iter()
//             .find(|attr| attr.path().is_ident("control_kind"))
//             .expect("control_kind attribute is required");
//         let meta = control_kind_attr
//             .parse_args::<syn::Meta>()
//             .expect("Failed to parse control_kind attribute");
//         let syn::Meta::Path(path) = meta else {
//             panic!("control_kind attribute must be a path");
//         };
//         path.get_ident()
//             .expect("control_kind attribute must be a single identifier")
//             .clone()
//     };

//     quote! {
//         impl ::leafwing_input_manager::Action for #ident {
//             fn input_control_kind(&self) -> InputControlKind {
//                 ::leafwing_input_manager::InputControlKind::#control_kind
//             }
//         }
//     }
//     .into()
// }

#[proc_macro_derive(Action)]
pub fn derive_action(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = &input.ident;

    quote! {
        impl ::bevy_pretty_nice_input::Action for #ident {
        }
    }
    .into()
}
