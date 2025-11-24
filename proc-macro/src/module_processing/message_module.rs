use crate::*;

pub(crate) fn process_message_from_module(
  msg: &mut MessageData,
  oneofs_map: &mut HashMap<Ident, OneofData>,
  package_attribute: &Attribute,
) -> Result<(), Error> {
  let MessageData {
    fields,
    reserved_names,
    reserved_numbers,
    name: proto_name,
    oneofs,
    used_tags,
    ..
  } = msg;

  for oneof in oneofs {
    let oneof_data = oneofs_map.get_mut(oneof).expect("Failed to find oneof");

    let taken_tags = std::mem::take(&mut oneof_data.used_tags);
    used_tags.extend(taken_tags);
  }

  let reserved_numbers = std::mem::take(reserved_numbers);

  let unavailable_tags = reserved_numbers.build_unavailable_ranges(&used_tags);

  let mut tag_allocator = TagAllocator::new(&unavailable_tags);

  for field in fields {
    if field.is_oneof {
      let oneof = oneofs_map
        .get_mut(field.type_.inner().require_ident()?)
        .expect("Failed to find oneof");

      for variant in &mut oneof.variants {
        if variant.tag.is_none() {
          let tag = tag_allocator.next_tag();

          let variant_attr: Attribute = parse_quote!(#[proto(tag = #tag)]);

          variant.inject_attr(variant_attr);
        }
      }

      continue;
    }

    if field.tag.is_none() {
      let tag = tag_allocator.next_tag();

      let field_attr: Attribute = parse_quote!(#[proto(tag = #tag)]);

      field.inject_attr(field_attr);
    }
  }

  if let Some(full_name) = msg.full_name.get() {
    let full_name_attr: Attribute = parse_quote!(#[proto(full_name = #full_name)]);

    msg.inject_attr(full_name_attr);
  }

  msg.inject_attr(package_attribute.clone());

  Ok(())
}
