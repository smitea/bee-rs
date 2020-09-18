use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod to_data;
mod to_ds;
mod to_function;

#[cfg(not(tarpaulin_include))]
#[proc_macro_attribute]
pub fn function(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    to_function::impl_function(args, input)
}

#[cfg(not(tarpaulin_include))]
#[proc_macro_attribute]
pub fn datasource(args: TokenStream, input: TokenStream) -> TokenStream {
    to_ds::impl_datasource(args, input)
}

#[cfg(not(tarpaulin_include))]
#[proc_macro_derive(Data)]
pub fn data(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    to_data::impl_to_data(input)
}
