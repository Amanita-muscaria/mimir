use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DTNode {
    name: String,
    children: HashMap<String, DTNode>,
    properties: HashMap<String, Option<String>>,
}

impl DTNode {
    pub fn new<S: Into<String>>(name: S) -> Self {
        DTNode {
            name: name.into(),
            children: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    pub fn add_property(mut self, label: String, val: Option<String>) -> Self {
        self.properties.insert(label, val);
        return self;
    }

    pub fn add_child_get(&mut self, child: DTNode) -> &mut Self {
        let name = child.name.clone();
        self.children.insert(name.clone(), child);
        return self.children.get_mut(&name).unwrap();
    }

    pub fn add_child(&mut self, child: DTNode) {
        self.children.insert(child.name.clone(), child);
    }

    pub fn add_node<S: Into<String> + Clone>(&mut self, path: &[S], new_node: DTNode) {
        match self.get_node(path) {
            Some(node) => {
                node.add_child(new_node);
            }
            None => (),
        };
    }

    pub fn add_node_get<S: Into<String> + Clone>(
        &mut self,
        path: &[S],
        new_node: DTNode,
    ) -> Option<&DTNode> {
        match self.get_node(path) {
            Some(node) => Some(node.add_child_get(new_node)),
            None => None,
        }
    }

    pub fn get_or_add_node<S: Into<String> + Clone>(
        &mut self,
        path: &[S],
        node_name: &str,
    ) -> Option<&DTNode> {
        match self.get_node(path) {
            Some(n) => {
                if n.name == node_name.to_string() {
                    return Some(n);
                } else if n.children.contains_key(node_name) {
                    return n.children.get(node_name);
                } else {
                    n.children
                        .insert(node_name.to_string(), DTNode::new(node_name));
                    return n.children.get(node_name);
                }
            }
            None => {
                return None;
            }
        };
    }

    pub fn get_node<S: Into<String> + Clone>(&mut self, path: &[S]) -> Option<&mut DTNode> {
        let current = path.get(0);
        let next = path.get(1);
        match (next, current) {
            (Some(n), Some(c)) => {
                if c.clone().into() == self.name {
                    match self.children.get_mut(&n.clone().into()) {
                        Some(child) => child.get_node(&path[1..]),
                        None => None,
                    }
                } else {
                    None
                }
            }
            (None, Some(c)) => {
                if c.clone().into() == self.name {
                    return Some(self);
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
