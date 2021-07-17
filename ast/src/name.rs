use serde::{Serialize,Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NameCase {
    SnakeCase,
    ScreamingSnakeCase,
    UpperCamelCase,
    LowerCamelCase,
    Fixed
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Name {
    tokens: Vec<String>,
    case: NameCase
}

#[derive(Constructor, Clone, Getters, CopyGetters, Setters, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[get]
pub struct QualifiedName {
    names: Vec<Name>
}
impl Name {
    pub fn from(name: &str) -> Name {
        // Sanitize the names
        let name = name
            .replace("/", "_")
            .replace("+", "_plus");

        // Tokenize
        let mut names = vec!();
        let mut current_name = String::new();
        let mut last_char_was_lowercase = false;
        for ch in name.chars() {
            if last_char_was_lowercase && ch.is_uppercase() {
                names.push(current_name);
                current_name = String::new()
            }
            current_name = current_name + ch.to_string().as_str();
            last_char_was_lowercase = ch.is_lowercase();
        }
        if !current_name.is_empty() {
            names.push(current_name)
        }

        return Name { tokens: names, case: NameCase::Fixed };
    }

    pub fn with_prepended(&self, prepended_token: &str) -> Name {
        let mut tokens = vec!(prepended_token.to_string());
        for token in self.tokens.clone() {
            tokens.push(token);
        }
        return Name { tokens: tokens, case: self.case };
    }

    fn check_reserved(s: String, reserved: &[&str]) -> String {
        for k in reserved {
            if &s.as_str() == k {
                return s + "_";
            }
        }
        return s;
    }

    pub fn to_fixed_case(&self) -> String {
        return self.tokens.join("");
    }

    pub fn to_snake_case(&self, reserved: &[&str]) -> String {
        let s = self.tokens.iter()
            .map(|x| { x.to_lowercase() })
            .collect::<Vec<String>>().join("_");

        return Name::check_reserved(s, reserved);
    }

    pub fn to_screaming_snake_case(&self,  reserved: &[&str]) -> String {
        let s = self.tokens.iter()
            .map(|x| { x.to_uppercase() })
            .collect::<Vec<String>>().join("_");

        return Name::check_reserved(s, reserved);
    }

    pub fn to_upper_camel_case(&self, reserved: &[&str]) -> String {
        let s = self.tokens
            .iter()
            .map(|x| {
                if x.is_empty() {
                    return String::new();
                }
                x[0..1].to_uppercase() + x[1..].to_lowercase().as_str()
            })
            .collect::<Vec<String>>()
            .join("");

        return Name::check_reserved(s, reserved);
    }

    pub fn to_lower_camel_case(&self, reserved: &[&str]) -> String {
        if self.tokens.len() == 0 {
            return String::new()
        }

        let (head,tail) = self.tokens.split_first().unwrap();
        let s = 
            head.to_lowercase() +
            tail.iter()
                .map(|x| {
                    if x.is_empty() {
                        return String::new();
                    }
                    x[0..1].to_uppercase() + x[1..].to_lowercase().as_str()
                })
                .collect::<Vec<String>>()
                .join("").as_str();

        return Name::check_reserved(s, reserved);
    }
}

impl ToString for Name {
    fn to_string(&self) -> String {
        match self.case {
            NameCase::Fixed => self.to_fixed_case(),
            NameCase::LowerCamelCase => self.to_lower_camel_case(&[]),
            NameCase::UpperCamelCase => self.to_upper_camel_case(&[]),
            NameCase::ScreamingSnakeCase => self.to_screaming_snake_case(&[]),
            NameCase::SnakeCase => self.to_snake_case(&[])
        }
    }
}

impl QualifiedName {
    pub fn empty() -> Self {
        QualifiedName { names: vec!() }
    }

    pub fn with_prepended(&self, name: &Name) -> QualifiedName {
        QualifiedName { names: std::iter::once(name.clone()).chain(self.names().clone()).collect() }
    }

    pub fn with_appended(&self, name: &Name) -> QualifiedName {
        let mut names = self.names().clone();
        names.push(name.clone());
        QualifiedName { names: names }
    }

    pub fn head(&self) -> Option<&Name> {
        if self.names.len() == 0 {
            return None;
        }
        Some(self.names.get(0).unwrap())
    }

    pub fn tail(&self) -> QualifiedName {
        let mut tail_names = self.names.clone();
        tail_names.remove(0);
        QualifiedName {
            names: tail_names
        }
    }

    pub fn parent(&self) -> QualifiedName {
        match self.names.split_last() {
            Some((_last,names)) =>
                QualifiedName {
                    names: names.to_vec()
                },
            None => QualifiedName::empty()
        }
    }

    pub fn last(&self) -> Option<&Name> {
        self.names.last()
    }

    pub fn is_prefixed_by(&self, prefix: &QualifiedName) -> bool {
        if self.names.len() < prefix.names.len() {
            return false;
        }

        for idx in 0..prefix.names.len() {
            if self.names.get(idx) != prefix.names.get(idx) {
                return false;
            }
        }

        return true;
    }
}

impl From<Vec<&str>> for QualifiedName {
    fn from(names: Vec<&str>) -> QualifiedName {
        QualifiedName { names: names.iter().map(|n| Name::from(&n)).collect() }
    }
}

impl ToString for QualifiedName {
    fn to_string(&self) -> String {
        self.names.iter().map(Name::to_string).collect::<Vec<String>>().join(".")
    }
}