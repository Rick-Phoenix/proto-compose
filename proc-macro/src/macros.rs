macro_rules! get_ident_or_continue {
  ($path:expr) => {
    if let Some(ident) = $path.get_ident() {
      ident.to_string()
    } else {
      continue;
    }
  };
}
