use crate::{EvalError, Object};
use scout_parser::ast::Identifier;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub(crate) type EnvPointer = Rc<RefCell<Env>>;

#[derive(Debug, Default, Clone)]
pub struct Env {
    store: HashMap<String, Rc<Object>>,
    outer: RefCell<Weak<RefCell<Env>>>,
}

// impl PartialEq for Env {
//     fn eq(&self, other: &Env) -> bool {
//         self.store == other.store
//     }
// }
//
// impl Eq for Env {}

impl Env {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_outer(&mut self, env: EnvPointer) {
        *self.outer.borrow_mut() = Rc::downgrade(&env);
    }

    pub fn get(&self, id: &Identifier) -> Option<Rc<Object>> {
        match self.outer.borrow().upgrade() {
            None => self.store.get(&id.name).cloned(),
            Some(env) => {
                if let Some(obj) = self.store.get(&id.name) {
                    Some(Rc::clone(obj))
                } else {
                    env.borrow().get(id)
                }
            }
        }
    }

    // Assigns the key to an object. Will only assign if the key does not already exist
    // in the local scope. Will never assign to the outer scope.
    pub fn set(&mut self, id: &Identifier, obj: Rc<Object>) -> Result<(), EvalError> {
        if !self.store.contains_key(&id.name) {
            self.store.insert(id.name.clone(), obj);
            Ok(())
        } else {
            Err(EvalError::DuplicateDeclare)
        }
    }

    // Override the value of a var with a new object. Will prioritize overriding the local
    // scope before checking the outer scope. Will only override if the key already exists.
    pub fn assign(&mut self, id: &Identifier, obj: Rc<Object>) -> Result<(), EvalError> {
        if self.store.get(&id.name).is_some() {
            self.store.insert(id.name.clone(), obj);
            return Ok(());
        }

        match self.outer.borrow_mut().upgrade() {
            None => Err(EvalError::UnknownIdent),
            Some(env) => env.borrow_mut().assign(id, obj),
        }
    }
}
