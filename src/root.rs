mod dt_node;

use dt_node::DTNode;
use std::collections::HashMap;
use crate::dt_lexer::DTError;

#[derive(Debug)]
pub enum RootError {
    MissingNode,
    UnknownLabel,
    BadPath,
    Redefine,
    Err,
}
#[derive(Debug)]
pub struct Root {
    defines: HashMap<String, String>,
    labels: HashMap<String, Vec<String>>,
    the_root: Option<DTNode>,
}

impl Root {
    pub fn new() -> Self {
        Root {
            defines: HashMap::new(),
            labels: HashMap::new(),
            the_root: None,
        }
    }

    pub fn add_define(mut self, d: String, v: String) -> Result<Self, RootError> {
        if self.defines.contains_key(&d) {
            Err(RootError::Redefine)
        } else {
            self.defines.insert(d, v);
            Ok(self)
        }
    }

    pub fn add_node<P: ToString>(mut self, path: &Vec<P>, name: &P) -> Result<Self, RootError> {
        match self.the_root.as_mut() {
            Some(r) => {
                let n = find_node(r, path)?;
                n.add_child(DTNode::new(name.to_string()));
            }
            None => {
                if path.len() > 0 {
                    return Err(RootError::Err);
                } else {
                    self.the_root = Some(DTNode::new(name.to_string()));
                }
            }
        };
        Ok(self)
    }

    pub fn add_property<P: ToString>(
        mut self,
        path: &Vec<P>,
        props: (String, Option<String>),
    ) -> Result<Self, RootError> {
        match self.the_root.as_mut() {
            Some(r) => {
                let n = find_node(r, &path)?;
                n.add_properties(props);
            }
            None => return Err(RootError::Err),
        };
        return Ok(self);
    }

    pub fn add_path<P: ToString>(&mut self, l: P, p: &Vec<String>) {
        self.labels.insert(l.to_string(), p.clone());
    }

    pub fn get_path<P: ToString>(&mut self, l: P) -> Result<Vec<String>, RootError> {
        match self.labels.get_mut(&l.to_string()) {
            Some(p) => Ok(p.clone()),
            None => Err(RootError::UnknownLabel),
        }
    }

    pub fn delete_from_label<P: ToString>(self, l: P) -> Result<Self, RootError> {
        let mut root = self
            .the_root
            .expect("Need to make a node before you delete it");
        let mut path = match self.labels.get(&l.to_string()) {
            Some(p) => p.clone(),
            None => return Err(RootError::UnknownLabel),
        };
        let node = path.split_off(path.len() - 2)[0].to_string();
        match find_node(&mut root, &path) {
            Ok(n) => match n.remove(node) {
                Ok(_) => Ok(Self {
                    defines: self.defines,
                    labels: self.labels,
                    the_root: Some(root),
                }),
                Err(_) => Err(RootError::MissingNode),
            },
            Err(e) => Err(e),
        }
    }

    pub fn delete_node<P: ToString>(self, path: Vec<P>) -> Result<Self, RootError> {
        let mut root = self
            .the_root
            .expect("Need to make a node before you delete it");
        let node = match path.last() {
            Some(n) => n,
            None => return Err(RootError::BadPath),
        };
        match find_node(&mut root, &path) {
            Ok(n) => match n.remove(node.to_string()) {
                Ok(_) => Ok(Self {
                    defines: self.defines,
                    labels: self.labels,
                    the_root: Some(root),
                }),
                Err(_) => Err(RootError::MissingNode),
            },
            Err(e) => Err(e),
        }
    }
}

fn find_node<'a, P: ToString>(
    root: &'a mut DTNode,
    path: &Vec<P>,
) -> Result<&'a mut DTNode, RootError> {
    let mut n = root;

    if n.name == path[0].to_string() {
        for p in &path[1..] {
            match n.get_child(p.to_string()) {
                Some(o) => n = o,
                None => return Err(RootError::MissingNode)
            }
        }
        return Ok(n);
    }
    return Err(RootError::MissingNode);
}
