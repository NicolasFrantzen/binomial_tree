use proc_macro::TokenStream;
use syn::parse_macro_input;

mod proc_impl;

#[proc_macro]
pub fn binomial_tree_stack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    proc_impl::binomial_stack(input).into()
}
