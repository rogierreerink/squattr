use proc_macro2::{Span, TokenStream};
use syn::{Error, Meta, MetaList, MetaNameValue, Path, Result, parse::ParseStream};

use crate::{
    ast::{List, Value, Values},
    errors::ErrorsExt,
    types::{ParseValue, format_error},
};

pub trait Attribute: Sized {
    fn from_values(values: Values) -> Result<Self>;

    fn from_input(input: ParseStream) -> Result<Self> {
        Self::from_values(input.parse()?)
    }

    fn from_tokens(tokens: TokenStream) -> Result<Self> {
        syn::parse::Parser::parse2(|input: ParseStream| Self::from_input(input), tokens)
    }

    fn from_meta(meta: &Meta) -> Result<Self> {
        match meta {
            Meta::List(MetaList { tokens, .. }) => Self::from_tokens(tokens.clone()),
            Meta::NameValue(MetaNameValue { .. }) => Err(Error::new(
                Span::call_site(),
                "meta name values are not supported",
            )),
            Meta::Path(Path { .. }) => Err(Error::new(
                Span::call_site(),
                "meta paths are not supported",
            )),
        }
    }

    fn extract_from_attributes(
        attributes: &mut Vec<syn::Attribute>,
        path: &str,
    ) -> Result<Vec<Self>> {
        let mut errors = Vec::new();

        let parsed = attributes
            .iter()
            .filter_map(|attr| {
                if !attr.path().is_ident(path) {
                    return None;
                }

                match Self::from_meta(&attr.meta) {
                    Ok(attr) => Some(attr),
                    Err(error) => {
                        errors.push(error);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        attributes.retain(|attr| !attr.path().is_ident(path));

        if let Some(error) = errors.combine() {
            return Err(error);
        }

        Ok(parsed)
    }
}

impl<T> ParseValue for T
where
    T: Attribute,
{
    fn parse(value: Value) -> Result<Self> {
        T::from_values(match value {
            Value::List(List { values, .. }) => values,
            value => return Err(format_error(&value, "list of values")),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::Values, attribute::Attribute, errors::ErrorsExt, types::ValueStorageExt};

    use proc_macro2::Span;
    use quote::quote;
    use syn::{Ident, Lit, LitInt};

    #[test]
    fn parse_attributes() {
        #[derive(PartialEq, Debug)]
        struct SomeAttribute {
            some_list: Vec<String>,
            some_ident_list: Vec<Ident>,
            some_bool: bool,
            some_expr: Option<String>,
            some_ident: Option<Ident>,
            some_lit: Option<Lit>,
            some_sub_attr: Option<SubAttribute>,
        }

        impl Attribute for SomeAttribute {
            fn from_values(values: Values) -> syn::Result<Self> {
                let span = values.span();
                let mut errors = Vec::new();

                let mut some_list: Option<Vec<String>> = None;
                let mut some_ident_list: Option<Vec<Ident>> = None;
                let mut some_bool: Option<bool> = None;
                let mut some_expr: Option<String> = None;
                let mut some_ident: Option<Ident> = None;
                let mut some_lit: Option<Lit> = None;
                let mut some_sub_attr: Option<SubAttribute> = None;

                for value in values {
                    let id = match value.identifier() {
                        Some(id) => id,
                        None => {
                            errors.push(syn::Error::new(
                                value.span(),
                                format!("expected an identifier"),
                            ));
                            continue;
                        }
                    };
                    match id.as_str() {
                        id_str if id_str == "some_list" => {
                            some_list.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_ident_list" => {
                            some_ident_list.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_bool" => {
                            some_bool.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_expr" => {
                            some_expr.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_ident" => {
                            some_ident.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_lit" => {
                            some_lit.insert_value(id_str, value, &mut errors);
                        }
                        id_str if id_str == "some_sub_attr" => {
                            some_sub_attr.insert_value(id_str, value, &mut errors);
                        }
                        id_str => {
                            errors.push(syn::Error::new(
                                value.span(),
                                format!("unrecognized key `{}`", id_str),
                            ));
                        }
                    }
                }

                if some_list.is_none() {
                    errors.push(syn::Error::new(span, "expected key `some_list` not found"));
                }
                if some_ident_list.is_none() {
                    errors.push(syn::Error::new(
                        span,
                        "expected key `some_ident_list` not found",
                    ));
                }

                if let Some(error) = errors.combine() {
                    return Err(error);
                }

                Ok(Self {
                    some_list: some_list.expect("values existance has already been confirmed"),
                    some_ident_list: some_ident_list
                        .expect("values existance has already been confirmed"),
                    some_bool: some_bool.unwrap_or_default(),
                    some_expr,
                    some_ident,
                    some_lit,
                    some_sub_attr,
                })
            }
        }

        #[derive(PartialEq, Debug)]
        struct SubAttribute {
            some_sub_bool: bool,
        }

        impl Attribute for SubAttribute {
            fn from_values(values: Values) -> syn::Result<Self> {
                let _span = values.span();
                let mut errors = Vec::new();

                let mut some_sub_bool: Option<bool> = None;

                for value in values {
                    let id = match value.identifier() {
                        Some(id) => id,
                        None => continue,
                    };
                    match id.as_str() {
                        id_str if id_str == "some_sub_bool" => {
                            some_sub_bool.insert_value(id_str, value, &mut errors);
                        }
                        id_str => {
                            errors.push(syn::Error::new(
                                value.span(),
                                format!("unrecognized key `{}`", id_str),
                            ));
                        }
                    }
                }

                if let Some(error) = errors.combine() {
                    return Err(error);
                }

                Ok(Self {
                    some_sub_bool: some_sub_bool.unwrap_or_default(),
                })
            }
        }

        let input = quote! {
            some_list("lit1", "lit2"),
            some_ident_list(id1, id2),
            some_bool,
            some_expr = "foo",
            some_ident,
            some_lit = 123,
            some_sub_attr(
                some_sub_bool = false
            ),
        };

        assert_eq!(
            SomeAttribute::from_tokens(input).expect("values existance has already been confirmed"),
            SomeAttribute {
                some_list: vec!["lit1".into(), "lit2".into()],
                some_ident_list: vec![
                    Ident::new("id1", Span::call_site()),
                    Ident::new("id2", Span::call_site())
                ],
                some_bool: true,
                some_expr: Some("foo".into()),
                some_ident: Some(Ident::new("some_ident", Span::call_site())),
                some_lit: Some(Lit::Int(LitInt::new("123", Span::call_site()))),
                some_sub_attr: Some(SubAttribute {
                    some_sub_bool: false
                }),
            }
        );
    }
}
