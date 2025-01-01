use std::{collections::HashMap, sync::LazyLock};

use proc_macro2::Ident;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{braced, parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, token::Brace, LitStr, Token, Type};

const TYPE_CONVERSION: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("int8_t", "i8"),
        ("uint8_t", "u8"),
        ("int16_t", "i16"),
        ("uint16_t", "u16"),
        ("int32_t", "i32"),
        ("uint32_t", "u32"),
        ("int", "i32"),
        ("uint", "u32"),
        ("int64_t", "i64"),
        ("uint64_t", "u64"),
        ("float", "f32"),

        #[cfg(not(feature = "glam"))]
        ("float2", "[f32; 2]"),
        #[cfg(not(feature = "glam"))]
        ("float3", "[f32; 3]"),
        #[cfg(not(feature = "glam"))]
        ("float4", "[f32; 4]"),
        #[cfg(not(feature = "glam"))]
        ("float4x4", "[f32; 16]"),

        #[cfg(feature = "glam")]
        ("float2", "glam::Vec2"),
        #[cfg(feature = "glam")]
        ("float3", "glam::Vec3"),
        #[cfg(feature = "glam")]
        ("float4", "glam::Vec4"),
        #[cfg(feature = "glam")]
        ("float4x4", "glam::Mat4"),
    ])
});

struct SlangStructArray {
    slang_structs: Vec<SlangStruct>
}

struct SlangStruct {
    _struct_token: Token![struct],
    name: Ident,
    _brace_token: Brace,
    fields: Punctuated<Field, Token![;]>
}

struct Field {
    ty: Type,
    name: Ident
}

impl Parse for SlangStructArray {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut slang_structs = Vec::<SlangStruct>::new();
        while input.peek(Token![struct])  {
            slang_structs.push(input.parse()?);
        }

        Ok(SlangStructArray { slang_structs })
    }
}

impl Parse for SlangStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SlangStruct {
            _struct_token: input.parse()?,
            name: input.parse()?,
            _brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse, Token![;])?
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ty: Type = input.parse()?;

        if input.parse::<Option<Token![*]>>()?.is_some() {
            ty = syn::parse("u64".parse().unwrap()).unwrap()
        } else {
            let mut str = ty.to_token_stream().to_string();
            for (key, value) in TYPE_CONVERSION.iter() {
                let mut r = String::from("([^a-zA-Z0-9]|^)");
                r.push_str(*key);
                r.push_str("([^a-zA-Z0-9]|$)");

                let regex = Regex::new(&r).unwrap();
                str = regex.replace_all(&str, *value).to_string();
            }
            
            ty = syn::parse(str.parse().unwrap()).unwrap()
        }

        Ok(Field {
            ty,
            name: input.parse()?
        })
    }
}

#[proc_macro]
pub fn slang_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SlangStructArray);
    let SlangStructArray {
        slang_structs
    } = input;

    let mut ret = proc_macro2::TokenStream::new();
    for slang in slang_structs {
        let SlangStruct {
            _struct_token,
            name,
            _brace_token,
            fields
        } = slang;

        let names: Vec<Ident> = fields.iter().clone().map(|field| field.name.clone()).collect();
        let types: Vec<Type> = fields.iter().clone().map(|field| field.ty.clone()).collect();

        ret.extend(quote!(#[repr(C)]));
        ret.extend(if cfg!(feature = "bytemuck") {
            quote!(#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)])
        } else {
            quote!(#[derive(Clone, Copy, Default)])
        });

        ret.extend(quote! {
            pub struct #name {
                #(#names: #types),*
            }
        });
    }

    ret.into()
}

#[proc_macro]
pub fn slang_include(input: TokenStream) -> TokenStream {
    let string = parse_macro_input!(input as LitStr).value();
    let file_contents = std::fs::read_to_string(string.as_str()).unwrap();

    slang_struct(file_contents.parse().unwrap())
}