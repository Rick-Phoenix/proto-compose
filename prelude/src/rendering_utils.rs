use std::fmt::Write;

use crate::*;

pub(crate) const PROTOBUF_MAX_TAG: i32 = 536_870_911;

#[derive(Clone, Copy)]
pub(crate) enum OptionKind {
  FieldOption,
  NormalOption,
}

pub(crate) fn render_option(
  option: &ProtoOption,
  option_str: &mut String,
  option_kind: OptionKind,
) {
  let option_value_str = match option_kind {
    OptionKind::FieldOption => option.render_as_field_option(),
    OptionKind::NormalOption => option.render(),
  };

  let mut lines = option_value_str.lines().enumerate().peekable();

  while let Some((i, line)) = lines.next() {
    if i != 0 {
      option_str.push_str("  ");
    }

    option_str.push_str(line);

    if lines.peek().is_some() {
      option_str.push('\n');
    }
  }
}

pub(crate) fn render_field_options<'a, I>(options: I, options_len: usize, field_str: &mut String)
where
  I: IntoIterator<Item = (usize, &'a ProtoOption)>,
{
  field_str.push_str(" [\n");

  for (i, option) in options {
    field_str.push_str("    ");
    render_option(option, field_str, OptionKind::FieldOption);

    if i != options_len - 1 {
      field_str.push_str(",\n");
    }
  }

  field_str.push_str("\n    ]");
}

pub(crate) fn render_normal_options<'a, I>(options: I) -> String
where
  I: IntoIterator<Item = &'a ProtoOption>,
{
  let mut options_str = String::new();

  for option in options {
    render_option(option, &mut options_str, OptionKind::NormalOption);
    options_str.push('\n');
  }

  options_str
}

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

pub(crate) fn render_reserved_names(names: &[&'static str]) -> Option<String> {
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
