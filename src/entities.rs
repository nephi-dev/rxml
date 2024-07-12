use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn get_child_by_name(&self, name: &str) -> Option<Node> {
        self.children.iter().find(|&child| child.name == name).cloned()
    }

    fn __getitem__(&self, key: &Bound<'_, PyAny>) -> PyResult<Option<Node>> {
        if let Ok(name) = key.extract::<String>() {
            Ok(self.get_child_by_name(&name))
        } else if let Ok(index) = key.extract::<usize>() {
            Ok(self.children.get(index).cloned())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err("Invalid key type. Key must be a string or an integer"))
        }
    }

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

#[cfg(test)]
mod tests {
    use crate::entities::Node;
    use crate::f_str;
    use std::collections::HashMap;

    #[test]
    fn test_node() {
        let mut attrs = HashMap::new();
        attrs.insert(f_str!("test"), f_str!("test"));
        let node = Node::new(
            f_str!("test"),
            Some(attrs),
            Some(Vec::new()),
            Some(f_str!("test")),
        )
        .unwrap();
        assert_eq!(node.name, String::from("test"));
        assert_eq!(node.attrs.len(), 1);
        assert_eq!(node.attrs.get("test").unwrap(), "test");
        assert_eq!(node.children.len(), 0);
        assert_eq!(node.text.unwrap(), "test");
    }

    #[test]
    fn test_get_child_by_name() {
        let child = Node::new(f_str!("child"), None, None, None).unwrap();
        let child2 = Node::new(f_str!("child2"), None, None, None).unwrap();
        let node = Node::new(
            f_str!("test"),
            None,
            Some(vec![child.clone(), child2.clone()]),
            None,
        )
        .unwrap();

        assert_eq!(node.get_child_by_name("child"), Some(child));
        assert_eq!(node.get_child_by_name("child2"), Some(child2));
        assert_eq!(node.get_child_by_name("child3"), None);
    }
}
