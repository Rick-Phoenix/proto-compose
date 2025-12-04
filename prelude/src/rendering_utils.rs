use std::fmt::Write;

use crate::*;

pub(crate) const PROTOBUF_MAX_TAG: i32 = 536_870_911;

pub(crate) enum OptionKind {
  FieldOption,
  NormalOption,
}

pub(crate) fn render_option(option: &ProtoOption, field_str: &mut String, option_kind: OptionKind) {
  let option_str = match option_kind {
    OptionKind::FieldOption => option.render_as_field_option(),
    OptionKind::NormalOption => option.render(),
  };

  let mut lines = option_str.lines().peekable();

  while let Some(line) = lines.next() {
    field_str.push_str("  ");
    field_str.push_str(line);

    if lines.peek().is_some() {
      field_str.push('\n');
    }
  }
}

pub(crate) fn render_reserved_numbers(ranges: &[Range<i32>]) -> Option<String> {
  if ranges.is_empty() {
    return None;
  }

  let mut output_str = format!("reserved ");

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

pub(crate) fn render_reserved_names(names: &[&'static str]) -> Option<String> {
  if names.is_empty() {
    return None;
  }

  let mut output_str = format!("reserved ");

  for (i, name) in names.iter().enumerate() {
    write!(output_str, "\"{name}\"").unwrap();

    if i != names.len() - 1 {
      output_str.push_str(", ");
    }
  }

  output_str.push(';');

  Some(output_str)
}
