use crate::object::Object;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::FutureExt;
use scout_parser::ast::Identifier;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

pub type EnvPointer = Arc<Mutex<Env>>;

#[derive(Debug, Default)]
pub struct Env {
    pub store: HashMap<String, Arc<Object>>,
    outer: Mutex<Weak<Mutex<Env>>>,
    pub globals: Vec<Identifier>,
}

impl Env {
    pub async fn add_outer(&mut self, env: EnvPointer) {
        *self.outer.get_mut() = Arc::downgrade(&env);
    }

    pub async fn inherit_globals(&mut self, other: EnvPointer) {
        let inner = other.lock().await;
        for global in &inner.globals {
            // This should be safe because we only add to globals
            // and store at the same time.
            let val = inner.get(global).await.unwrap();
            self.add_global(global, val.clone()).await;
        }
    }

    pub async fn add_global(&mut self, id: &Identifier, obj: Arc<Object>) {
        self.set(id, obj).await;
        self.globals.push(id.clone());
    }

    pub fn get<'a>(&'a self, id: &'a Identifier) -> BoxFuture<'a, Option<Arc<Object>>> {
        async move {
            match self.outer.borrow().lock().await.upgrade() {
                None => self.store.get(&id.name).cloned(),
                Some(env) => {
                    if let Some(obj) = self.store.get(&id.name) {
                        Some(obj.clone())
                    } else {
                        env.lock().await.get(id).await
                    }
                }
            }
        }
        .boxed()
    }

    pub fn set<'a>(&'a mut self, id: &'a Identifier, obj: Arc<Object>) -> BoxFuture<'a, ()> {
        async move {
            match self.outer.lock().await.upgrade() {
                None => {
                    self.store.insert(id.name.clone(), obj);
                }
                Some(env) => {
                    if env.lock().await.get(id).await.is_some() {
                        env.lock().await.set(id, obj).await;
                    } else {
                        self.store.insert(id.name.clone(), obj);
                    }
                }
            }
        }
        .boxed()
    }
}
