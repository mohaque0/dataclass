use std::collections::HashMap;

use crate::TypeRef;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct TypeId(usize);

pub struct Context {
    types: HashMap<TypeId, TypeRef>
}