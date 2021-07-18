use std::fmt::Display;

use ast::*;
use colored::Colorize;
use parser::Rule;
use pest::iterators::{Pair, Pairs};
use regex::Regex;

pub trait DisplayableNode : Sized {
    fn displayable(&self) -> String;
    fn children(&self) -> Vec<Self>;
}

pub enum DisplayableAST<'a> {
    Root(&'a Root),
    File(&'a File),
    TypeDef(&'a TypeDef),
    Type(&'a Type),
    Struct(&'a Struct),
    Enum(&'a Enum),
    Field(&'a Field),
    TypeRef(&'a TypeRef),
    TypeRefByName(&'a TypeRefByName),
    TypeRefById(&'a TypeRefById),
    TypeId(&'a TypeId),
    Primitive(&'a Primitive)
}

#[derive(Clone)]
struct Indent {
    tabs: Vec<Indentation>
}

#[derive(Clone)]
enum Indentation {
    Blank,
    Branch,
    Node
}

impl Indent {
    fn new() -> Indent {
        Indent { tabs: vec![] }
    }

    fn append(&self, next: Indentation) -> Indent {
        let mut tabs = self.tabs.clone();
        tabs.push(next);
        Indent {
            tabs
        }
    }

    fn last_as(&self, last: Indentation) -> Indent {
        let mut tabs = self.tabs.clone();
        match tabs.pop() {
            Some(_) => tabs.push(last),
            None => {}
        }
        Indent {
            tabs
        }
    }
}

impl Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tabs.iter().map(|i| {
            let indent =
                match i {
                    Indentation::Blank => "  ",
                    Indentation::Branch => "| ",
                    Indentation::Node => "+-",
                };
            f.write_str(indent.black().to_string().as_str())
        })
        .fold(Ok(()), Result::and)
    }
}

impl DisplayableNode for Pair<'_, Rule> {
    fn displayable(&self) -> String {
        let re = Regex::new(r"\s+").unwrap();
        let rule_name = format!("{:?}", self.as_rule()).bright_white();
        let contents = format!("{}", re.replace_all(self.as_str(), " ")).blue();
        format!("{}{}{}{}", rule_name, "(".black(), contents, ")".black())
    }

    fn children(&self) -> Vec<Self> {
        self.clone().into_inner()
            .collect()
    }
}

impl DisplayableNode for DisplayableAST<'_> {
    fn displayable(&self) -> String {
        match self {
            DisplayableAST::Root(_) => "Root".to_string(),
            DisplayableAST::File(f) => format!("File({})", f.namespace().to_string()),
            DisplayableAST::TypeDef(t) => format!("TypeDef({})", t.name().to_string()),
            DisplayableAST::Type(t) => format!(
                "Type({})",
                match &t {
                    Type::Primitive(_) => "Primitive",
                    Type::Struct(_) => "Struct",
                    Type::Enum(_) => "Enum",
                    Type::List(_) => "List",
                }
            ),
            DisplayableAST::Struct(_) => format!("Struct"),
            DisplayableAST::Enum(_) => format!("Enum"),
            DisplayableAST::Field(f) => format!("Field({})", f.name().to_string()),
            DisplayableAST::Primitive(p) => format!("Primitive({:?})", p),
            DisplayableAST::TypeRef(_) => format!("TypeRef"),
            DisplayableAST::TypeRefByName(r) => format!("ByName({})", r.name().to_string()),
            DisplayableAST::TypeRefById(id) => format!("ById({})", id.id().id()),
            DisplayableAST::TypeId(id) => format!("ById({})", id.id()),
        }
    }

    fn children(&self) -> Vec<Self> {
        match self {
            DisplayableAST::Root(r) => r.files().iter().map(DisplayableAST::File).collect(),
            DisplayableAST::File(f) => f.types().iter().map(DisplayableAST::TypeDef).collect(),
            DisplayableAST::TypeDef(t) => vec![DisplayableAST::Type(t.type_())],
            DisplayableAST::Type(t) => match &t {
                Type::Primitive(p) => vec![DisplayableAST::Primitive(p)],
                Type::Struct(s) => vec![DisplayableAST::Struct(s)],
                Type::Enum(e) => vec![DisplayableAST::Enum(e)],
                Type::List(t) => vec![DisplayableAST::Type(t)],
            },
            DisplayableAST::Struct(s) => s.fields().iter().map(DisplayableAST::Field).collect(),
            DisplayableAST::Enum(e) => e.fields().iter().map(DisplayableAST::Field).collect(),
            DisplayableAST::Field(f) => vec![DisplayableAST::TypeRef(f.type_())],
            DisplayableAST::Primitive(_) => vec![],
            DisplayableAST::TypeRef(t) => match t {
                TypeRef::ByName(r) => vec![DisplayableAST::TypeRefByName(r)],
                TypeRef::ById(id) => vec![DisplayableAST::TypeRefById(id)]
            },
            DisplayableAST::TypeRefByName(t) => t.params().iter().map(Box::as_ref).map(DisplayableAST::TypeRef).collect(),
            DisplayableAST::TypeRefById(id) => id.params().iter().map(Box::as_ref).map(DisplayableAST::TypeRef).collect(),
            DisplayableAST::TypeId(_) => vec![],
        }
    }
}

fn display_node_with_indent(node: &impl DisplayableNode, indent: Indent) {
    println!("{}{}", indent.last_as(Indentation::Node), node.displayable());
    let children = node.children();
    for (i, child) in node.children().iter().enumerate() {
        let indent =
            if i == children.len() - 1 {
                indent.append(Indentation::Blank)
            } else {
                indent.append(Indentation::Branch)
            };
        display_node_with_indent(child, indent);
    }
}

pub fn display_debug_tree(node: &impl DisplayableNode) {
    display_node_with_indent(node, Indent::new());
}

pub fn display_debug_parse_tree(pairs: &Pairs<Rule>) {
    display_debug_tree(&pairs.clone().next().unwrap());
}

pub fn display_debug_ast(root: &Root) {
    display_debug_tree(&DisplayableAST::Root(root));
}
