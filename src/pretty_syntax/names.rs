use std::rc::Rc;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Clone, Debug)]
struct Scope {
    index_count: usize,
    old_counts: HashMap<Rc<String>, usize>,
}

#[derive(Clone, Debug)]
pub struct Names {
    names: Vec<Rc<String>>,
    counts: HashMap<Rc<String>, usize>,
    scopes: Vec<Scope>,
}

impl Names {
    pub fn new() -> Self {
        Names {
            names: Vec::new(),
            counts: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            index_count: self.names.len(),
            old_counts: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        let mut scope = self.scopes.pop().expect("Stack underflow");
        self.names.truncate(scope.index_count);
        for (name, old_count) in scope.old_counts.drain() {
            if old_count == 0 {
                self.counts.remove(&name);
            } else {
                self.counts.insert(name, old_count);
            }
        }
    }

    pub fn add_name(&mut self, name: Rc<String>) -> Rc<String> {
        if let Some(curr_count) = self.counts.get(&name).cloned() {
            if let Some(last_scope) = self.scopes.last_mut() {
                if !last_scope.old_counts.contains_key(&name) {
                    last_scope.old_counts.insert(name.clone(), curr_count);
                }
            }
            self.counts.insert(name.clone(), curr_count + 1);
            let new_name = Rc::new({
                let mut owned: String = (*name).clone();
                write!(&mut owned, "#{}", curr_count);
                owned
            });
            self.names.push(new_name.clone());
            new_name
        } else {
            if let Some(last_scope) = self.scopes.last_mut() {
                last_scope.old_counts.insert(name.clone(), 0);
            }
            self.counts.insert(name.clone(), 1);
            self.names.push(name.clone());
            name
        }
    }

    pub fn get_name(&self, index: usize) -> Rc<String> {
        self.names[index].clone()
    }
}
