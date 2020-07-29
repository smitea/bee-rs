use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::ItemFn;

pub fn impl_function(_args: TokenStream, input: ItemFn) -> TokenStream {
    let name = &input.sig.ident;
    let define = &input.to_token_stream();
    // 获取函数调用参数
    let args_body: Vec<proc_macro2::TokenStream> = input
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => unimplemented!(),
            syn::FnArg::Typed(typed) => {
                let pat = &typed.pat;
                let type_ = &typed.ty;
                let ident = quote! {#pat}.to_string();

                (type_, ident, typed)
            }
        })
        .enumerate()
        .filter(|(_, arg)| {
            let type_ = arg.0;
            if let syn::Type::Reference(_) = **type_ {
                false
            } else {
                true
            }
        })
        .map(|(index, arg)| {
            let type_ = arg.0;
            let typed = arg.2;
            match **type_ {
                syn::Type::Reference(_) => unimplemented!(),
                _ => {
                    quote_spanned! {typed.span()=>
                        args.get::<#type_>(#index)?
                    }
                }
            }
        })
        .collect();

    let args_size = args_body.len();
    let args_body = quote! { let value = #name(#(#args_body),*)?; return Ok(value.into())};

    // 实现 DS
    let name_str = quote! {#name}.to_string();
    let function_impl = quote! {
        #define

        pub struct FunctionImpl;

        impl FunctionImpl {
            pub fn name() -> &'static str {
                #name_str
            }

            pub fn args() -> usize{
                #args_size
            }

            pub fn invoke(args: &crate::Args) -> Result<Value> {
                #args_body
            }
        }
    };

    TokenStream::from(quote!(#function_impl))
}
