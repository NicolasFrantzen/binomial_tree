use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse2, LitInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum UpDown {
    Up,
    Down,
}

impl ToTokens for UpDown {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self {
            UpDown::Up => { quote! { UpDown::Up }}
            UpDown::Down => { quote! { UpDown::Down }}
        };
        tokens.extend(stream);
    }
}

// TODO: Move
struct NodeName {
    name: Vec<UpDown>
}

impl ToTokens for NodeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let stream = quote! {
            NodeName2{ name: &[
                #(#name, )*
            ], direction: None }
        };

        tokens.extend(stream);
    }
}

pub(crate) fn binomial_stack(input: TokenStream) -> TokenStream {
    let number_of_steps = parse2::<LitInt>(input) // LitInt for e.g. 3
        .expect("Must be number")
        .base10_parse::<usize>()
        .expect("Must be number");

    let mut tokens = Vec::<TokenStream>::new();
    for i in 0..=number_of_steps {
        tokens.push(create_level(i));
    }

    quote! {
        &[
            #(#tokens, )*
        ]
    }
}

fn create_level(i: usize) -> TokenStream {
    let iter: Vec<NodeName> =  [UpDown::Up, UpDown::Down]
        .iter()
        .cloned()
        .combinations_with_replacement(i)
        .map(|x| NodeName{ name: x })
        .collect();

    quote! {
        &[
            #(#iter, )*
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proc() {
        println!("{:?}", binomial_stack(quote! { 3 }));

        let expected = quote!{ & [& [NodeName2 { name : & [] , direction : None } ,] , & [NodeName2 { name : & [UpDown :: Up ,] , direction : None } , NodeName2 { name : & [UpDown :: Down ,] , direction : None } ,] , & [NodeName2 { name : & [UpDown :: Up , UpDown :: Up ,] , direction : None } , NodeName2 { name : & [UpDown :: Up , UpDown :: Down ,] , direction : None } , NodeName2 { name : & [UpDown :: Down , UpDown :: Down ,] , direction : None } ,] , & [NodeName2 { name : & [UpDown :: Up , UpDown :: Up , UpDown :: Up ,] , direction : None } , NodeName2 { name : & [UpDown :: Up , UpDown :: Up , UpDown :: Down ,] , direction : None } , NodeName2 { name : & [UpDown :: Up , UpDown :: Down , UpDown :: Down ,] , direction : None } , NodeName2 { name : & [UpDown :: Down , UpDown :: Down , UpDown :: Down ,] , direction : None } ,] ,] };
        assert_eq!(binomial_stack(quote! { 3 }).to_string(), expected.to_string());
    }
}