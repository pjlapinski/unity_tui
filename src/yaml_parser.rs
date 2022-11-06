use crate::class_id::ClassId;
use std::{fmt::Display, iter::Peekable};

type IndentSize = u8;

#[derive(Debug, Clone)]
pub enum YamlValue {
    // int could be not needed, but would have to figure out how to do flags then. However,
    // Unity knows to convert a float to an int while deserializing. Tested on an int,
    // setting it to 1.5 changed to 1 after deserialization. Still not sure aboug flags
    Int(i64),
    Float(f64),
    Str(String),
    Entries(Vec<YamlEntry>),
    Object(Vec<YamlEntry>),
    Array(Vec<YamlEntry>),
}

#[derive(Debug, Clone)]
pub struct YamlEntry {
    pub key: String,
    pub value: YamlValue,
}

#[derive(Debug, Clone)]
pub struct UnityObject {
    pub id: String,
    pub class_id: ClassId,
    pub object_type_name: String,
    pub entries: Vec<YamlEntry>,
}

impl YamlValue {
    pub fn to_indented_string(&self, indent: IndentSize) -> String {
        match self {
            YamlValue::Int(i) => i.to_string(),
            YamlValue::Float(f) => f.to_string(),
            YamlValue::Str(s) => s.to_string(),
            YamlValue::Object(o) => {
                let s = o
                    .iter()
                    .map(|entry| format!("{}: {}", entry.key, entry.value.to_indented_string(0)))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{{{}}}", s)
            }
            YamlValue::Entries(e) => {
                let s = e
                    .iter()
                    .map(|entry| entry.to_indented_string(indent + 1))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("\n{}", s)
            }
            YamlValue::Array(a) => {
                let s = a
                    .iter()
                    .map(|entry| entry.to_array_string(indent))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("\n{}", s)
            }
        }
    }
}

impl Display for YamlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_indented_string(0))
    }
}

impl YamlEntry {
    fn indent(size: IndentSize) -> String {
        "  ".repeat(size as usize)
    }

    pub fn to_indented_string(&self, indent: IndentSize) -> String {
        format!(
            "{}{}: {}",
            Self::indent(indent),
            self.key,
            self.value.to_indented_string(indent + 1)
        )
    }

    pub fn to_array_string(&self, indent: IndentSize) -> String {
        format!(
            "{}- {}: {}",
            Self::indent(indent),
            self.key,
            self.value.to_indented_string(indent + 1)
        )
    }
}

impl Display for YamlEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_indented_string(0))
    }
}

impl Display for UnityObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "--- !u!{} &{}\n{}:\n{}",
            self.class_id,
            self.id,
            self.object_type_name,
            self.entries
                .iter()
                .map(|e| e.to_indented_string(1))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn count_indents(line: &str) -> IndentSize {
    ((line.len() - line.trim_start().len()) / 2)
        .try_into()
        .unwrap()
}

//pub fn parse(text: &str) -> Result<Vec<UnityObject>, &'static str> {
pub fn parse(text: &'static str) -> Vec<UnityObject> {
    let mut objs = vec![];
    let mut lines = text
        .lines()
        .skip_while(|line| line.starts_with('%'))
        .peekable();

    let mut current_object = UnityObject {
        id: "".to_owned(),
        class_id: 0,
        object_type_name: "".to_owned(),
        entries: vec![],
    };
    while let Some(line) = lines.peek() {
        if line.starts_with("--- ") {
            let obj_exists = !current_object.id.is_empty();
            if obj_exists {
                objs.push(current_object.clone());
            }
            let mut parts = line.split(' ').skip(1);
            current_object.class_id = parts
                .next()
                .unwrap()
                .strip_prefix("!u!")
                .unwrap()
                .parse::<ClassId>()
                .unwrap();
            current_object.id = parts.next().unwrap().strip_prefix('&').unwrap().to_owned();
            current_object.object_type_name = "".to_owned();
            current_object.entries = vec![];
            lines.next();
            continue;
        }
        let indents = count_indents(line);
        if indents == 0 {
            current_object.object_type_name = line.split_once(':').unwrap().0.to_owned();
            current_object.entries = vec![];
            lines.next();
            continue;
        }
        current_object
            .entries
            .append(&mut parse_single(&mut lines, indents));
        lines.next();
    }
    let obj_exists = !current_object.id.is_empty();
    if obj_exists {
        objs.push(current_object);
    }
    objs
}

fn parse_single<'a, T>(iterator: &mut Peekable<T>, indents: IndentSize) -> Vec<YamlEntry>
where
    T: Iterator<Item = &'a str>,
{
    let line = iterator.peek().unwrap();
    let mut entries = vec![];

    let parts = line.split_once(':').unwrap();
    let key = parts.0.chars().skip((2 * indents).into()).collect();

    // value might need to be an empty string. in that case, pass a space, and later change it back to empty
    let value = if parts.1 == " " {
        parts.1
    } else {
        parts.1.trim_start()
    };

    match value {
        // "" => todo!("Entries or Array"),
        s if s.starts_with('{') => {
            let l = s.strip_prefix('{').unwrap().strip_suffix('}').unwrap();
            let value = YamlValue::Object(
                l.split(", ")
                    .flat_map(|kvp| parse_single(&mut kvp.lines().peekable(), 0))
                    .collect(),
            );
            entries.push(YamlEntry { key, value })
        }
        s => {
            let value = if let Ok(i) = s.parse::<i64>() {
                // in case of, for example, a Unity GUID with value 00000000000000000000000000000000
                if s.len() != i.to_string().len() {
                    YamlValue::Str(s.to_owned())
                } else {
                    YamlValue::Int(i)
                }
            } else if let Ok(f) = s.parse::<f64>() {
                YamlValue::Float(f)
            } else if s == " " {
                YamlValue::Str(String::new())
            } else {
                YamlValue::Str(s.to_owned())
            };
            entries.push(YamlEntry { key, value });
        }
    }
    entries
}
