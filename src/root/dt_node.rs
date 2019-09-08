use std::collections::HashMap;

#[derive(Clone)]
pub struct DTNode {
    pub name: String,
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

    pub fn add_properties(&mut self, key: (String, Option<String>)) {
        let k = key.0;
        let v = key.1;
        self.properties.insert(k, v);
    }

    pub fn get_properties(&mut self) -> HashMap<String, DTNode> {
        return self.children.clone();
    }

    pub fn add_child(&mut self, child: DTNode) {
        self.children.insert(child.name.clone(), child);
    }

    pub fn get_child(&mut self, name: String) -> Option<&mut DTNode> {
        self.children.get_mut(&name)
    }

    pub fn remove(&mut self, name: String) -> Result<(), ()> {
        match self.children.remove_entry(&name) {
            Some(_c) => Ok(()),
            None => Err(()),
        }
    }
}
