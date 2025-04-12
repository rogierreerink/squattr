use syn::{
    Result,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

pub trait TryParse: Sized {
    /// Try to parse a value without advancing the stream if parsing fails.
    ///
    fn try_parse(input: ParseStream) -> Result<Self>;
}

pub trait TryParseExt {
    fn try_parse<T: Parse>(&self) -> Result<T>;
}

impl TryParseExt for ParseStream<'_> {
    fn try_parse<T: Parse>(&self) -> Result<T> {
        let fork = self.fork();
        match fork.parse() {
            Ok(result) => {
                self.advance_to(&fork);
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }
}
