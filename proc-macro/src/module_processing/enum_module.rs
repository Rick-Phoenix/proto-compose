use crate::*;

pub fn process_enum_from_module(
  enum_data: &mut EnumData,
  parent_message: Option<String>,
  package_attribute: &Attribute,
) -> Result<(), Error> {
  let EnumData {
    name: proto_name,
    reserved_names,
    reserved_numbers,
    variants,
    used_tags,
    tokens,
  } = enum_data;

  let taken_tags = reserved_numbers
    .clone()
    .build_unavailable_ranges(used_tags.clone());

  let mut tag_allocator = TagAllocator::new(&taken_tags.0);

  for variant in variants {
    if variant.tag.is_none() {
      let tag = tag_allocator.next_tag();

      let variant_attr: Attribute = parse_quote!(#[proto(tag = #tag)]);

      variant.inject_attr(variant_attr);
    }
  }

  enum_data.inject_attr(package_attribute.clone());

  Ok(())
}
