use pest::iterators::Pair;
use crate::Rule;
use ast::*;

fn convert_identifier(ident: Pair<Rule>) -> Name {
    Name::from(ident.as_str().trim())
}

fn convert_qualified_name(pair: Pair<Rule>) -> QualifiedName {
    let pairs = pair.into_inner();
    let mut identifiers = vec![];

    for ident in pairs {
        identifiers.push(convert_identifier(ident));
    }

    QualifiedName::new(identifiers)
}

fn convert_namespace(pair: Pair<Rule>) -> QualifiedName {
    convert_qualified_name(pair.into_inner().next().unwrap())
}

fn convert_type_params(pair: Pair<Rule>) -> Vec<Box<TypeRef>> {
    pair.into_inner()
        .map(convert_type_ref)
        .map(Box::new)
        .collect()
}

fn convert_type_ref(pair: Pair<Rule>) -> TypeRef {
    let mut pairs = pair.into_inner();

    TypeRef::new(
        convert_qualified_name(pairs.next().unwrap()),
        pairs.next().map(convert_type_params).unwrap_or(vec![])
    )
}

fn convert_field(pair: Pair<Rule>) -> Field {
    let mut pairs = pair.into_inner();
    let name = convert_identifier(pairs.next().unwrap());
    let type_ref = convert_type_ref(pairs.next().unwrap());
    Field::new(
        name,
        type_ref
    )
}

fn convert_enum(pair: Pair<Rule>) -> TypeDef {
    let mut pairs = pair.into_inner();
    let name = convert_identifier(pairs.next().unwrap());
    let fields = pairs.into_iter()
        .map(convert_field)
        .collect();

    TypeDef::new(
        name,
        Type::Enum(Enum::new(fields))
    )
}

fn convert_struct(pair: Pair<Rule>) -> TypeDef {
    let mut pairs = pair.into_inner();
    let name = convert_identifier(pairs.next().unwrap());
    let fields = pairs.into_iter()
        .map(convert_field)
        .collect();

    TypeDef::new(
        name,
        Type::Struct(Struct::new(fields))
    )
}

fn convert_type_def(pair: Pair<Rule>) -> TypeDef {
    let tokens = pair.clone().tokens();
    let pair = pair.into_inner().next().expect(format!("{:?}", tokens).as_str());
    match pair.as_rule() {
        Rule::struct_def => convert_struct(pair),
        Rule::enum_def => convert_enum(pair),
        _ => panic!()
    }
}

pub(crate) fn convert_file(pair: Pair<Rule>) -> File {
    let mut pairs = pair.into_inner();
    let namespace = convert_namespace(pairs.next().unwrap());

    let mut types = vec![];
    for pair in pairs.into_iter() {
        match pair.as_rule() {
            Rule::type_def => types.push(convert_type_def(pair)),
            _ => {}
        }
    }

    File::new(
        namespace,
        types
    )
}