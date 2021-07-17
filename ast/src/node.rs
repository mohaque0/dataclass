
use serde::Serialize;
use serde::Deserialize;

use crate::Name;
use crate::QualifiedName;

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct Root {
    files: Vec<File>
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct File {
    namespace: QualifiedName,
    types: Vec<TypeDef>
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct TypeDef {
    name: Name,

    #[serde(rename = "type")]
    type_: Type
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Primitive(Primitive),
    Struct(Struct),
    Enum(Enum),
    List(Box<Type>)
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct Struct {
    fields: Vec<Field>
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct Enum {
    fields: Vec<Field>
}


#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct Field {
    name: Name,

    #[serde(rename = "type")]
    type_: TypeRef
}


#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Serialize, Deserialize)]
#[get="pub"]
pub struct TypeRef {
    name: QualifiedName,
    params: Vec<Box<TypeRef>>
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Primitive {
    Unit,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String
}