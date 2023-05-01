use pyo3::prelude::*;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{Cursor, Write};

macro_rules! f_str {
    ($e:expr) => {
        String::from($e)
    };
}
macro_rules! f_utf {
    ($e:expr) => {
        String::from_utf8($e.to_vec()).unwrap()
    };
}

#[derive(Clone)]
#[pyclass]
struct Node {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    attrs: HashMap<String, String>,
    #[pyo3(get)]
    children: Vec<Node>,
    #[pyo3(get)]
    text: Option<String>,
    is_closed: bool,
}

#[pymethods]
impl Node {
    fn __to_string(&self, spacing: Option<u8>) -> String {
        let _spacing = spacing.unwrap_or(0);
        let spaces = " ".repeat(_spacing as usize);
        let mut s = String::new();
        s.push_str(&format!("{}Name: {}", spaces, self.name));
        if !self.attrs.is_empty() {
            s.push_str(&format!("\n{}Attributes:", spaces));
            for (k, v) in &self.attrs {
                s.push_str(&format!("\n{}{}: {}", spaces, k, v));
            }
        }
        if let Some(text) = &self.text {
            s.push_str(&format!("\n{}Text: {}", spaces, text));
        }
        if !self.children.is_empty() {
            s.push_str(&format!("\n{}Children:", spaces));
            for child in &self.children {
                s.push_str(&format!(
                    "\n{}{}\n",
                    spaces,
                    child.__to_string(Some(_spacing + 2))
                ));
            }
        }
        s
    }

    fn __str__(&self) -> String {
        self.__to_string(None)
    }

    fn __repr__(&self) -> String {
        format!("Node({})", self.name)
    }
}

fn read_node(root_tag: String, reader: &mut Reader<&[u8]>) -> Node {
    let mut buf = Vec::new();
    let mut root = Node {
        name: root_tag.clone(),
        attrs: HashMap::new(),
        children: Vec::new(),
        text: None,
        is_closed: false,
    };
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                _e if _e == root_tag.as_bytes() => {
                    for attr in e.attributes() {
                        let attr = attr.unwrap();
                        let attr_name = f_utf!(attr.key.as_ref());
                        let attr_value = f_utf!(attr.value);
                        root.attrs.insert(attr_name, attr_value);
                    }
                }
                _ => {
                    let tag_name = f_utf!(e.name().as_ref());
                    let child = read_node(tag_name, reader);
                    root.children.push(child);
                }
            },
            Ok(Event::Text(e)) => {
                root.text = Some(f_str!(e.unescape().unwrap()));
            }
            Ok(Event::End(e)) if e.name().as_ref() == root_tag.as_bytes() => {
                root.is_closed = true;
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

fn write_node_to_string(node: Node, indent: usize, default_xml_def: bool) -> String {
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
fn read_file(file_path: String, root_tag: String) -> Node {
    let mut file = File::open(file_path).unwrap();
    let mut file_str = String::new();
    file.read_to_string(&mut file_str).unwrap();
    let mut reader = Reader::from_str(file_str.as_str());
    reader.trim_text(true);
    read_node(f_str!(root_tag), &mut reader)
}

#[pyfunction]
fn read_string(xml_string: String, root_tag: String) -> Node {
    let mut reader = Reader::from_str(xml_string.as_str());
    reader.trim_text(true);
    read_node(f_str!(root_tag), &mut reader)
}

#[pyfunction]
fn write_file(node: Node, file_path: String, indent: Option<usize>, default_xml_def: Option<bool>) {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    let mut file = File::create(file_path).unwrap();
    let xml_string = write_node_to_string(node, _indent, _default_xml_def);
    file.write_all(xml_string.as_bytes()).unwrap();
}

#[pyfunction]
fn write_string(node: Node, indent: Option<usize>, default_xml_def: Option<bool>) -> String {
    let _indent = indent.unwrap_or(4);
    let _default_xml_def = default_xml_def.unwrap_or(true);
    write_node_to_string(node, _indent, _default_xml_def)
}

#[pymodule]
fn rxml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Node>()?;
    m.add_function(wrap_pyfunction!(read_file, m)?)?;
    m.add_function(wrap_pyfunction!(read_string, m)?)?;
    m.add_function(wrap_pyfunction!(write_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_string, m)?)?;
    Ok(())
}
