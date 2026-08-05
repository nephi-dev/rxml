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
#[pyclass(from_py_object, eq, eq_int)]
pub enum SearchType {
    Tag,
    Attr,
    Text,
}

/// Plain, GIL-free tree used for parsing and serialization off the GIL.
pub struct RawNode {
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<RawNode>,
    pub text: Option<String>,
}

#[pyclass]
pub struct Node {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
    pub children: Vec<Py<Node>>,
    #[pyo3(get)]
    pub text: Option<String>,
}

impl Node {
    pub fn from_raw(py: Python<'_>, raw: RawNode) -> PyResult<Py<Node>> {
        let children = raw
            .children
            .into_iter()
            .map(|child| Node::from_raw(py, child))
            .collect::<PyResult<Vec<_>>>()?;
        Py::new(
            py,
            Node {
                name: raw.name,
                attrs: raw.attrs,
                children,
                text: raw.text,
            },
        )
    }

    pub fn to_raw(&self, py: Python<'_>) -> RawNode {
        RawNode {
            name: self.name.clone(),
            attrs: self.attrs.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.borrow(py).to_raw(py))
                .collect(),
            text: self.text.clone(),
        }
    }
}

fn collect_matches(
    handle: &Py<Node>,
    py: Python<'_>,
    matches: &impl Fn(&Node) -> bool,
    depth: Option<i32>,
    out: &mut Vec<Py<Node>>,
) {
    let node = handle.borrow(py);
    if matches(&node) {
        out.push(handle.clone_ref(py));
    }
    if let Some(0) = depth {
        return;
    }
    let next_depth = depth.map(|d| d - 1);
    for child in &node.children {
        collect_matches(child, py, matches, next_depth, out);
    }
}

#[pymethods]
impl Node {
    #[new]
    #[pyo3(signature = (name, attrs=None, children=None, text=None))]
    pub fn new(
        name: String,
        attrs: Option<HashMap<String, String>>,
        children: Option<Vec<Py<Node>>>,
        text: Option<String>,
    ) -> PyResult<Self> {
        Ok(Node {
            name,
            attrs: attrs.unwrap_or_default(),
            children: children.unwrap_or_default(),
            text,
        })
    }

    /// Children are shared references to the same nodes, not copies.
    #[getter]
    fn children(&self, py: Python<'_>) -> Vec<Py<Node>> {
        self.children
            .iter()
            .map(|child| child.clone_ref(py))
            .collect()
    }

    #[pyo3(signature = (spacing=None))]
    fn __to_string(&self, py: Python<'_>, spacing: Option<u8>) -> String {
        use std::fmt::Write as _;
        let _spacing = spacing.unwrap_or(0);
        let spaces = " ".repeat(_spacing as usize);
        let mut s = String::new();
        let _ = write!(s, "{}Name: {}", spaces, self.name);
        if !self.attrs.is_empty() {
            let _ = write!(s, "\n{}Attributes:", spaces);
            for (k, v) in &self.attrs {
                let _ = write!(s, "\n{}{}: {}", spaces, k, v);
            }
        }
        if let Some(text) = &self.text {
            let _ = write!(s, "\n{}Text: {}", spaces, text);
        }
        if !self.children.is_empty() {
            let _ = write!(s, "\n{}Children:", spaces);
            for child in &self.children {
                let _ = write!(
                    s,
                    "\n{}{}\n",
                    spaces,
                    child.borrow(py).__to_string(py, Some(_spacing + 2))
                );
            }
        }
        s
    }

    fn __str__(&self, py: Python<'_>) -> String {
        self.__to_string(py, None)
    }

    fn __repr__(&self) -> String {
        format!("Node({})", self.name)
    }
    #[pyo3(signature = (name, depth=None))]
    fn search_by_name(slf: &Bound<'_, Self>, name: &str, depth: Option<i32>) -> Vec<Py<Node>> {
        let mut nodes = Vec::new();
        collect_matches(
            slf.as_unbound(),
            slf.py(),
            &|node| node.name == name,
            depth,
            &mut nodes,
        );
        nodes
    }
    #[pyo3(signature = (key, depth=None))]
    fn search_by_attr(slf: &Bound<'_, Self>, key: &str, depth: Option<i32>) -> Vec<Py<Node>> {
        let mut nodes = Vec::new();
        collect_matches(
            slf.as_unbound(),
            slf.py(),
            &|node| node.attrs.contains_key(key),
            depth,
            &mut nodes,
        );
        nodes
    }
    #[pyo3(signature = (text, depth=None))]
    fn search_by_text(slf: &Bound<'_, Self>, text: &str, depth: Option<i32>) -> Vec<Py<Node>> {
        let mut nodes = Vec::new();
        collect_matches(
            slf.as_unbound(),
            slf.py(),
            &|node| node.text.as_deref() == Some(text),
            depth,
            &mut nodes,
        );
        nodes
    }
    #[pyo3(signature = (by, value, depth=None))]
    pub fn search(
        slf: &Bound<'_, Self>,
        by: SearchType,
        value: &str,
        depth: Option<i32>,
    ) -> Vec<Py<Node>> {
        match by {
            SearchType::Tag => Self::search_by_name(slf, value, depth),
            SearchType::Attr => Self::search_by_attr(slf, value, depth),
            SearchType::Text => Self::search_by_text(slf, value, depth),
        }
    }

    #[classmethod]
    pub fn from_dict(
        cls: &Bound<'_, PyType>,
        mut dict_: HashMap<String, HashmapTypes>,
    ) -> PyResult<Self> {
        let name = match dict_.remove("name") {
            Some(HashmapTypes::String(n)) => n,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid name")),
        };
        let attrs = match dict_.remove("attrs") {
            Some(HashmapTypes::Map(a)) => a,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid attrs")),
        };
        let children = match dict_.remove("children") {
            Some(HashmapTypes::Vec(c)) => c,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid children")),
        }
        .into_iter()
        .map(|child| Node::from_dict(cls, child).and_then(|node| Py::new(cls.py(), node)))
        .collect::<PyResult<Vec<Py<Node>>>>()?;
        let text = match dict_.remove("text") {
            Some(HashmapTypes::NullableString(t)) => t,
            Some(HashmapTypes::String(t)) => Some(t),
            _ => None,
        };
        Ok(Self {
            name,
            attrs,
            children,
            text,
        })
    }

    pub fn to_dict(&self, py: Python<'_>) -> HashMap<String, HashmapTypes> {
        HashMap::from([
            (f_str!("name"), HashmapTypes::String(self.name.clone())),
            (f_str!("attrs"), HashmapTypes::Map(self.attrs.clone())),
            (
                f_str!("children"),
                HashmapTypes::Vec(
                    self.children
                        .iter()
                        .map(|child| child.borrow(py).to_dict(py))
                        .collect(),
                ),
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
    use pyo3::{Py, PyTypeInfo, Python};

    use crate::entities::Node;
    use crate::f_str;
    use std::collections::HashMap;

    use super::HashmapTypes;
    #[test]
    fn test_node() {
        Python::initialize();
        Python::attach(|py| {
            let mut attrs = HashMap::new();
            attrs.insert(f_str!("test"), f_str!("test"));
            let node = Node::new(
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
            child_node
                .children
                .push(Py::new(py, second_child_node).unwrap());
            let root = Py::new(py, node).unwrap();
            root.borrow_mut(py)
                .children
                .push(Py::new(py, child_node).unwrap());
            let bound = root.bind(py);
            let by_name = Node::search_by_name(bound, "test", None);
            assert_eq!(by_name.len(), 1);
            // Search results are shared references to the tree, not copies.
            assert!(by_name[0].is(&root));
            assert_eq!(Node::search_by_name(bound, "test new", Some(2)).len(), 2);
            assert_eq!(Node::search_by_attr(bound, "test", Some(2)).len(), 3);
            assert_eq!(Node::search_by_text(bound, "test", Some(2)).len(), 3);
            let children = root.borrow(py).children(py);
            assert!(children[0].is(&root.borrow(py).children[0]));
        });
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
        Python::attach(|py| {
            let node = Node::from_dict(&Node::type_object(py), hash1).unwrap();
            assert_eq!(node.name, "test");
            assert_eq!(node.attrs.len(), 1);
            assert_eq!(node.attrs.get("test").unwrap(), "test");
            assert_eq!(node.children.len(), 1);
            assert_eq!(
                node.children[0].borrow(py).text.clone().unwrap(),
                f_str!("test")
            );
            assert_eq!(node.text, None);
        });
    }

    #[test]
    fn test_to_dict() {
        Python::initialize();
        Python::attach(|py| {
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
            node.children.push(Py::new(py, child_node).unwrap());
            let hash = node.to_dict(py);
            assert_eq!(
                hash.get("name").unwrap(),
                &HashmapTypes::String(f_str!("test"))
            );
            assert_eq!(hash.get("attrs").unwrap(), &HashmapTypes::Map(attrs));
            assert_eq!(
                hash.get("children").unwrap().clone(),
                HashmapTypes::Vec(vec![node.children[0].borrow(py).to_dict(py)])
            );
            assert_eq!(
                hash.get("text").unwrap(),
                &HashmapTypes::NullableString(Some(f_str!("test")))
            );
        });
    }
}
