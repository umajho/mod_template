use proc_macro2::{TokenStream, TokenTree};
use quote::TokenStreamExt;

pub struct TokenStreamOrSynErrors(Result<TokenStream, Vec<syn::Error>>);
impl TokenStreamOrSynErrors {
    pub fn new() -> Self {
        TokenStreamOrSynErrors(Ok(TokenStream::new()))
    }
    pub fn into_result(self) -> Result<TokenStream, Vec<syn::Error>> {
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
    pub fn push_error(&mut self, err: syn::Error) {
        match self.0 {
            Ok(..) => self.0 = Err(vec![err]),
            Err(ref mut errors) => errors.push(err),
        }
    }
    pub fn extend_errors(&mut self, errs: impl IntoIterator<Item = syn::Error>) {
        match self.0 {
            Ok(..) => self.0 = Err(errs.into_iter().collect()),
            Err(ref mut errors) => errors.extend(errs),
        }
    }
}
