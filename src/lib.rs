use proc_macro2::{Ident, Span};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{braced, parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, token::Brace, Path, Token, Type, TypePath};

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
        Ok(Field {
            ty: input.parse()?,
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

    let u64_type = Type::Path(TypePath {
        qself: None,
        path: Path::from(Ident::new("u64", Span::call_site())),
    });

    let mut ret = proc_macro2::TokenStream::new();
    for slang in slang_structs {
        let SlangStruct {
            _struct_token,
            name,
            _brace_token,
            fields
        } = slang;

        let names: Vec<Ident> = fields.iter().clone().map(|field| field.name.clone()).collect();
        let types: Vec<Type> = fields.iter().clone()
            .map(|field| match field.ty.to_token_stream().to_string().starts_with("Ptr") {
                true => u64_type.clone(),
                false => field.ty.clone()
            }).collect();

        ret.extend(quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
            pub struct #name {
                #(#names: #types),*
            }
        });
    }

    ret.into()
}