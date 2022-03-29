use std::ops::Range;

use crate::content_encoding::ContentEncodingValue;
use crate::content_type::EXTENSION_CONTENT_TYPES;

#[derive(Debug, PartialEq, Eq)]
pub struct Basename<'a> {
    pub basename: &'a str,
    pub basename_range: Range<usize>,
    pub content_type_ext: Option<&'a str>,
    pub content_type_ext_range: Option<Range<usize>>,
    pub content_encoding_ext: Option<&'a str>,
    pub content_encoding_ext_range: Option<Range<usize>>,
    pub basename_no_ext: &'a str,
    pub basename_no_ext_range: Range<usize>,
}

pub fn grab_basename(raw_path: &str) -> Basename<'_> {
    let mut content_type_ext_range = None;
    let mut content_encoding_ext_range = None;
    let basename_range = raw_path.rfind('/').map(|n| n + 1).unwrap_or(0)..raw_path.len();
    let basename = &raw_path[basename_range.clone()];
    println!("Basename: {}", basename);

    if let Some(dot_index) = basename.rfind('.') {
        let start = basename_range.start;
        let slice = &raw_path[start..start + dot_index];
        content_type_ext_range = Some(start + dot_index + 1..basename_range.end);
        println!(
            "Without extension: {}, {}",
            slice,
            &raw_path[start + dot_index + 1..basename_range.end]
        );
        if let Some(second_dot_index) = slice.rfind('.') {
            content_encoding_ext_range = content_type_ext_range.take();
            content_type_ext_range = Some(start + second_dot_index + 1..start + dot_index);
            println!(
                "Second without extension: {}",
                &raw_path[start + second_dot_index + 1..start + dot_index]
            );
        }
    }
    let (basename_no_ext, basename_no_ext_range) = match content_type_ext_range.clone() {
        None => (basename, basename_range.clone()),
        Some(r) => (
            &raw_path[basename_range.start..r.start - 1],
            basename_range.start..r.start - 1,
        ),
    };
    let content_type_ext = content_type_ext_range.clone().map(|r| &raw_path[r]);
    let content_encoding_ext = content_encoding_ext_range.clone().map(|r| &raw_path[r]);
    Basename {
        basename,
        basename_range,
        content_type_ext,
        content_type_ext_range,
        content_encoding_ext,
        content_encoding_ext_range,
        basename_no_ext,
        basename_no_ext_range,
    }
}

impl Basename<'_> {
    pub fn find_content_type(&self) -> Option<(&str, &str)> {
        self.content_type_ext
            .and_then(|ext| EXTENSION_CONTENT_TYPES.get(ext))
            .cloned()
    }
    pub fn find_content_encoding(&self) -> Option<ContentEncodingValue> {
        self.content_encoding_ext
            .map(ContentEncodingValue::from_extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_strip_with_content_type_and_encoding() {
        assert_eq!(
            Basename {
                basename: "whatever.txt.gz",
                content_encoding_ext: Some("gz"),
                content_type_ext: Some("txt"),
                basename_no_ext: "whatever",
                basename_range: 6..21,
                content_type_ext_range: Some(15..18),
                content_encoding_ext_range: Some(19..21),
                basename_no_ext_range: 6..14
            },
            grab_basename("hello/whatever.txt.gz")
        )
    }
    #[test]
    fn no_path_strip_with_content_type_and_encoding() {
        assert_eq!(
            Basename {
                basename: "whatever.txt.gz",
                content_encoding_ext: Some("gz"),
                content_type_ext: Some("txt"),
                basename_no_ext: "whatever",
                basename_range: 0..15,
                content_type_ext_range: Some(9..12),
                content_encoding_ext_range: Some(13..15),
                basename_no_ext_range: 0..8
            },
            grab_basename("whatever.txt.gz")
        )
    }

    #[test]
    fn no_path_strip_with_content_type_and_no_encoding() {
        assert_eq!(
            Basename {
                basename: "whatever.txt",
                content_encoding_ext: None,
                content_type_ext: Some("txt"),
                basename_no_ext: "whatever",
                basename_range: 0..12,
                content_type_ext_range: Some(9..12),
                content_encoding_ext_range: None,
                basename_no_ext_range: 0..8
            },
            grab_basename("whatever.txt")
        )
    }
    #[test]
    fn no_path_strip_with_no_content_type_and_no_encoding() {
        assert_eq!(
            Basename {
                basename: "whatever",
                content_encoding_ext: None,
                content_type_ext: None,
                basename_no_ext: "whatever",
                basename_range: 0..8,
                content_type_ext_range: None,
                content_encoding_ext_range: None,
                basename_no_ext_range: 0..8
            },
            grab_basename("whatever")
        )
    }

    #[test]
    fn content_type_and_encoding_lookup() {
        let basename = Basename {
            basename: "whatever.txt.gz",
            content_encoding_ext: Some("gz"),
            content_type_ext: Some("txt"),
            basename_no_ext: "whatever",
            basename_range: 6..21,
            content_type_ext_range: Some(15..18),
            content_encoding_ext_range: Some(19..21),
            basename_no_ext_range: 6..14,
        };
        assert_eq!(
            Some(ContentEncodingValue::Gzip),
            basename.find_content_encoding()
        );
        assert_eq!(Some(("text", "plain")), basename.find_content_type());
    }
    #[test]
    fn no_content_type_and_encoding_lookup() {
        let basename = Basename {
            basename: "whatever",
            content_encoding_ext: None,
            content_type_ext: None,
            basename_no_ext: "whatever",
            basename_range: 6..21,
            content_type_ext_range: None,
            content_encoding_ext_range: None,
            basename_no_ext_range: 6..14,
        };
        assert_eq!(None, basename.find_content_encoding());
        assert_eq!(None, basename.find_content_type());
    }
}
