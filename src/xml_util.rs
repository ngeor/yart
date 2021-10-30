extern crate xml;

use self::xml::{EventReader, EventWriter};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::string::FromUtf8Error;

/// Represents an XML element path.
pub enum ElementPath {
    /// An empty path.
    Empty,

    /// A leaf element, along side its ancestor path.
    Leaf(String, Box<ElementPath>),
}

impl ElementPath {
    /// Pushes a new item to the path.
    /// The current path becomes the ancestor path of the result value.
    pub fn push(self, name: &str) -> Self {
        Self::Leaf(name.to_owned(), Box::new(self))
    }

    /// Discards the last item of the path.
    /// Returns the remaining ancestor path.
    pub fn pop(self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Leaf(_, boxed_parent) => *boxed_parent,
        }
    }

    /// Checks if the given XML element names match the contents of this element path.
    /// Comparison is case sensitive.
    /// The order of the names is from parent element to child element.
    pub fn matches(&self, names: &[&str]) -> bool {
        match self {
            Self::Empty => names.is_empty(),
            Self::Leaf(name, boxed_parent) => match names.split_last() {
                Some((last, rest)) => last == name && boxed_parent.matches(rest),
                _ => false,
            },
        }
    }
}

#[derive(Debug)]
pub enum XmlError {
    ReadError(xml::reader::Error),
    WriterError(xml::writer::Error),
    FromUtf8Error(FromUtf8Error),
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(e) => std::fmt::Display::fmt(&e, f),
            Self::WriterError(e) => std::fmt::Display::fmt(&e, f),
            Self::FromUtf8Error(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for XmlError {}

impl From<xml::reader::Error> for XmlError {
    fn from(value: xml::reader::Error) -> Self {
        Self::ReadError(value)
    }
}

impl From<xml::writer::Error> for XmlError {
    fn from(value: xml::writer::Error) -> Self {
        Self::WriterError(value)
    }
}

impl From<FromUtf8Error> for XmlError {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8Error(value)
    }
}

/// Transforms the given XML string with the specified processor.
/// The processor is a function that receives an EventReader and
/// EventWriter.
pub fn transform_xml<F>(contents: &str, processor: F) -> Result<String, XmlError>
where
    F: FnOnce(EventReader<&[u8]>, &mut EventWriter<&mut Vec<u8>>) -> Result<(), XmlError>,
{
    let parser = xml::reader::EventReader::from_str(contents);
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = xml::writer::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut buf);
    processor(parser, &mut writer)?;
    let mut result = String::from_utf8(buf)?;
    if !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

pub fn echo<W: Write>(
    read_event: &xml::reader::XmlEvent,
    writer: &mut EventWriter<W>,
) -> Result<(), xml::writer::Error> {
    match read_event.as_writer_event() {
        Some(writer_event) => writer.write(writer_event),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match() {
        let element_path = ElementPath::Empty.push("project").push("modules");
        assert!(element_path.matches(&["project", "modules"]));
        assert!(!element_path.matches(&["modules", "project"]));
    }
}
