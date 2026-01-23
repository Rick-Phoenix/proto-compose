use crate::*;

pub(crate) const PROTOBUF_MAX_TAG: i32 = 536_870_911;

pub(crate) fn render_reserved_numbers(ranges: &[Range<i32>]) -> Option<String> {
  if ranges.is_empty() {
    return None;
  }

  let mut output_str = "reserved ".to_string();

  for (i, range) in ranges.iter().enumerate() {
    let Range { start, end } = range;

    if *start == *end - 1 {
      write!(output_str, "{start}").unwrap();
    } else {
      let end = end - 1;

      if end == PROTOBUF_MAX_TAG {
        write!(output_str, "{start} to max").unwrap();
      } else {
        write!(output_str, "{start} to {end}").unwrap();
      }
    }

    if i != ranges.len() - 1 {
      output_str.push_str(", ");
    }
  }

  output_str.push(';');

  Some(output_str)
}

pub(crate) fn render_reserved_names(names: &[FixedStr]) -> Option<String> {
  if names.is_empty() {
    return None;
  }

  let mut output_str = "reserved ".to_string();

  for (i, name) in names.iter().enumerate() {
    write!(output_str, "\"{name}\"").unwrap();

    if i != names.len() - 1 {
      output_str.push_str(", ");
    }
  }

  output_str.push(';');

  Some(output_str)
}
