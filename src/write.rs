use pyo3::prelude::*;

use crate::entities::Node;
use crate::f_utf;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::fs::File;
use std::io::{Cursor, Write};

fn write_node(writer: &mut Writer<Cursor<Vec<u8>>>, node: Node) {
    let mut start = BytesStart::new(&node.name);
    for (k, v) in node.attrs {
        start.push_attribute((k.as_str(), v.as_str()));
    }
    writer.write_event(Event::Start(start)).unwrap();
    if let Some(text) = node.text {
        writer
            .write_event(Event::Text(BytesText::new(&text)))
            .unwrap();
    }
    for child in node.children {
        write_node(writer, child);
    }
    writer
        .write_event(Event::End(BytesEnd::new(node.name)))
        .unwrap();
}

pub fn write_node_to_string(node: Node, indent: usize, default_xml_def: bool) -> String {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', indent);
    write_node(&mut writer, node);
    let result = writer.into_inner().into_inner();
    let mut return_string = String::new();
    if default_xml_def {
        return_string.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    }
    return_string.push_str(&f_utf!(result));
    return_string
}

#[pyfunction]
#[pyo3(signature = (node, file_path, indent=None, default_xml_def=None))]
pub fn write_file(
    node: Node,
    file_path: String,
    indent: Option<usize>,
    default_xml_def: Option<bool>,
) {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    let mut file = File::create(file_path).unwrap();
    let xml_string = write_node_to_string(node, _indent, _default_xml_def);
    file.write_all(xml_string.as_bytes()).unwrap();
}

#[pyfunction]
#[pyo3(signature = (node, indent, default_xml_def=None))]
pub fn write_string(node: Node, indent: Option<usize>, default_xml_def: Option<bool>) -> String {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    write_node_to_string(node, _indent, _default_xml_def)
}

#[cfg(test)]
mod tests {
    use crate::entities::Node;
    use crate::f_str;
    use crate::write::{write_file, write_node_to_string, write_string};
    use std::collections::HashMap;
    use std::fs::{read_to_string, remove_file};
    fn root_node() -> Node {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("test"), f_str!("test"));
        let mut root = Node {
            name: f_str!("root"),
            attrs: attrs.clone(),
            children: Vec::new(),
            text: None,
        };
        let mut child = Node {
            name: f_str!("child"),
            attrs,
            children: Vec::new(),
            text: None,
        };
        child.children.push(Node {
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
        let result = write_node_to_string(root, 4, true);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_write_string() {
        let root = root_node();
        let expected = expected_file();
        let result = write_string(root, Some(4), Some(true));
        assert_eq!(result, expected);
    }
    #[test]
    fn test_write_file() {
        let root = root_node();
        let expected = expected_file();
        write_file(root, f_str!("tests/test_write.xml"), Some(4), Some(true));
        let file_str = read_to_string("tests/test_write.xml").unwrap();
        remove_file("tests/test_write.xml").unwrap();
        assert_eq!(file_str, expected);
    }
}
