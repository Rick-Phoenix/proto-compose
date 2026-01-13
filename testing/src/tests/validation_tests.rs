use super::*;

#[proto_message(proxied, no_auto_test)]
pub struct ProxiedMessageInCollections {
  #[proto(map(int32, message(proxied)))]
  pub map: HashMap<i32, ProxiedMessageInCollections>,
  #[proto(repeated(message(proxied)))]
  pub vec: Vec<ProxiedMessageInCollections>,
  #[proto(validate = |v| v.const_(1))]
  pub id: i32,
}

#[test]
fn proxied_message_in_collections() {
  let dummy = ProxiedMessageInCollections {
    map: HashMap::new(),
    vec: vec![],
    id: 1,
  };

  let msg = dummy.into_message();

  assert!(msg.validate().is_ok());
}
