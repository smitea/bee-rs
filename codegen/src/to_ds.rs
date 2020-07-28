use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn, PatType, Type};

pub fn impl_datasource(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let name = &input.sig.ident;
    let define = &input.to_token_stream();
    let args: Vec<(&Box<Type>, String, &PatType)> = input
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
        .collect();

    // 获取函数调用参数
    let args_body: Vec<proc_macro2::TokenStream> = args
        .iter()
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
                        request.get_args().get::<#type_>(#index)?
                    }
                }
            }
        })
        .collect();

    let args_body = if (&args_body).is_empty() {
        quote! { let _ = #name(&mut promise)?;}
    } else {
        quote! { let _ = #name(#(#args_body),*,&mut promise)?;}
    };
    
    let t = args
        .iter()
        .filter(|arg| {
            let type_ = arg.0;
            if let syn::Type::Reference(_) = **type_ {
                true
            } else {
                false
            }
        })
        .last()
        .unwrap();

    // 获取范型 T
    let type_ = t.0;
    let t = match **type_ {
        syn::Type::Reference(ref reference) => {
            let typed = &reference.elem;

            match **typed {
                Type::Path(ref token) => {
                    let typed = token.path.segments.last().unwrap();
                    let arg = &typed.arguments;

                    match arg {
                        syn::PathArguments::AngleBracketed(angle) => {
                            let typed = angle.args.last();
                            quote_spanned! {arg.span() =>
                                #typed
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };

    // 构建 Promise
    let promise_body = quote! {
        let mut promise: Promise<#t> = request.head()?;
    };

    // DS 参数
    let args = args
        .iter()
        .filter(|arg| {
            let type_ = arg.0;
            if let syn::Type::Reference(_) = **type_ {
                false
            } else {
                true
            }
        })
        .map(|arg| {
            let typed = arg.2;
            let ident = &arg.1;
            let type_ = arg.0;
            quote_spanned! {typed.span() =>
                #type_::get_type() => #ident
            }
        });
    let args = quote! {
        crate::columns![#(#args),*]
    };

    // 实现 DS
    let name_str = quote! {#name}.to_string();
    let data_source_impl = quote! {
        #define

        pub struct DataSourceImpl;

        impl crate::DataSource for DataSourceImpl{
            fn name(&self) -> &str {
                #name_str
            }
            fn args(&self) -> crate::Columns {
                #args
            }
            fn columns(&self) -> crate::Columns {
                #t::columns()
            }
            fn collect(&self, request: &mut crate::Request) -> crate::Result<()> {
                #promise_body
                #args_body
                return Ok(());
            }
        }
    };

    TokenStream::from(quote!(#data_source_impl))
}
