use std::collections::HashMap;

use crate::{QualifiedName, TypeDef, TypeId, node};
use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct Context {
    types_by_id: HashMap<TypeId, TypeInfo>,
    types_by_name: HashMap<QualifiedName, TypeId>,
    unresolved_ids: Vec<TypeId>
}

#[derive(Serialize, Deserialize)]
pub struct TypeInfo {
    path: QualifiedName,
    def: TypeDef
}

impl Context {
    pub fn from(ast: &node::Root) -> Context {
        Context::new()
    }

    fn new() -> Context {
        Context {
            types_by_id: HashMap::new(),
            types_by_name: HashMap::new(),
            unresolved_ids: vec![]
        }
    }
}