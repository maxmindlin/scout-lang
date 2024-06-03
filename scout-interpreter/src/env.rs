use crate::Object;
use scout_parser::ast::Identifier;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

pub(crate) type EnvPointer = Arc<Mutex<Env>>;

#[derive(Debug, Default)]
pub struct Env {
    store: HashMap<String, Arc<Object>>,
    outer: Mutex<Weak<Mutex<Env>>>,
}

impl Env {
    pub fn add_outer(&mut self, env: EnvPointer) {
        *self.outer.get_mut().unwrap() = Arc::downgrade(&env);
    }

    pub fn get(&self, id: &Identifier) -> Option<Arc<Object>> {
        match self.outer.borrow().lock().unwrap().upgrade() {
            None => self.store.get(&id.name).cloned(),
            Some(env) => {
                if let Some(obj) = self.store.get(&id.name) {
                    Some(obj.clone())
                } else {
                    env.lock().unwrap().get(id)
                }
            }
        }
    }

    pub fn set(&mut self, id: &Identifier, obj: Arc<Object>) {
        match self.outer.lock().unwrap().upgrade() {
            None => {
                self.store.insert(id.name.clone(), obj);
            }
            Some(env) => {
                if env.lock().unwrap().get(id).is_some() {
                    env.lock().unwrap().set(id, obj);
                } else {
                    self.store.insert(id.name.clone(), obj);
                }
            }
        }
    }
}
