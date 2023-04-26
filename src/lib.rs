use pyo3::prelude::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

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

fn read_node(root_tag: String, reader: &mut Reader<BufReader<File>>) -> Node {
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

#[pyfunction]
fn read_file(file_path: String, root_tag: String) -> Node {
    let file_name = f_str!(file_path);
    let mut reader = Reader::from_file(file_name).unwrap();
    reader.trim_text(true);
    read_node(f_str!(root_tag), &mut reader)
}

#[pymodule]
fn xml_reader(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Node>()?;
    m.add_function(wrap_pyfunction!(read_file, m)?)?;
    Ok(())
}
