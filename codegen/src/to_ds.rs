use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, ItemFn, PatType, Type};

pub fn impl_datasource(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let name = &input.sig.ident;
    let define = &input.to_token_stream();
    // 获取函数参数列表
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
    let call_args_body = match_call_args(name, &args);
    // 获取 DS 的输入参数列表
    let ds_args = match_ds_args(&args);
    // 获取 Promise 类型
    let typed = look_promise(&args);
    // 获取 Promise 范型类型
    let t = match_promise_t(typed);
    // 获取 函数名称(字符串类型) 用作于 DS 的名称
    let name_str = quote! {#name}.to_string();
    // 获取 Promise 的构建代码
    let promise_body = quote! {
        let mut promise: Promise<#t> = request.head()?;
    };

    // 实现 DS 结构
    let data_source_impl = quote! {
        #define

        pub struct DataSourceImpl{
            register: crate::Register,
        }

        impl DataSourceImpl{
            pub fn new() -> Self{
                Self{
                    register: crate::Register::new()
                }
            }
        }

        impl crate::DataSource for DataSourceImpl{
            fn name(&self) -> &str {
                #name_str
            }
            fn args(&self) -> crate::Columns {
                #ds_args
            }
            fn columns(&self) -> crate::Columns {
                #t::columns()
            }
            fn get_register(&self) -> &crate::Register{
                &self.register
            }
            fn collect(&self, request: &mut crate::Request) -> crate::Result<()> {
                #promise_body
                #call_args_body
                return Ok(());
            }
        }
    };

    TokenStream::from(quote!(#data_source_impl))
}

/// 查找 Promise 参数类型
fn look_promise<'a>(args: &'a Vec<(&Box<Type>, String, &PatType)>) -> &'a Box<Type> {
    args.iter()
        .filter(|arg| {
            let typed = arg.0;
            let type_str = quote! {#typed}.to_string();
            type_str.contains("Promise")
        })
        .map(|arg| arg.0)
        .last()
        .unwrap()
}

/// 获取 Promise 范型 T 类型
fn match_promise_t(type_: &Box<Type>) -> proc_macro2::TokenStream {
    match **type_ {
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
    }
}

/// 获取 DS 输入参数列表
fn match_ds_args(args: &Vec<(&Box<Type>, String, &PatType)>) -> proc_macro2::TokenStream {
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
        .filter(|arg| {
            let typed = arg.2;
            let type_str = quote! {#typed}.to_string();
            !type_str.contains("Arc") && !type_str.contains("Promise")
        })
        .map(|arg| {
            let typed = arg.2;
            let ident = &arg.1;
            let type_ = arg.0;
            quote_spanned! {typed.span() =>
                #type_::get_type() => #ident
            }
        });
    quote! {
        crate::columns![#(#args),*]
    }
}

/// 获取函数调用形式
fn match_call_args(
    name: &Ident,
    args: &Vec<(&Box<Type>, String, &PatType)>,
) -> proc_macro2::TokenStream {
    let mut args_body = vec![];
    let mut index = 0_usize;
    for (type_, _, typed) in args.iter() {
        let type_str = quote! {#typed}.to_string();
        let body = if type_str.contains("Arc") {
            quote_spanned! {typed.span()=>
                self.get_register().get_state::<#type_>()
            }
        } else if type_str.contains("Promise") {
            quote_spanned! {typed.span()=>
                &mut promise
            }
        } else {
            let token = quote_spanned! {typed.span()=>
                request.get_args().get::<#type_>(#index)?
            };
            index += 1;

            token
        };

        args_body.push(body);
    }

    quote! { let _ = #name(#(#args_body),*)?;}
}
