#![allow(clippy::only_used_in_recursion)]
use crate::f_str;
use pyo3::{prelude::*, types::PyType};
use std::collections::HashMap;

#[derive(Clone, FromPyObject, IntoPyObject, Eq, PartialEq, Debug)]
pub enum HashmapTypes {
    String(String),
    Vec(Vec<HashMap<String, HashmapTypes>>),
    NullableString(Option<String>),
    Map(HashMap<String, String>),
}

#[derive(Clone, PartialEq)]
#[pyclass(eq, eq_int)]
pub enum SearchType {
    Tag,
    Attr,
    Text,
}

#[derive(Clone)]
#[pyclass]
pub struct Node {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
    #[pyo3(get)]
    pub children: Vec<Node>,
    #[pyo3(get)]
    pub text: Option<String>,
}

#[pymethods]
impl Node {
    #[new]
    #[pyo3(signature = (name, attrs=None, children=None, text=None))]
    pub fn new(
        name: String,
        attrs: Option<HashMap<String, String>>,
        children: Option<Vec<Node>>,
        text: Option<String>,
    ) -> PyResult<Self> {
        let _attrs = attrs.unwrap_or_default();
        let _children = children.unwrap_or_default();
        Ok(Node {
            name,
            attrs: _attrs,
            children: _children,
            text,
        })
    }
    #[pyo3(signature = (spacing=None))]
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
    #[pyo3(signature = (name, depth=None))]
    fn search_by_name(&self, name: &str, depth: Option<i32>) -> Vec<Node> {
        let mut nodes = Vec::new();
        if self.name == name {
            nodes.push(self.clone());
        }
        if let Some(d) = depth {
            if d == 0 {
                return nodes;
            }
        }
        for child in &self.children {
            nodes.append(&mut child.search_by_name(name, depth.map(|d| d - 1)));
        }
        nodes
    }
    #[pyo3(signature = (key, depth=None))]
    fn search_by_attr(&self, key: &str, depth: Option<i32>) -> Vec<Node> {
        let mut nodes = Vec::new();
        if self.attrs.contains_key(key) {
            nodes.push(self.clone());
        }
        if let Some(d) = depth {
            if d == 0 {
                return nodes;
            }
        }
        for child in &self.children {
            nodes.append(&mut child.search_by_attr(key, depth.map(|d| d - 1)));
        }
        nodes
    }
    #[pyo3(signature = (text, depth=None))]
    fn search_by_text(&self, text: &str, depth: Option<i32>) -> Vec<Node> {
        let mut nodes = Vec::new();
        if let Some(t) = &self.text {
            if t == text {
                nodes.push(self.clone());
            }
        }
        if let Some(d) = depth {
            if d == 0 {
                return nodes;
            }
        }
        for child in &self.children {
            nodes.append(&mut child.search_by_text(text, depth.map(|d| d - 1)));
        }
        nodes
    }
    #[pyo3(signature = (by, value, depth=None))]
    pub fn search(&self, by: SearchType, value: &str, depth: Option<i32>) -> Vec<Node> {
        match by {
            SearchType::Tag => self.search_by_name(value, depth),
            SearchType::Attr => self.search_by_attr(value, depth),
            SearchType::Text => self.search_by_text(value, depth),
        }
    }

    #[classmethod]
    pub fn from_dict(
        cls: &Bound<'_, PyType>,
        dict_: HashMap<String, HashmapTypes>,
    ) -> PyResult<Self> {
        let name = match dict_.get("name") {
            Some(HashmapTypes::String(n)) => n,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid name")),
        }
        .clone();
        let attrs = match dict_.get("attrs") {
            Some(HashmapTypes::Map(a)) => a,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid attrs")),
        }
        .clone();
        let children = match dict_.get("children") {
            Some(HashmapTypes::Vec(c)) => c,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid children")),
        }
        .iter()
        .map(|child| Node::from_dict(cls, child.clone()))
        .collect::<PyResult<Vec<Node>>>()?;
        let text = match dict_.get("text") {
            Some(HashmapTypes::NullableString(t)) => t.clone(),
            Some(HashmapTypes::String(t)) => Some(t.clone()),
            _ => None,
        };
        Ok(Self {
            name,
            attrs,
            children,
            text,
        })
    }

    pub fn to_dict(&self) -> HashMap<String, HashmapTypes> {
        HashMap::from([
            (f_str!("name"), HashmapTypes::String(self.name.clone())),
            (f_str!("attrs"), HashmapTypes::Map(self.attrs.clone())),
            (
                f_str!("children"),
                HashmapTypes::Vec(self.children.iter().map(|child| child.to_dict()).collect()),
            ),
            (
                f_str!("text"),
                HashmapTypes::NullableString(self.text.clone()),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use pyo3::{PyTypeInfo, Python};

    use crate::entities::Node;
    use crate::f_str;
    use std::collections::HashMap;

    use super::HashmapTypes;
    #[test]
    fn test_node() {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("test"), f_str!("test"));
        let mut node = Node::new(
            f_str!("test"),
            Some(attrs.clone()),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        assert_eq!(node.name, String::from("test"));
        assert_eq!(node.attrs.len(), 1);
        assert_eq!(node.attrs.get("test").unwrap(), "test");
        assert_eq!(node.children.len(), 0);
        assert_eq!(node.text.clone().unwrap(), "test");
        let mut child_node = Node::new(
            f_str!("test new"),
            Some(attrs.clone()),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        let second_child_node = Node::new(
            f_str!("test new"),
            Some(attrs),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        child_node.children.push(second_child_node);
        node.children.push(child_node);
        assert_eq!(node.search_by_name("test", None).len(), 1);
        assert_eq!(node.search_by_name("test new", Some(2)).len(), 2);
        assert_eq!(node.search_by_attr("test", Some(2)).len(), 3);
        assert_eq!(node.search_by_text("test", Some(2)).len(), 3);
    }

    #[test]
    fn test_from_dict() {
        let mut hash1 = HashMap::new();
        let mut attrs = HashMap::new();
        let mut hash2 = HashMap::new();
        hash2.insert(f_str!("name"), HashmapTypes::String(f_str!("test")));
        hash2.insert(f_str!("attrs"), HashmapTypes::Map(HashMap::new()));
        hash2.insert(f_str!("children"), HashmapTypes::Vec(Vec::new()));
        hash2.insert(
            f_str!("text"),
            HashmapTypes::NullableString(Some(f_str!("test"))),
        );

        attrs.insert(f_str!("test"), f_str!("test"));

        hash1.insert(f_str!("name"), HashmapTypes::String(f_str!("test")));
        hash1.insert(f_str!("attrs"), HashmapTypes::Map(attrs));
        hash1.insert(f_str!("children"), HashmapTypes::Vec(vec![(hash2)]));
        hash1.insert(f_str!("text"), HashmapTypes::NullableString(None));

        Python::initialize();
        let node = Python::attach(|py| -> Node {
            Node::from_dict(&Node::type_object(py), hash1).unwrap()
        });
        assert_eq!(node.name, "test");
        assert_eq!(node.attrs.len(), 1);
        assert_eq!(node.attrs.get("test").unwrap(), "test");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].text.clone().unwrap(), f_str!("test"));
        assert_eq!(node.text, None);
    }

    #[test]
    fn test_to_dict() {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("test"), f_str!("test"));
        let mut node = Node::new(
            f_str!("test"),
            Some(attrs.clone()),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        let child_node = Node::new(
            f_str!("test new"),
            Some(attrs.clone()),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        node.children.push(child_node);
        let hash = node.to_dict();
        assert_eq!(
            hash.get("name").unwrap(),
            &HashmapTypes::String(f_str!("test"))
        );
        assert_eq!(hash.get("attrs").unwrap(), &HashmapTypes::Map(attrs));
        assert_eq!(
            hash.get("children").unwrap().clone(),
            HashmapTypes::Vec(vec![node.children[0].to_dict()])
        );
        assert_eq!(
            hash.get("text").unwrap(),
            &HashmapTypes::NullableString(Some(f_str!("test")))
        );
    }
}
