use proc_macro2::{TokenStream, TokenTree};
use quote::TokenStreamExt;

pub struct TokenStreamOrSynError(Result<TokenStream, syn::Error>);
impl TokenStreamOrSynError {
    pub fn new() -> Self {
        TokenStreamOrSynError(Ok(TokenStream::new()))
    }
    pub fn into_result(self) -> Result<TokenStream, syn::Error> {
        self.0
    }
    pub fn append(&mut self, token: impl Into<TokenTree>) {
        if let Ok(ref mut output) = self.0 {
            output.append(token);
        }
    }
    pub fn extend(&mut self, stream: impl IntoIterator<Item = TokenTree>) {
        if let Ok(ref mut output) = self.0 {
            output.extend(stream);
        }
    }
    pub fn error_combine(&mut self, new_err: syn::Error) {
        match self.0 {
            Ok(..) => self.0 = Err(new_err),
            Err(ref mut err) => err.combine(new_err),
        }
    }
}
