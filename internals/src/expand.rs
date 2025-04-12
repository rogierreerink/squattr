use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, Ident, PathArguments,
    PathSegment, Result, Type, TypePath, parse2, punctuated, spanned::Spanned,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    let ident = input.ident;
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => expand_struct(ident.clone(), fields),
        Data::Enum(_) => Err(Error::new(Span::call_site(), "enums are not supported")),
        Data::Union(_) => Err(Error::new(Span::call_site(), "unions are not supported")),
    }
}

fn expand_struct(ident: Ident, fields: Fields) -> Result<TokenStream> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => expand_named_struct(ident, named.iter()),
        Fields::Unnamed(_) => {
            return Err(Error::new(
                Span::call_site(),
                "unnamed structs are not supported",
            ));
        }
        Fields::Unit => {
            return Err(Error::new(
                Span::call_site(),
                "unit structs are not supported",
            ));
        }
    }
}

fn expand_named_struct(ident: Ident, fields: punctuated::Iter<Field>) -> Result<TokenStream> {
    let mut variables = TokenStream::new();
    let mut match_arms = TokenStream::new();
    let mut required_checks = TokenStream::new();
    let mut struct_fields = TokenStream::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ident_str = ident.to_string();
        let ty = &field.ty;

        match_arms.extend(quote_spanned! {
            ty.span()=>
            id_str if id_str == #ident_str => {
                #ident.insert_value(id_str, value, &mut errors);
            }
        });

        if is_option_type(&field.ty) {
            variables.extend(quote! {
                let mut #ident: #ty = Option::None;
            });

            struct_fields.extend(quote! {
                #ident,
            })
        } else {
            variables.extend(quote! {
                let mut #ident: ::std::option::Option<#ty> = ::std::option::Option::None;
            });

            let error_msg = format!("expected key `{}` not found", ident);
            required_checks.extend(quote! {
                if #ident.is_none() {
                    errors.push(::syn::Error::new(span, #error_msg));
                };
            });

            struct_fields.extend(quote! {
                #ident: #ident.unwrap_or_default(),
            });
        }
    }

    Ok(quote! {
        #[automatically_derived]
        impl ::squattr::attribute::Attribute for #ident {
            fn parse(values: ::squattr::ast::Values, span: ::proc_macro2::Span) -> ::syn::Result<Self> {
                use ::squattr::{
                    errors::CombineErrorsExt,
                    types::ValueStorageExt,
                };

                #variables

                let mut errors = ::std::vec::Vec::new();

                for value in values {
                    let id = match value.identifier() {
                        ::std::option::Option::Some(id) => id,
                        ::std::option::Option::None => continue,
                    };

                    match id.as_str() {
                        #match_arms

                        id_str => {
                            errors.push(::syn::Error::new(
                                value.span(),
                                format!("unrecognized entry `{}`", id_str),
                            ));
                        }
                    }
                }

                #required_checks

                if let ::std::option::Option::Some(error) = errors.combine_errors() {
                    return Err(error);
                }

                Ok(Self {
                    #struct_fields
                })
            }
        }
    })
}

/// Determine wether a type is wrapped in an Option.
///
/// From back to front, the given type needs to completely follow at least part
/// of the `::std::option::Option` type path. This gives users some flexibility
/// with regard to their type imports (comparted to just checking for `Option`).
/// However, the `Option` type path cannot be renamed and the user must make
/// sure that they use the `::std::option::Option` type and not some other type
/// with the same name.
///
fn is_option_type(ty: &Type) -> bool {
    let option_path_segments = [
        PathSegment {
            ident: Ident::new("std", Span::call_site()),
            arguments: PathArguments::None,
        },
        PathSegment {
            ident: Ident::new("option", Span::call_site()),
            arguments: PathArguments::None,
        },
        PathSegment {
            ident: Ident::new("Option", Span::call_site()),
            arguments: PathArguments::None,
        },
    ];

    let ty_path_segments = match ty {
        Type::Path(TypePath { path, .. }) => &path.segments,
        _ => return false,
    };

    ty_path_segments
        .iter()
        .rev()
        .zip(option_path_segments.iter().rev())
        .all(|(ty_path_segment, option_path_segment)| {
            ty_path_segment.ident == option_path_segment.ident
        })
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::expand;

    #[test]
    fn expand_named_struct() {
        let input = quote! {
            struct FooAttribute {
                bar: String,
                baz: Option<bool>,
            }
        };

        let expect = quote! {
            #[automatically_derived]
            impl ::squattr::attribute::Attribute for FooAttribute {
                fn parse(values: ::squattr::ast::Values, span: ::proc_macro2::Span) -> ::syn::Result<Self> {
                    use ::squattr::{
                        errors::CombineErrorsExt,
                        types::ValueStorageExt
                    };

                    let mut bar: ::std::option::Option<String> = ::std::option::Option::None;
                    let mut baz: Option<bool> = Option::None;
                    let mut errors = ::std::vec::Vec::new();

                    for value in values {
                        let id = match value.identifier() {
                            ::std::option::Option::Some(id) => id,
                            ::std::option::Option::None => continue,
                        };
                        match id.as_str() {
                            id_str if id_str == "bar" => {
                                bar.insert_value(id_str, value, &mut errors);
                            }
                            id_str if id_str == "baz" => {
                                baz.insert_value(id_str, value, &mut errors);
                            }
                            id_str => {
                                errors
                                    .push(
                                        ::syn::Error::new(
                                            value.span(),
                                            format!("unrecognized entry `{}`", id_str),
                                        ),
                                    );
                            }
                        }
                    }

                    if bar.is_none() {
                        errors.push(::syn::Error::new(span, "expected key `bar` not found"));
                    }

                    if let ::std::option::Option::Some(error) = errors.combine_errors() {
                        return Err(error);
                    }

                    Ok(Self {
                        bar: bar.unwrap_or_default(),
                        baz,
                    })
                }
            }
        };

        assert_eq_token_streams(&expand(input).unwrap(), &expect);
    }

    /// Pretty compare token streams for equality.
    ///
    pub fn assert_eq_token_streams(a: &TokenStream, b: &TokenStream) {
        let a_str = a.to_string();
        let a_parsed = syn::parse_file(&a_str).unwrap();
        let a_pretty = prettyplease::unparse(&a_parsed);

        let b_str = b.to_string();
        let b_parsed = syn::parse_file(&b_str).unwrap();
        let b_pretty = prettyplease::unparse(&b_parsed);

        pretty_assertions::assert_eq!(a_pretty, b_pretty);
    }
}
