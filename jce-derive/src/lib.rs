use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use std::error::Error;
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
    let mut fields_default: Vec<proc_macro2::TokenStream> = vec![];

    let mut fields_encoded_len: Vec<proc_macro2::TokenStream> = vec![];

    let mut tags: Vec<u8> = vec![];

    let mut tag: i32 = -1;
    for field in &s.fields {
        let ident = &field.ident;
        fields_default.push(quote!(#ident: #default));

        fields_encoded_len.push(quote!(::jce::types::JceType::write_len(&self.#ident)));

        if field.attrs.is_empty() {
            tag += 1;
            tags.push(tag as u8);
            continue;
        }

        for attr in &field.attrs {
            if attr
                .path
                .segments
                .iter()
                .find(|seg| seg.ident == "jce")
                .is_some()
            {
                if let Some(tt) = attr.tokens.clone().into_iter().next() {
                    match tt {
                        TokenTree::Group(e) => {
                            let mut stream = e.stream().into_iter();

                            match (stream.next(), stream.next()) {
                                (Some(TokenTree::Ident(ident)), Some(TokenTree::Punct(punct))) => {
                                    if ident != "tag" || punct.as_char() != '=' {
                                        error!("tag error");
                                    }
                                }
                                _ => error!("attribute error"),
                            }

                            tag = if let Some(TokenTree::Literal(lit)) = stream.next() {
                                let str = lit.to_string();
                                <u8 as std::str::FromStr>::from_str(&str[1..str.len() - 1])?
                            } else {
                                error!("tag error");
                            } as i32;

                            tags.push(tag as u8);
                        }
                        _ => error!("wrong attribute"),
                    }
                }
                break;
            }
        }
    }

    let mut matches = vec![];
    let mut encodes = vec![];

    let tags_encoded_len: usize = tags.iter().map(|tag|
        if *tag < 0xF {
            1
        } else { 2 }
    ).sum();

    for (i, tag) in tags.into_iter().enumerate() {
        let ident = &s.fields.iter().nth(i).unwrap().ident;

        let tag_to = quote!(#tag => );
        let read = quote!(::jce::types::JceType::read(
            buf,
            t,
            STRUCT_NAME,
            stringify!(#ident)
        )?);

        matches.push(quote!(#tag_to val.#ident = #read));
        encodes.push(quote!(::jce::types::JceType::write(&self.#ident, buf, #tag)));
    }

    Ok(quote! {
        impl #imp_generics ::jce::JceStruct for #name #ty_generics #where_clause {
            fn encode_raw<B: ::jce::bytes::BufMut>(&self, buf: &mut B) {
                #(#encodes);*;
            }

            fn encoded_len(&self) -> usize {
                #tags_encoded_len + #(#fields_encoded_len)+*
            }

            fn decode_raw<B: ::jce::bytes::Buf>(
                buf: &mut B,
                to_end: bool,
            ) -> ::jce::error::DecodeResult<Self> {
                const STRUCT_NAME: &str = stringify!(#name);

                let mut val = Self::default();

                let mut t = 0;
                while buf.remaining() > 0 {
                    let header = ::jce::de::read_header(buf)?;

                    t = header.value_type();
                    if !to_end && t == ::jce::types::STRUCT_END {
                        break;
                    }

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
                    #(#fields_default),*,
                }
            }
        }
    }
    .into())
}
