use std::collections::{HashMap, HashSet};

use regex::Regex;

#[derive(Clone)]
pub struct Snippet {
    code: String,
    params: HashSet<String>,
    param_lists: HashMap<String, ListGenerator>,
    repl: HashMap<String, Snippet>,
    repl_lists: HashMap<String, Vec<Snippet>>
}

#[derive(Constructor, Clone)]
struct ListGenerator {
    prefix_if_non_empty: String,
    delimiter: String,
    suffix_if_non_empty: String,
}

#[derive(Constructor)]
struct MatchData {
    start: usize,
    end: usize,
}

impl ListGenerator {
    fn apply(&self, snippets: &Vec<Snippet>) -> String {
        let prefix = if !snippets.is_empty() { &self.prefix_if_non_empty.as_str() } else { "" };
        let middle =
            snippets.iter()
                .map(Snippet::gen)
                .collect::<Vec<String>>()
                .join(&self.delimiter);
        let suffix = if !snippets.is_empty() { &self.suffix_if_non_empty.as_str() } else { "" };
        format!("{}{}{}", prefix, middle, suffix)
    }
}

impl Snippet {
    pub fn new<'a, S: Into<&'a str>>(s: S) -> Snippet {
        Snippet {
            code: String::from(s.into()),
            params: HashSet::new(),
            param_lists: HashMap::new(),
            repl: HashMap::new(),
            repl_lists: HashMap::new(),
        }
    }

    pub fn param(&self, name: &str) -> Snippet {
        let mut s = self.clone();
        s.params.insert(name.to_owned());
        return s;
    }

    pub fn param_list(&self, name: &str, prefix_if_non_empty: &str, delimeter: &str, suffix_if_non_empty: &str) -> Snippet {
        let mut s = self.clone();
        s.param_lists.insert(
            name.to_owned(), 
            ListGenerator::new(prefix_if_non_empty.to_owned(), delimeter.to_owned(), suffix_if_non_empty.to_owned())
        );
        return s;
    }

    pub fn with(&self, param: &str, value: &Snippet) -> Snippet {
        let mut s = self.clone();
        s.repl.insert(param.to_owned(), value.clone());
        return s;
    }

    pub fn with_list(&self, param: &str, values: Vec<Snippet>) -> Snippet {
        let mut s = self.clone();
        s.repl_lists.insert(param.to_owned(), values);
        return s;
    }

    pub fn gen(&self) -> String {
        let mut code = self.code.clone();
        
        // TODO: less dupliation
        for (param, value) in &self.repl {
            let re = Regex::new(format!("{}", param).as_str()).unwrap();
            
            let mut ms: Vec<MatchData> =
                re.find_iter(code.as_str())
                    .map(|m| MatchData::new(m.start(), m.end()))
                    .collect();

            ms.reverse();

            for m in ms {
                let indent = Self::indent_of_match(&m, &code);
                let indent = format!("\n{}", indent);
                let param_replacement = value.gen();
                let param_replacement = param_replacement.replace("\n", indent.as_str());

                code.replace_range(
                    m.start..m.end,
                    param_replacement.as_str()
                )
            }
        }

        for (param, value) in &self.repl_lists {
            let re = Regex::new(format!("{}", param).as_str()).unwrap();
            let generator = self.param_lists.get(param).unwrap();
            
            let mut ms: Vec<MatchData> =
                re.find_iter(code.as_str())
                    .map(|m| MatchData::new(m.start(), m.end()))
                    .collect();

            ms.reverse();

            for m in ms {
                let indent = Self::indent_of_match(&m, &code);
                let indent = format!("\n{}", indent);
                let param_replacement = generator.apply(value);
                let param_replacement = param_replacement.replace("\n", indent.as_str());

                code.replace_range(
                    m.start..m.end,
                    param_replacement.as_str()
                )
            }
        }

        code
    }

    fn indent_of_match(m: &MatchData, code: &String) -> String
    {
        let re_indent : Regex = Regex::new(r"\n([ \t]*).*$").unwrap();
        let beginning_of_string = &code[0..m.start];
        return if let Some(captures) = re_indent.captures(beginning_of_string) {
            if captures.len() >= 2 {
                String::from(captures.get(1).unwrap().as_str())
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

impl <'a, S> From<S> for Snippet where S : Into<&'a str> {
    fn from(s: S) -> Self {
        Snippet::new(s)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let c = Snippet::new(
"#FQN::#NAME(
    #ARGS
)
    #FIELDS_INIT
{}"
            )
            .param("#FQN")
            .param("#NAME")
            .param_list("#ARGS", "", ",\n", "")
            .param_list("#FIELDS_INIT", ":\n", ",\n", "")
            .with("#FQN", &Snippet::new("MYNS1::MYNS2"))
            .with("#NAME", &Snippet::new("my_class"))
            .with_list("#ARGS", vec![Snippet::new("a"), Snippet::new("b")])
            .with_list("#FIELDS_INIT", vec![Snippet::new("_a(a)"), Snippet::new("_b(b)")])
        ;

        let generated = c.gen();
        println!("{}", generated);

        assert_eq!(
"MYNS1::MYNS2::my_class(
    a,
    b
)
    :
    _a(a),
    _b(b)
{}",
        generated);
    }
}