use std::collections::HashMap;

use crate::{Type, TypeDef};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct TypeId(usize);

pub struct Context {
    types: HashMap<TypeId, Type>
}