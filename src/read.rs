use pyo3::prelude::*;

use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::entities::Node;
use crate::{f_str, f_utf};
fn get_attrs(attrs: Attributes) -> HashMap<String, String> {
    let mut _attrs = HashMap::new();
    for attr in attrs {
        let attr = attr.unwrap();
        let attr_name = f_utf!(attr.key.as_ref());
        let attr_value = f_utf!(attr.value);
        _attrs.insert(attr_name, attr_value);
    }
    _attrs
}
fn read_node(root_tag: String, reader: &mut Reader<&[u8]>) -> Node {
    let mut buf = Vec::new();
    let mut root = Node {
        name: root_tag.clone(),
        attrs: HashMap::new(),
        children: Vec::new(),
        text: None,
    };
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                _e if _e == root_tag.as_bytes() => {
                    root.attrs = get_attrs(e.attributes());
                }
                _ => {
                    let tag_name = f_utf!(e.name().as_ref());
                    let mut child = read_node(tag_name, reader);
                    child.attrs = get_attrs(e.attributes());
                    root.children.push(child);
                }
            },
            Ok(Event::Empty(e)) => {
                let node = Node {
                    name: f_utf!(e.name().as_ref()),
                    attrs: get_attrs(e.attributes()),
                    children: Vec::new(),
                    text: None,
                };
                root.children.push(node);
            }
            Ok(Event::Text(e)) => {
                root.text = Some(f_str!(e.unescape().unwrap()));
            }
            Ok(Event::End(e)) if e.name().as_ref() == root_tag.as_bytes() => {
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
    root
}

#[pyfunction]
pub fn read_file(file_path: String, root_tag: String) -> Node {
    let mut file = File::open(file_path).unwrap();
    let mut file_str = String::new();
    file.read_to_string(&mut file_str).unwrap();
    let mut reader = Reader::from_str(file_str.as_str());
    reader.config_mut().trim_text(true);
    read_node(f_str!(root_tag), &mut reader)
}

#[pyfunction]
pub fn read_string(xml_string: String, root_tag: String) -> Node {
    let mut reader = Reader::from_str(xml_string.as_str());
    reader.config_mut().trim_text(true);
    read_node(f_str!(root_tag), &mut reader)
}

#[cfg(test)]
mod tests {
    use crate::f_str;
    use crate::read::{read_file, read_string};
    use std::fs::{remove_file, File};
    use std::io::prelude::*;
    #[test]
    fn test_read_file() {
        let mut file = File::create("tests/test.xml").unwrap();
        file.write_all(b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<root test=\"test\">\n    <child test=\"test\">\n        <child>test</child>\n    </child>\n</root>")
            .unwrap();
        let node = read_file(f_str!("tests/test.xml"), f_str!("root"));
        remove_file("tests/test.xml").unwrap();
        assert_eq!(node.name, f_str!("root"));
        assert_eq!(node.attrs.len(), 1);
        assert_eq!(node.attrs.get("test").unwrap(), "test");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, f_str!("child"));
        assert_eq!(node.children[0].attrs.len(), 1);
        assert_eq!(node.children[0].attrs.get("test").unwrap(), "test");
        assert_eq!(node.children[0].text.as_ref().unwrap(), "test");
        assert_eq!(node.children[0].children.len(), 0);
    }
    #[test]
    fn test_read_self_closing_tag() {
        let xml_string = f_str!("<?xml version=\"1.0\" encoding=\"utf-8\"?><tag><wrapper><inner1>value</inner1><inner2 attr=\"attr\"/><inner3>value</inner3></wrapper></tag>");
        let node = read_string(xml_string, f_str!("tag"));
        assert_eq!(node.children[0].children.len(), 3);
    }
}
