use proc_macro::TokenStream;
use std::error::Error;
use proc_macro2::{Ident, Span, TokenTree};
use quote::quote;
use syn::{Data, DeriveInput};

macro_rules! error {
    ($e:expr) => {
        return Err($e.into())
    };
}

#[proc_macro_derive(JceStruct, attributes(jce))]
pub fn jce(input: TokenStream) -> TokenStream {
    try_jce(input).unwrap()
}

fn try_jce(input: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let input: DeriveInput = syn::parse(input)?;

    let s = match input.data {
        Data::Struct(s) => s,
        _ => error!("JceStruct can only derive for struct"),
    };

    let (imp_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name = input.ident;

    let default = quote! { Default::default() };
    let fields_default: proc_macro2::TokenStream = s.fields
        .iter()
        .map(|field| {
            let name = &field.ident;
            quote! { #name: #default, }
        }).collect();

    let mut tokens = vec![];
    for field in &s.fields {
        for attr in &field.attrs {
            if attr.path.segments.iter().find(|seg| seg.ident == "jce").is_some() {
                tokens.push(&attr.tokens);
                break;
            }
        }
    }

    if tokens.len() != s.fields.len() {
        error!("field missing attribute");
    }

    let mut v: Vec<(String, u8)> = vec![];

    for token in tokens {
        if let Some(tt) = token.clone().into_iter().next() {
            match tt {
                TokenTree::Group(e) => {
                    let mut stream = e.stream().into_iter();

                    let ty = if let Some(TokenTree::Ident(ident)) = stream.next() {
                        ident.to_string()
                    } else {
                        error!("type wrong");
                    };

                    if let Some(TokenTree::Punct(pun)) = stream.next() {
                        if pun.as_char() == ',' {
                        } else {
                            error!("attribute error");
                        }
                    } else {
                        if stream.next().is_some() {
                            error!("attribute error");
                        }
                    }

                    if let (Some(TokenTree::Ident(ident)), Some(TokenTree::Punct(punct))) = (stream.next(), stream.next()) {
                        if ident == "tag" && punct.as_char() == '=' {

                        } else {
                            error!("tag error");
                        }
                    }

                    let tag = if let Some(TokenTree::Literal(lit)) = stream.next() {
                        let str = lit.to_string();
                        <u8 as std::str::FromStr>::from_str(&str[1..str.len() - 1])?
                    } else {
                        error!("tag error");
                    };

                    v.push((ty, tag));
                },
                _ => error!("wrong attribute"),
            }
        }
    }

    let mut matches = vec![];

    fn primitive(type_name: &str, field: &Option<Ident>) -> proc_macro2::TokenStream {
        let ident = Ident::new(type_name, Span::call_site());
        quote!(::jce::types::#ident::read(buf, t, STRUCT_NAME, stringify!(#field))?)
    }

    for (i, (name, tag)) in v.into_iter().enumerate() {
        let ident = &s.fields.iter().nth(i).unwrap().ident;

        let tag_to = quote!(#tag => );

        matches.push(
            match &*name {
                "int8" => {
                    let pri = primitive("i8", ident);
                    quote!(#tag_to val.#ident = #pri)
                }
                "uint8" => {
                    let pri = primitive("u8", ident);
                    quote!(#tag_to val.#ident = #pri)
                },
                _ => error!("wrong type"),
            }
        )
    }

    Ok(quote! {
        impl #imp_generics ::jce::JceStruct for #name #ty_generics #where_clause {
            fn encode_raw<B: ::jce::bytes::BufMut>(&self, buf: &mut B) {
                todo!()
            }

            fn encoded_len(&self) -> usize {
                todo!()
            }

            fn decode_raw<B: ::jce::bytes::Buf>(
                buf: &mut B,
                to_end: bool,
            ) -> ::jce::error::DecodeResult<Self> {
                const STRUCT_NAME: &str = stringify!(#name);

                let mut val = Self::default();

                let mut t = 0;
                while if to_end {
                    buf.remaining() > 0
                } else {
                    t != ::jce::types::STRUCT_END
                } {
                    let header = ::jce::de::read_header(buf)?;

                    t = header.value_type();

                    match header.tag() {
                        #(#matches),*,
                        _ => ::jce::types::skip_field(buf, t)?,
                    }
                }

                Ok(val)
            }
        }

        impl #imp_generics ::core::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #fields_default
                }
            }
        }
    }.into())
}
