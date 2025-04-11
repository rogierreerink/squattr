use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Ident, PathArguments, PathSegment,
    Result, Type, TypePath, parse2,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => expand_struct(input.ident, fields),
        Data::Enum(_) => return Err(Error::new(Span::call_site(), "enums are not supported")),
        Data::Union(_) => return Err(Error::new(Span::call_site(), "unions are not supported")),
    }
}

fn expand_struct(ident: Ident, fields: Fields) -> Result<TokenStream> {
    let fields = match fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
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
    };

    let variables = &fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            if is_option_type(&field.ty) {
                quote! { let mut #ident: #ty = None; }
            } else {
                quote! { let mut #ident: Option<#ty> = None; }
            }
        })
        .collect::<TokenStream>();

    let match_arms = &fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ident_str = ident.to_string();

            quote! {
                id_str if id_str == #ident_str => {
                    #ident.from_value(id_str, value, &mut errors);
                }
            }
        })
        .collect::<TokenStream>();

    let required_checks = &fields
        .iter()
        .filter(|field| is_option_type(&field.ty))
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let error_msg = format!("expected key `{}` not found", ident);

            quote! {
                if #ident.is_none() {
                    errors.push(Error::new(span, #error_msg));
                };
            }
        })
        .collect::<TokenStream>();

    let struct_fields: &TokenStream = &fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();

            if is_option_type(&field.ty) {
                quote! { #ident: #ident.unwrap_or_default(), }
            } else {
                quote! { #ident, }
            }
        })
        .collect::<TokenStream>();

    Ok(quote! {
        impl TryFrom<(Values, Span)> for #ident {
            type Error = Error;

            fn try_from((values, span): (Values, Span)) -> Result<Self> {
                let mut errors = Vec::new();

                #variables

                for value in values {
                    let id = match value.identifier() {
                        Some(id) => id,
                        None => continue,
                    };

                    match id.as_str() {
                        #match_arms

                        id_str => {
                            errors.push(Error::new(
                                value.span(),
                                format!("unrecognized entry `{}`", id_str),
                            ));
                        }
                    }
                }

                #required_checks

                if let Some(error) = errors.combine_errors() {
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
/// of the `std::option::Option` type path. This gives users some flexibility
/// with regard to their type imports (comparted to just checking for `Option`).
/// However, the `Option` type path cannot be renamed.
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
