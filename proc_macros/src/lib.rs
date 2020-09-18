// SPDX-License-Identifier: GPL-3.0-or-later

use {
    inflector::cases::pascalcase::to_pascal_case,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::quote,
    syn::{
        braced,
        parse::{Parse, ParseStream, Result},
        parse_macro_input,
        punctuated::Punctuated,
        token::Brace,
        ExprRange, Ident, Token, Type, Visibility,
    },
};

struct Register {
    visibility: Visibility,
    struct_token: Token![struct],
    name: Ident,
    colon_token: Token![:],
    ty: Type,
    brace_token: Brace,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility = input.parse()?;
        let struct_token = input.parse()?;
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;

        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            visibility,
            struct_token,
            name,
            colon_token,
            ty,
            brace_token,
            fields: content.parse_terminated(Field::parse)?,
        })
    }
}

struct Field {
    name: Ident,
    colon_token: Token![:],
    range: ExprRange,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon_token: input.parse()?,
            range: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn add_register_type(stream: TokenStream) -> TokenStream {
    let Register {
        visibility,
        struct_token: _,
        name,
        colon_token: _,
        ty,
        brace_token: _,
        fields,
    } = parse_macro_input!(stream as Register);

    let enum_name = Ident::new(
        &format!("{}{}", to_pascal_case(&name.to_string()), "Field"),
        Span::call_site(),
    );

    let enum_variants = fields
        .iter()
        .map(|field| Ident::new(&to_pascal_case(&field.name.to_string()), Span::call_site()))
        .collect::<Vec<_>>();

    let bit_range = fields.iter().map(|field| &field.range).collect::<Vec<_>>();

    let expanded = quote! {
        #visibility struct #name;
        impl #name{
            #visibility fn get(base_addr:x86_64::PhysAddr,field:#enum_name)->#ty{
                let raw=Self::fetch_raw(base_addr);
                match field{
                    #(#enum_name::#enum_variants => raw.bit_range(#bit_range),)*
                }
            }

            #visibility fn set(base_addr:x86_64::PhysAddr,field:#enum_name,value:#ty){
                let mut raw=Self::fetch_raw(base_addr);
                let val=match field{
                    #(#enum_name::#enum_variants => raw.set_bit_range(#bit_range,value),)*
                };
                crate::mem::allocator::virt::map_to_phys_temporary(base_addr,|virt_addr| unsafe{
                    core::ptr::write(virt_addr.as_mut_ptr(),*val)}
                );
            }

            #visibility fn fetch_raw(addr:x86_64::PhysAddr)->#ty{
                crate::mem::allocator::virt::map_to_phys_temporary(addr,|virt_addr| unsafe{
                    core::ptr::read(virt_addr.as_mut_ptr())}
                )
            }

        }

        #visibility enum #enum_name{
            #(#enum_variants,)*
        }
    };

    TokenStream::from(expanded)
}
