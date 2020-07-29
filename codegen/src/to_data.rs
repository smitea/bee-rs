use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};
use proc_macro::TokenStream;

pub fn impl_to_data(input: DeriveInput) -> TokenStream{
    let name = input.ident;
    
    let columns = match input.data{
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let col = fields.named.iter().map(|field|{
                        let ident = &field.ident;
                        let type_ = &field.ty;
                        
                        let name_str = quote! {#ident}.to_string();
                        quote_spanned! {field.span() =>
                            #type_::get_type() => #name_str
                        }
                    });
                    
                    quote! {
                        crate::columns![#(#col),*]
                    }
                }
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    };

    let row = match input.data{
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let col = fields.named.iter().map(|field|{
                        let ident = &field.ident;
                        
                        quote_spanned! {field.span() =>
                            self.#ident
                        }
                    });
                    
                    quote! {
                        crate::row![#(#col),*]
                    }
                }
                // 不支持非命名属性
                _ => unimplemented!()
            }
        },
        // 不支持非结构体类型
        _ => unimplemented!()
    };

    let expanded = quote! {        
        impl crate::ToData for #name{
            fn columns() -> Columns{
                use crate::ToType;
                #columns
            }
            fn to_row(self) -> Row{
                #row
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}