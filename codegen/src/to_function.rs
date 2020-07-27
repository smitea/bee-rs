use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, ItemFn};

pub fn impl_function(args: TokenStream,input: ItemFn) -> TokenStream {
    let name = input.sig.ident;
    let attrs = input.attrs;
    let name_str = quote! {#name}.to_string();
    for attr in attrs {
        let attr_st = quote! {#attr}.to_string();
        println!("{}", attr_st);
    }

    let expanded = quote! {
        struct DatasourceImpl;

        impl bee_core::DataSource for DatasourceImpl{
            fn name(&self) -> &str{
                #name_str
            }
            fn args(&self) -> bee_core::Columns{
                bee_core::columns![]
            }
            fn columns(&self) -> bee_core::Columns{
                bee_core::columns![]
            }
            fn collect(&self, request: &mut bee_core::Request) -> bee_core::Result<()>{
                Ok(())
            }
        }
    };
    TokenStream::from(expanded)
}
