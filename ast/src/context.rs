use std::collections::HashMap;

use crate::{Field, Struct, Type, TypeRef};
use crate::{QualifiedName, TypeDef, TypeId, node};
use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct Context {
    // All type definitions encountered so far.
    resolved_types: HashMap<TypeId, TypeInfo>,

    // Map from name to type id.
    name_to_type_id: HashMap<QualifiedName, TypeId>,

    // Map unknown type names encountered to type definitions that reference them.
    unresolved_ids: Vec<TypeId>,

    type_id_generator: TypeIdGenerator
}

#[derive(Constructor, Getters, Serialize, Deserialize)]
#[get]
pub struct TypeInfo {
    name: QualifiedName,
    def: TypeDef
}

#[derive(Serialize, Deserialize)]
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
            resolved_types: HashMap::new(),
            name_to_type_id: HashMap::new(),
            unresolved_ids: vec![],
            type_id_generator: TypeIdGenerator::new()
        }
    }

    pub fn from(ast: &node::Root) -> Context {
        let mut ctx = Context::new();
        ctx.add_root(ast);
        ctx
    }

    pub fn resolve_name(&self, ctx_namespace: &QualifiedName, qname: &QualifiedName) -> NameResolution {
        let qname = 
            if qname.names().len() == 1 {
                ctx_namespace.with_appended(qname.last().unwrap())
            } else {
                qname.clone()
            };

        let type_id =
            self.name_to_type_id
                .get(&qname)
                .map(TypeId::clone);

        let type_def =
            if let Some(id) = type_id {
                self.resolved_types.get(&id).map(TypeInfo::def).map(TypeDef::clone)
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
        let resolved = self.resolved_types.get(id);

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
        &self.unresolved_ids
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

            self.name_to_type_id.insert(qname.clone(), type_id);
            self.unresolved_ids.retain(|it| it != &type_id);
            self.add_unresolved_type_references_from_type(namespace, &type_id, typedef.type_());
            self.resolved_types.insert(
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
                    self.name_to_type_id.insert(resolution.name, type_id);
                    self.unresolved_ids.push(type_id)
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
        // TODO: handle params
        match typeref {
            TypeRef::ByName(name) => TypeRef::ById(
                self.resolve_name(ctx_namespace, name.name()).id().unwrap()
            ),
            TypeRef::ById(id) => TypeRef::ById(id.clone()),
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