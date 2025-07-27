use proc_macro::TokenStream;

#[proc_macro_derive(Entity, attributes(compose, comprise))]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    todo!()
}