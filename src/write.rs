use pyo3::prelude::*;

use crate::entities::{Node, RawNode};
use quick_xml::Writer;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::name::QName;
use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufWriter, Write};

const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";

const ATTR_SPECIAL: [char; 8] = ['&', '<', '>', '"', '\'', '\t', '\r', '\n'];

/// Escapes an attribute value, including tab/CR/LF as character references —
/// a spec-compliant reader normalizes their literal forms to spaces
/// (XML 1.0 §3.3.3), which would corrupt round-trips.
fn push_attribute(start: &mut BytesStart<'_>, key: &str, value: &str) {
    if !value.contains(ATTR_SPECIAL) {
        start.push_attribute(Attribute {
            key: QName(key.as_bytes()),
            value: Cow::Borrowed(value.as_bytes()),
        });
        return;
    }
    let mut escaped = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '\t' => escaped.push_str("&#9;"),
            '\r' => escaped.push_str("&#13;"),
            '\n' => escaped.push_str("&#10;"),
            _ => escaped.push(ch),
        }
    }
    start.push_attribute(Attribute {
        key: QName(key.as_bytes()),
        value: Cow::Owned(escaped.into_bytes()),
    });
}

fn write_node<W: Write>(writer: &mut Writer<W>, node: RawNode) -> io::Result<()> {
    let mut start = BytesStart::new(&node.name);
    for (k, v) in &node.attrs {
        push_attribute(&mut start, k, v);
    }
    if node.children.is_empty() && node.text.is_none() {
        return writer.write_event(Event::Empty(start));
    }
    writer.write_event(Event::Start(start))?;
    if let Some(text) = node.text {
        writer.write_event(Event::Text(BytesText::new(&text)))?;
    }
    for child in node.children {
        write_node(writer, child)?;
    }
    writer.write_event(Event::End(BytesEnd::new(node.name)))
}

pub fn write_node_to_string(
    node: RawNode,
    indent: usize,
    default_xml_def: bool,
) -> io::Result<String> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', indent);
    write_node(&mut writer, node)?;
    let body = String::from_utf8(writer.into_inner())
        .expect("invariant: the XML writer only emits valid UTF-8");
    if !default_xml_def {
        return Ok(body);
    }
    let mut result = String::with_capacity(XML_DECL.len() + body.len());
    result.push_str(XML_DECL);
    result.push_str(&body);
    Ok(result)
}

#[pyfunction]
#[pyo3(signature = (node, file_path, indent=None, default_xml_def=None))]
pub fn write_file(
    py: Python<'_>,
    node: PyRef<'_, Node>,
    file_path: String,
    indent: Option<usize>,
    default_xml_def: Option<bool>,
) -> PyResult<()> {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    let raw = node.to_raw(py);
    py.detach(|| -> PyResult<()> {
        let file = File::create(&file_path)?;
        let mut buf = BufWriter::new(file);
        if _default_xml_def {
            buf.write_all(XML_DECL.as_bytes())?;
        }
        let mut writer = Writer::new_with_indent(buf, b' ', _indent);
        write_node(&mut writer, raw)?;
        writer.into_inner().flush()?;
        Ok(())
    })
}

#[pyfunction]
#[pyo3(signature = (node, indent=None, default_xml_def=None))]
pub fn write_string(
    py: Python<'_>,
    node: PyRef<'_, Node>,
    indent: Option<usize>,
    default_xml_def: Option<bool>,
) -> PyResult<String> {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    let raw = node.to_raw(py);
    py.detach(|| Ok(write_node_to_string(raw, _indent, _default_xml_def)?))
}

#[cfg(test)]
mod tests {
    use crate::entities::{Node, RawNode};
    use crate::f_str;
    use crate::read::read_string;
    use crate::write::{write_file, write_node_to_string, write_string};
    use pyo3::Python;
    use std::collections::HashMap;
    use std::fs::{read_to_string, remove_file};
    fn root_node() -> RawNode {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("test"), f_str!("test"));
        let mut root = RawNode {
            name: f_str!("root"),
            attrs: attrs.clone(),
            children: Vec::new(),
            text: None,
        };
        let mut child = RawNode {
            name: f_str!("child"),
            attrs,
            children: Vec::new(),
            text: None,
        };
        child.children.push(RawNode {
            name: f_str!("child"),
            attrs: HashMap::new(),
            children: Vec::new(),
            text: Some(f_str!("test")),
        });
        root.children.push(child);
        root
    }
    fn expected_file() -> &'static str {
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<root test=\"test\">\n    <child test=\"test\">\n        <child>test</child>\n    </child>\n</root>"
    }
    #[test]
    fn test_write_node_to_string() {
        let root = root_node();
        let expected = expected_file();
        let result = write_node_to_string(root, 4, true).unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_write_string() {
        let expected = expected_file();
        Python::initialize();
        let result = Python::attach(|py| {
            let node = Node::from_raw(py, root_node()).unwrap();
            write_string(py, node.borrow(py), Some(4), Some(true)).unwrap()
        });
        assert_eq!(result, expected);
    }
    #[test]
    fn test_write_file() {
        let expected = expected_file();
        Python::initialize();
        Python::attach(|py| {
            let node = Node::from_raw(py, root_node()).unwrap();
            write_file(
                py,
                node.borrow(py),
                f_str!("tests/test_write.xml"),
                Some(4),
                Some(true),
            )
            .unwrap();
        });
        let file_str = read_to_string("tests/test_write.xml").unwrap();
        remove_file("tests/test_write.xml").unwrap();
        assert_eq!(file_str, expected);
    }
    #[test]
    fn test_write_self_closing_tag() {
        let mut root = RawNode {
            name: f_str!("root"),
            attrs: HashMap::new(),
            children: Vec::new(),
            text: None,
        };
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("attr"), f_str!("value"));
        root.children.push(RawNode {
            name: f_str!("child"),
            attrs,
            children: Vec::new(),
            text: None,
        });
        let expected = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<root>\n    <child attr=\"value\"/>\n</root>";
        Python::initialize();
        let result = Python::attach(|py| {
            let node = Node::from_raw(py, root).unwrap();
            write_string(py, node.borrow(py), Some(4), Some(true)).unwrap()
        });
        assert_eq!(result, expected);
    }
    #[test]
    fn test_roundtrip_preserves_special_characters() {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("attr"), f_str!("a & b"));
        let root = RawNode {
            name: f_str!("root"),
            attrs,
            children: Vec::new(),
            text: Some(f_str!("1 < 2 & 3")),
        };
        Python::initialize();
        Python::attach(|py| {
            let node = Node::from_raw(py, root).unwrap();
            let xml = write_string(py, node.borrow(py), Some(4), Some(true)).unwrap();
            assert!(xml.contains("a &amp; b"));
            assert!(xml.contains("1 &lt; 2 &amp; 3"));
            let reread = read_string(py, xml, f_str!("root")).unwrap();
            let reread = reread.borrow(py);
            assert_eq!(reread.attrs.get("attr").unwrap(), "a & b");
            assert_eq!(reread.text.as_ref().unwrap(), "1 < 2 & 3");
        });
    }
    #[test]
    fn test_roundtrip_preserves_attr_whitespace() {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("attr"), f_str!("line1\nline2\tend"));
        let root = RawNode {
            name: f_str!("root"),
            attrs,
            children: Vec::new(),
            text: None,
        };
        Python::initialize();
        Python::attach(|py| {
            let node = Node::from_raw(py, root).unwrap();
            let xml = write_string(py, node.borrow(py), Some(4), Some(true)).unwrap();
            assert!(xml.contains("&#10;"));
            assert!(xml.contains("&#9;"));
            let reread = read_string(py, xml, f_str!("root")).unwrap();
            let reread = reread.borrow(py);
            assert_eq!(reread.attrs.get("attr").unwrap(), "line1\nline2\tend");
        });
    }
}
