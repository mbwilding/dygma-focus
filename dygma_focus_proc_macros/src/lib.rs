extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(StrEnum)]
pub fn from_str_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("#[derive(StrEnum)] is only defined for enums"),
    };

    let match_arms = variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        quote! { #index => Ok(#name::#variant_name), }
    });

    let value_arms = variants.iter().enumerate().map(|(index, variant)| {
        let variant_name = &variant.ident;
        quote! { #name::#variant_name => #index as u8, }
    });

    let expanded = quote! {
        impl std::str::FromStr for #name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                match s.parse::<u8>() {
                    Ok(num) => match num as usize {
                        #(#match_arms)*
                        _ => Err(anyhow::anyhow!("Invalid value for {}", stringify!(#name))),
                    },
                    Err(_) => Err(anyhow::anyhow!("Invalid value for {}", stringify!(#name))),
                }
            }
        }

        impl #name {
            pub fn value(&self) -> u8 {
                match self {
                    #(#value_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
