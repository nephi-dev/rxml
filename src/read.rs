use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::HashMap;
use std::fmt;
use std::fs;

use crate::entities::{Node, RawNode};

#[derive(Debug)]
pub enum ParseError {
    Xml {
        position: u64,
        source: quick_xml::Error,
    },
    RootTagNotFound {
        root_tag: String,
    },
    UnexpectedEof {
        tag: String,
    },
    UnknownEntity {
        position: u64,
        name: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Xml { position, source } => {
                write!(f, "malformed XML at byte {position}: {source}")
            }
            ParseError::RootTagNotFound { root_tag } => {
                write!(f, "root tag <{root_tag}> not found in document")
            }
            ParseError::UnexpectedEof { tag } => {
                write!(f, "unexpected end of document: <{tag}> is never closed")
            }
            ParseError::UnknownEntity { position, name } => {
                write!(f, "unknown entity reference '&{name};' at byte {position}")
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Xml { source, .. } => Some(source),
            ParseError::RootTagNotFound { .. }
            | ParseError::UnexpectedEof { .. }
            | ParseError::UnknownEntity { .. } => None,
        }
    }
}

impl From<ParseError> for PyErr {
    fn from(error: ParseError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}

fn at_position<E: Into<quick_xml::Error>>(position: u64) -> impl FnOnce(E) -> ParseError {
    move |source| ParseError::Xml {
        position,
        source: source.into(),
    }
}

fn get_attrs(attrs: Attributes, position: u64) -> Result<HashMap<String, String>, ParseError> {
    let mut map = HashMap::new();
    for attr in attrs {
        let attr = attr.map_err(at_position(position))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let value = attr
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(at_position(position))?
            .into_owned();
        map.insert(key, value);
    }
    Ok(map)
}

fn node_from_start(start: &BytesStart<'_>, position: u64) -> Result<RawNode, ParseError> {
    Ok(RawNode {
        name: String::from_utf8_lossy(start.name().as_ref()).into_owned(),
        attrs: get_attrs(start.attributes(), position)?,
        children: Vec::new(),
        text: None,
    })
}

/// Parses the element whose `Start` event was just consumed, up to its
/// matching `End` event. Matching by nesting depth (not tag name) keeps
/// children that share an ancestor's tag name intact; mismatched end tags
/// are rejected by the reader itself (`check_end_names` is on by default).
/// Entity references arrive as separate `GeneralRef` events, so text is
/// accumulated across events and trimmed once at the end.
fn parse_element(
    start: &BytesStart<'_>,
    reader: &mut Reader<&[u8]>,
) -> Result<RawNode, ParseError> {
    let mut node = node_from_start(start, reader.buffer_position())?;
    let mut text = String::new();
    loop {
        let position = reader.buffer_position();
        match reader.read_event().map_err(at_position(position))? {
            Event::Start(e) => {
                let child = parse_element(&e, reader)?;
                node.children.push(child);
            }
            Event::Empty(e) => {
                node.children
                    .push(node_from_start(&e, reader.buffer_position())?);
            }
            Event::Text(e) => {
                text.push_str(&e.decode().map_err(at_position(position))?);
            }
            Event::GeneralRef(e) => {
                if let Some(ch) = e.resolve_char_ref().map_err(at_position(position))? {
                    text.push(ch);
                    continue;
                }
                let name = e.decode().map_err(at_position(position))?;
                let resolved =
                    resolve_predefined_entity(&name).ok_or_else(|| ParseError::UnknownEntity {
                        position,
                        name: name.to_string(),
                    })?;
                text.push_str(resolved);
            }
            Event::End(_) => break,
            Event::Eof => return Err(ParseError::UnexpectedEof { tag: node.name }),
            _ => (),
        }
    }
    let text = text.trim();
    if !text.is_empty() {
        node.text = Some(text.to_string());
    }
    Ok(node)
}

/// Scans for the first element named `root_tag` — at any depth, so a subtree
/// of a larger document can be extracted — and parses it and its descendants.
fn parse_str(xml: &str, root_tag: &str) -> Result<RawNode, ParseError> {
    let mut reader = Reader::from_str(xml);
    loop {
        let position = reader.buffer_position();
        match reader.read_event().map_err(at_position(position))? {
            Event::Start(e) if e.name().as_ref() == root_tag.as_bytes() => {
                return parse_element(&e, &mut reader);
            }
            Event::Empty(e) if e.name().as_ref() == root_tag.as_bytes() => {
                return node_from_start(&e, reader.buffer_position());
            }
            Event::Eof => {
                return Err(ParseError::RootTagNotFound {
                    root_tag: root_tag.to_string(),
                });
            }
            _ => (),
        }
    }
}

#[pyfunction]
pub fn read_file(py: Python<'_>, file_path: String, root_tag: String) -> PyResult<Py<Node>> {
    let raw = py.detach(|| -> PyResult<RawNode> {
        let file_str = fs::read_to_string(&file_path)?;
        Ok(parse_str(&file_str, &root_tag)?)
    })?;
    Node::from_raw(py, raw)
}

#[pyfunction]
pub fn read_string(py: Python<'_>, xml_string: String, root_tag: String) -> PyResult<Py<Node>> {
    let raw = py.detach(|| parse_str(&xml_string, &root_tag))?;
    Node::from_raw(py, raw)
}

#[cfg(test)]
mod tests {
    use crate::f_str;
    use crate::read::{read_file, read_string};
    use pyo3::Python;
    use pyo3::exceptions::{PyFileNotFoundError, PyValueError};
    use std::fs::{File, remove_file};
    use std::io::prelude::*;
    #[test]
    fn test_read_file() {
        let mut file = File::create("tests/test.xml").unwrap();
        file.write_all(b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<root test=\"test\">\n    <child test=\"test\">\n        <child>test</child>\n    </child>\n</root>")
            .unwrap();
        Python::initialize();
        Python::attach(|py| {
            let handle = read_file(py, f_str!("tests/test.xml"), f_str!("root")).unwrap();
            remove_file("tests/test.xml").unwrap();
            let node = handle.borrow(py);
            assert_eq!(node.name, f_str!("root"));
            assert_eq!(node.attrs.len(), 1);
            assert_eq!(node.attrs.get("test").unwrap(), "test");
            assert_eq!(node.children.len(), 1);
            let child = node.children[0].borrow(py);
            assert_eq!(child.name, f_str!("child"));
            assert_eq!(child.attrs.len(), 1);
            assert_eq!(child.attrs.get("test").unwrap(), "test");
            // The nested <child> is a real child node; the old parser
            // collapsed same-named nesting and hoisted its text.
            assert_eq!(child.text, None);
            assert_eq!(child.children.len(), 1);
            let grandchild = child.children[0].borrow(py);
            assert_eq!(grandchild.text.as_ref().unwrap(), "test");
        });
    }
    #[test]
    fn test_read_self_closing_tag() {
        let xml_string = f_str!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><tag><wrapper><inner1>value</inner1><inner2 attr=\"attr\"/><inner3>value</inner3></wrapper></tag>"
        );
        Python::initialize();
        Python::attach(|py| {
            let handle = read_string(py, xml_string, f_str!("tag")).unwrap();
            let node = handle.borrow(py);
            assert_eq!(node.children[0].borrow(py).children.len(), 3);
        });
    }
    #[test]
    fn test_read_unescapes_text_and_attrs() {
        let xml_string = f_str!("<root attr=\"a &amp; b\">1 &lt; 2 &amp; 3</root>");
        Python::initialize();
        Python::attach(|py| {
            let handle = read_string(py, xml_string, f_str!("root")).unwrap();
            let node = handle.borrow(py);
            assert_eq!(node.attrs.get("attr").unwrap(), "a & b");
            assert_eq!(node.text.as_ref().unwrap(), "1 < 2 & 3");
        });
    }
    #[test]
    fn test_read_child_with_same_name_as_root() {
        let xml_string =
            f_str!("<item outer=\"1\"><item inner=\"2\"><item>deep</item></item><other/></item>");
        Python::initialize();
        Python::attach(|py| {
            let handle = read_string(py, xml_string, f_str!("item")).unwrap();
            let root = handle.borrow(py);
            assert_eq!(root.attrs.get("outer").unwrap(), "1");
            assert_eq!(root.children.len(), 2);
            let inner = root.children[0].borrow(py);
            assert_eq!(inner.name, "item");
            assert_eq!(inner.attrs.get("inner").unwrap(), "2");
            assert_eq!(inner.children.len(), 1);
            assert_eq!(inner.children[0].borrow(py).text.as_ref().unwrap(), "deep");
        });
    }
    #[test]
    fn test_read_empty_root() {
        let xml_string = f_str!("<root attr=\"x\"/>");
        Python::initialize();
        Python::attach(|py| {
            let handle = read_string(py, xml_string, f_str!("root")).unwrap();
            let node = handle.borrow(py);
            assert_eq!(node.attrs.get("attr").unwrap(), "x");
            assert_eq!(node.children.len(), 0);
        });
    }
    #[test]
    fn test_read_mismatched_end_tag_raises_value_error() {
        let xml_string = f_str!("<root><a></b></root>");
        Python::initialize();
        Python::attach(|py| {
            let error = read_string(py, xml_string, f_str!("root")).unwrap_err();
            assert!(error.is_instance_of::<PyValueError>(py));
        });
    }
    #[test]
    fn test_read_unclosed_tag_raises_value_error() {
        let xml_string = f_str!("<root><a>");
        Python::initialize();
        Python::attach(|py| {
            let error = read_string(py, xml_string, f_str!("root")).unwrap_err();
            assert!(error.is_instance_of::<PyValueError>(py));
            assert!(error.to_string().contains("never closed"));
        });
    }
    #[test]
    fn test_read_extracts_nested_root_tag() {
        let xml_string = f_str!("<data><meta/><root a=\"1\">x</root></data>");
        Python::initialize();
        Python::attach(|py| {
            let handle = read_string(py, xml_string, f_str!("root")).unwrap();
            let node = handle.borrow(py);
            assert_eq!(node.attrs.get("a").unwrap(), "1");
            assert_eq!(node.text.as_ref().unwrap(), "x");
        });
    }
    #[test]
    fn test_read_missing_root_tag_raises_value_error() {
        let xml_string = f_str!("<other>content</other>");
        Python::initialize();
        Python::attach(|py| {
            let error = read_string(py, xml_string, f_str!("root")).unwrap_err();
            assert!(error.is_instance_of::<PyValueError>(py));
            assert!(error.to_string().contains("not found"));
        });
    }
    #[test]
    fn test_read_missing_file_raises_file_not_found() {
        Python::initialize();
        Python::attach(|py| {
            let error =
                read_file(py, f_str!("tests/does_not_exist.xml"), f_str!("root")).unwrap_err();
            assert!(error.is_instance_of::<PyFileNotFoundError>(py));
        });
    }
}
