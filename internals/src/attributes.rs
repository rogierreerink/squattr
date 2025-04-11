use proc_macro2::{Span, TokenStream};
use syn::{Error, parse::ParseStream};

use crate::ast::Values;

pub trait ParseAttributeExt: Sized {
    fn parse_attribute<T: TryFrom<(Values, Span), Error = Error>>(self) -> syn::Result<T>;
}

impl ParseAttributeExt for TokenStream {
    fn parse_attribute<T: TryFrom<(Values, Span), Error = Error>>(self) -> syn::Result<T> {
        syn::parse::Parser::parse2(
            |input: ParseStream| T::try_from((input.parse()?, input.span())),
            self,
        )
    }
}
