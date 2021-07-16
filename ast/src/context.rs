use std::collections::HashMap;

use crate::{Type, TypeDef};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct TypeId(usize);

struct Context {
    types: HashMap<TypeId, Type>
}