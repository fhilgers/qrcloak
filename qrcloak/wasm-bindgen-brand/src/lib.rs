// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Result};

#[proc_macro_derive(Brand)]
pub fn derive_brand(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    brand_inner(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn brand_inner(input: DeriveInput) -> Result<TokenStream> {
    let name = input.ident.clone();

    let brand_ident = format_ident!(
        "TS_CUSTOM_BRAND_{}",
        name.to_string().to_case(Case::UpperSnake)
    );

    let brand_content = format!("interface {} {{ __ts_brand: \"{}\"; }}", name, name);

    let output = quote! {
        #[automatically_derived]
        const _: () = {
            #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
            const #brand_ident: &'static str = #brand_content;
        };
    };

    Ok(output)
}
