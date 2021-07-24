use core::fmt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::vec;

use crate::{Field, Struct, Type, TypeRef, TypeRefById};
use crate::{QualifiedName, TypeDef, TypeId, node};
use serde::Serialize;
use serde::Deserialize;

pub struct Context {
    // All type definitions encountered so far.
    defined_types: HashMap<TypeId, TypeInfo>,

    // Map from name to type id. This is every type that was encountered, even if never defined anywhere.
    named_types: HashMap<QualifiedName, TypeId>,

    // Map unknown type names encountered to type definitions that reference them.
    undefined_ids: Vec<TypeId>,

    type_id_generator: TypeIdGenerator
}

#[derive(Constructor, Clone, Debug, Getters, Serialize, Deserialize)]
#[get]
pub struct TypeInfo {
    name: QualifiedName,
    def: TypeDef
}

// Workaround for serde_json that can only handle map keys that are strings: Use an intermediate type.
#[derive(Debug, Serialize, Deserialize)]
struct SerializableContext {
    defined_types: HashMap<usize, TypeInfo>,
    named_types: HashMap<String, usize>,
    undefined_ids: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TypeIdGenerator {
    next_type_id: usize
}

#[derive(Constructor, Debug, Getters)]
#[get="pub"]
pub struct NameResolution {
    name: QualifiedName,
    id: Option<TypeId>,
    def: Option<TypeDef>
}

impl Context {
    fn new() -> Context {
        Context {
            defined_types: HashMap::new(),
            named_types: HashMap::new(),
            undefined_ids: vec![],
            type_id_generator: TypeIdGenerator::new()
        }
    }

    pub fn resolve_name(&self, ctx_namespace: &QualifiedName, qname: &QualifiedName) -> NameResolution {
        let qname = 
            if qname.names().len() == 1 {
                ctx_namespace.with_appended(qname.last().unwrap())
            } else {
                qname.clone()
            };

        let type_id =
            self.named_types
                .get(&qname)
                .map(TypeId::clone);

        let type_def =
            if let Some(id) = type_id {
                self.defined_types.get(&id).map(TypeInfo::def).map(TypeDef::clone)
            } else {
                None
            };

        NameResolution::new(
            qname,
            type_id,
            type_def
        )
    }

    pub fn resolve_id(&self, id: &TypeId) -> Option<NameResolution> {
        let resolved = self.defined_types.get(id);

        match resolved {
            Some(info) => {
                Some(NameResolution::new(
                    info.name().clone(),
                    Some(id.clone()),
                    Some(info.def().clone())
                ))
            },
            None => None,
        }
    }

    pub fn unresolved_ids(&self) -> &Vec<TypeId> {
        &self.undefined_ids
    }

    fn add_root(&mut self, ast: &node::Root) {
        for file in ast.files() {
            self.add_file(file)
        }
    }

    fn add_file(&mut self, file: &node::File) {
        let namespace = file.namespace();
        
        for typedef in file.types() {
            let qname = namespace.with_appended(typedef.name());
            let type_id = self.resolve_name(namespace, &qname)
                .id()
                .or_else(|| Some(self.type_id_generator.next()))
                .unwrap();

            self.named_types.insert(qname.clone(), type_id);
            self.undefined_ids.retain(|it| it != &type_id);
            self.add_unresolved_type_references_from_type(namespace, &type_id, typedef.type_());
            self.defined_types.insert(
                type_id, 
                TypeInfo::new(
                    qname,
                    self.replace_refbyname_with_refbyid(namespace, typedef)
                )
            );
        }
    }

    fn add_unresolved_type_references_from_type(&mut self, ctx_namespace: &QualifiedName, container_type_id: &TypeId, type_: &node::Type) {
        match type_ {
            crate::Type::Primitive(_) => {},
            crate::Type::Struct(s) => {
                for field in s.fields() {
                    self.add_unresolved_type_references_from_ref(ctx_namespace, container_type_id, field.type_())
                }
            }
            crate::Type::Enum(e) => {
                for field in e.fields() {
                    self.add_unresolved_type_references_from_ref(ctx_namespace, container_type_id, field.type_())
                }
            },
            crate::Type::List(t) => self.add_unresolved_type_references_from_type(ctx_namespace, container_type_id, t),
        }
    }

    fn add_unresolved_type_references_from_ref(&mut self, ctx_namespace: &QualifiedName, container_type_id: &TypeId, typeref: &node::TypeRef) {
        match typeref {
            crate::TypeRef::ByName(r) => {
                let resolution = self.resolve_name(ctx_namespace, r.name());
                if resolution.id() == &None {
                    let type_id = self.type_id_generator.next();
                    self.named_types.insert(resolution.name, type_id);
                    self.undefined_ids.push(type_id)
                }

                for typeref in r.params() {
                    self.add_unresolved_type_references_from_ref(ctx_namespace, container_type_id, typeref);
                }
            }
            crate::TypeRef::ById(_) => {
                // TODO: Implement parameters using TypeRefById.
                todo!()
            }
        }
    }

    fn replace_refbyname_with_refbyid(&self, ctx_namespace: &QualifiedName, typedef: &node::TypeDef) -> node::TypeDef {
        TypeDef::new(
            typedef.name().clone(),
            self.replace_refbyname_with_refbyid_in_type(ctx_namespace, typedef.type_())
        )
    }

    fn replace_refbyname_with_refbyid_in_type(&self, ctx_namespace: &QualifiedName, type_: &node::Type) -> node::Type {
        match type_ {
            Type::Primitive(p) => Type::Primitive(p.clone()),
            Type::Struct(s) => {
                let new_fields = s.fields().iter()
                    .map(|f| {
                        Field::new(
                            f.name().clone(),
                            self.replace_refbyname_with_refbyid_in_ref(ctx_namespace, f.type_())
                        )
                    })
                    .collect();
                Type::Struct(Struct::new(new_fields))
            },
            Type::Enum(e) => {
                let new_fields = e.fields().iter()
                    .map(|f| {
                        Field::new(
                            f.name().clone(),
                            self.replace_refbyname_with_refbyid_in_ref(ctx_namespace, f.type_())
                        )
                    })
                    .collect();
                Type::Struct(Struct::new(new_fields))
            },
            Type::List(l) => Type::List(Box::new(self.replace_refbyname_with_refbyid_in_type(ctx_namespace, l))),
        }
    }

    fn replace_refbyname_with_refbyid_in_ref(&self, ctx_namespace: &QualifiedName, typeref: &node::TypeRef) -> node::TypeRef {
        match typeref {
            TypeRef::ByName(name) => TypeRef::ById(
                TypeRefById::new(
                    self.resolve_name(ctx_namespace, name.name()).id().unwrap(),
                    name.params().iter()
                        .map(|it| Box::new(self.replace_refbyname_with_refbyid_in_ref(ctx_namespace, it)))
                        .collect()
                )
            ),
            TypeRef::ById(id) => TypeRef::ById(
                TypeRefById::new(
                    id.id().clone(),
                    id.params().iter()
                        .map(|it| Box::new(self.replace_refbyname_with_refbyid_in_ref(ctx_namespace, it)))
                        .collect()
                )
            ),
        }
    }
}

impl From<&node::Root> for Context {
    fn from(ast: &node::Root) -> Context {
        let mut ctx = Context::new();
        ctx.add_root(ast);
        ctx
    }
}

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        SerializableContext::from(self).serialize(serializer)
    }
}

impl <'de> Deserialize<'de> for Context {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Ok(Context::from(&SerializableContext::deserialize(deserializer)?))
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut defined_type_map = HashMap::new();
        for (k, v) in &self.defined_types {
            // TODO: Nicely print all the QualifiedNames in the definition.
            defined_type_map.insert(format!("{} -> {}", k.id(), v.name()), v.def());
        }

        let ser_ctx = SerializableContext::from(self);

        let mut ctx_struct = f.debug_struct("Context");
        ctx_struct
            .field("defined", &defined_type_map)
            .field("named", &ser_ctx.named_types)
            .field("undefined", &ser_ctx.undefined_ids)
            .finish()
    }
}

impl SerializableContext {
    fn from(ctx: &Context) -> Self {
        let mut defined_types: HashMap<usize, TypeInfo> = HashMap::new();
        for (k, v) in &ctx.defined_types {
            defined_types.insert(k.id(), v.clone());
        }

        let mut named_types = HashMap::new();
        for (k, v) in &ctx.named_types {
            named_types.insert(k.to_string(), v.id());
        }

        let undefined_ids = ctx.undefined_ids.iter()
            .map(|id| id.id())
            .collect();

        Self {
            defined_types,
            named_types,
            undefined_ids
        }
    }
}

impl From<&SerializableContext> for Context {
    fn from(ser_ctx: &SerializableContext) -> Self {
        let mut type_id_generator = TypeIdGenerator::new();

        let mut defined_types = HashMap::new();
        for (k, v) in &ser_ctx.defined_types {
            type_id_generator.set_next(*k + 1);
            defined_types.insert(TypeId::new(*k), v.clone());
        }

        let mut named_types = HashMap::new();
        for (k, v) in &ser_ctx.named_types {
            let name_vec: Vec<&str> = k.split(".").collect();
            type_id_generator.set_next(*v + 1);
            named_types.insert(QualifiedName::from(name_vec), TypeId::new(*v));
        }

        let undefined_ids = ser_ctx.undefined_ids.iter()
            .map(|id| TypeId::new(*id))
            .collect();

        ser_ctx.undefined_ids.iter().for_each(|id| type_id_generator.set_next(*id + 1));

        Self {
            defined_types,
            named_types,
            undefined_ids,
            type_id_generator
        }
    }
}

impl TypeIdGenerator {
    fn new() -> TypeIdGenerator {
        TypeIdGenerator {
            next_type_id: 0
        }
    }

    fn next(&mut self) -> TypeId {
        let type_id = TypeId::new(self.next_type_id);
        self.next_type_id += 1;
        return type_id;
    }

    fn set_next(&mut self, value: usize) {
        self.next_type_id = value;
    }
}

impl NameResolution {
    pub fn is_referenced(&self) -> bool {
        self.id().is_some()
    }

    pub fn is_defined(&self) -> bool {
        self.def().is_some()
    }
}

#[cfg(test)]
mod tests {

    use crate::{Enum, Name, TypeRefByName};

    use super::*;

    #[test]
    fn test() {
        let ast = node::Root::new(
            vec![node::File::new(
                QualifiedName::empty().with_appended(&Name::from("myns")),
                vec![
                    TypeDef::new(
                        Name::from("mystruct"),
                        Type::Struct(Struct::new(vec![
                            Field::new(
                                Name::from("myfield"),
                                TypeRef::ByName(TypeRefByName::new(
                                    QualifiedName::empty().with_appended(&Name::from("myns")).with_appended(&Name::from("myenum")),
                                    vec![]
                                ))
                            )
                        ]))
                    ),
                    TypeDef::new(
                        Name::from("myenum"),
                        Type::Enum(Enum::new(vec![
                            Field::new(
                                Name::from("myfield"),
                                TypeRef::ByName(TypeRefByName::new(
                                    QualifiedName::empty().with_appended(&Name::from("myns")).with_appended(&Name::from("unknowntype")),
                                    vec![]
                                ))
                            )
                        ]))
                    )
                ]
            )]
        );

        let mut ctx = Context::new();
        ctx.add_root(&ast);

        let struct_resolution = ctx.resolve_name(
            &QualifiedName::empty().with_appended(&Name::from("myns")),
            &QualifiedName::empty().with_appended(&Name::from("mystruct"))
        );

        let unknown_resolution = ctx.resolve_name(
            &QualifiedName::empty().with_appended(&Name::from("myns")),
            &QualifiedName::empty().with_appended(&Name::from("unknowntype"))
        );

        assert!(struct_resolution.def().is_some());
        assert!(unknown_resolution.def().is_none());

        println!("{:#?}", struct_resolution);
        println!("{:#?}", unknown_resolution);
    }
}