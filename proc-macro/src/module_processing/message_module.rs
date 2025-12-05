use crate::*;

pub fn process_message_from_module(
  msg: &mut MessageData,
  oneofs_map: &mut HashMap<Ident, OneofData>,
  module_attrs: &ModuleAttrs,
) -> Result<(), Error> {
  let MessageData {
    fields,
    reserved_numbers,
    oneofs,
    used_tags,
    name,
    reserved_names,
    ..
  } = msg;

  for oneof in oneofs.iter() {
    let oneof_data = oneofs_map.get_mut(oneof).ok_or(spanned_error!(
      oneof,
      format!("Failed to find the data for the oneof `{oneof}`")
    ))?;

    for tag in &oneof_data.used_tags {
      used_tags.push(*tag);
    }
  }

  let unavailable_tags = reserved_numbers.clone().build_unavailable_ranges(used_tags);

  let mut tag_allocator = TagAllocator::new(&unavailable_tags);

  for field in fields {
    if field.is_ignored {
      continue;
    }

    if let Some(ident) = &field.oneof_ident {
      let oneof = oneofs_map.get_mut(ident).ok_or(spanned_error!(
        ident,
        format!("Failed to find the data for the oneof `{ident}`. If you are using a proxied oneof, use the `#[proto(oneof(proxied))]` attribute rather than using the proxied ident (ending with `Proto`) directly")
      ))?;

      for variant in &mut oneof.variants {
        if variant.is_ignored {
          continue;
        }

        if reserved_names.contains(&variant.name) {
          bail!(
            &variant.tokens,
            format!(
              "Name `{}` is a reserved name for message `{name}`",
              variant.name
            )
          )
        }

        if let Some(tag) = variant.tag {
          if reserved_numbers.contains(tag) {
            bail!(
              &field.tokens,
              format!("Tag {tag} used by oneof {ident} is a reserved number")
            );
          }
        } else {
          let tag = tag_allocator
            .next_tag()
            .map_err(|e| spanned_error!(&variant.tokens, e))?;

          variant.tag = Some(tag);
          oneof.used_tags.push(tag);

          let variant_attr: Attribute = parse_quote!(#[proto(tag = #tag)]);
          variant.inject_attr(variant_attr);
        }
      }

      let oneof_tags = &oneof.used_tags;

      let oneof_attr: Attribute = parse_quote!(#[proto(oneof(tags(#(#oneof_tags),*)))]);
      field.inject_attr(oneof_attr);

      continue;
    }

    if reserved_names.contains(&field.name) {
      bail!(&field.tokens, format!("Name `{}` is reserved", field.name));
    }

    if let Some(tag) = &field.tag {
      if reserved_numbers.contains(*tag) {
        bail!(&field.tokens, format!("Tag {tag} is a reserved number"));
      }
    } else {
      let tag = tag_allocator
        .next_tag()
        .map_err(|e| spanned_error!(&field.tokens, e))?;

      field.tag = Some(tag);

      let field_attr: Attribute = parse_quote!(#[proto(tag = #tag)]);
      field.inject_attr(field_attr);
    }
  }

  if let Some(full_name) = msg.full_name.get() {
    let full_name_attr: Attribute = parse_quote!(#[proto(full_name = #full_name)]);

    msg.inject_attr(full_name_attr);
  }

  msg.inject_attr(module_attrs.as_attribute());

  Ok(())
}
