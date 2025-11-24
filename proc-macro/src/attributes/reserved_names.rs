use crate::*;

#[derive(Debug)]
pub enum ReservedNames {
  List(Vec<String>),
  Expr(Expr),
}

impl ToTokens for ReservedNames {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    tokens.extend(match self {
      Self::List(list) => quote! { vec![ #(#list,)* ] },
      Self::Expr(expr) => expr.to_token_stream(),
    });
  }
}

impl Default for ReservedNames {
  fn default() -> Self {
    Self::List(vec![])
  }
}
