/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: MIT / Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-test-macros/||VERSION||")]

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn ruspiro_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse_macro_input!(item as ItemFn);

    let test_name = &format!("{}", f.sig.ident.to_string());
    let test_ident = Ident::new(
        &format!("{}_TEST_CONTAINER", f.sig.ident.to_string().to_uppercase()),
        Span::call_site(),
    );
    let test_code_block = f.block;

    quote!(
        #[test_case]
        const #test_ident: UnitTest = UnitTest {
            name: #test_name,
            test_func: || #test_code_block,
        };
    )
    .into()
}
