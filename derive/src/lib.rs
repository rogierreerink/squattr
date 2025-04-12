use proc_macro::TokenStream;
use squattr_core::expand::expand;

#[proc_macro_derive(Squattr)]
pub fn derive_attribute_parser(input: TokenStream) -> TokenStream {
    match expand(input.into()) {
        Ok(token_stream) => token_stream.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
