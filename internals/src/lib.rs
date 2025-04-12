pub mod ast;
pub mod attributes;
pub mod errors;
pub mod expand;
pub mod parser;
pub mod types;

/// Test utilities.
///
#[cfg(test)]
pub mod tests {
    use proc_macro2::TokenStream;

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
