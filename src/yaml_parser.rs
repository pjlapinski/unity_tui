use crate::{class_id::ClassId, fs, fs::FileLines};
use std::{fmt::Display, iter::Peekable, path::Path};

type IndentSize = u8;

#[derive(Debug, Clone)]
pub enum YamlValue {
    // int could be not needed, but would have to figure out how to do flags then. However,
    // Unity knows to convert a float to an int while deserializing. Tested on an int,
    // setting it to 1.5 changed to 1 after deserialization. Still not sure about flags
    Int(i64),
    Float(f64),
    Str(String),
    Entries(Vec<YamlEntry>),
    Object(Vec<YamlEntry>),
    Array(Vec<Vec<YamlEntry>>),
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
            YamlValue::Str(s) => {
                let mut lines = s.lines().peekable();
                let mut line = lines.next();
                if lines.peek().is_none() {
                    return s.to_string();
                }
                let mut single_quote = true;
                let mut output = "".to_owned();
                while line.is_some() {
                    let content = line.unwrap();
                    if single_quote && (content.contains("\\n") || content.contains("\\r")) {
                        single_quote = false;
                    }
                    if output.is_empty() {
                        output += content;
                    } else {
                        output +=
                            ("\r\n".to_owned() + "  ".repeat(indent as usize).as_str() + content)
                                .as_str();
                    }
                    line = lines.next();
                }
                if single_quote {
                    format!("'{}'", output)
                } else {
                    format!("\"{}\"", output)
                }
            }
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
                    .map(|entry| entry.to_indented_string(indent))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("\n{}", s)
            }
            YamlValue::Array(a) => {
                let s = a
                    .iter()
                    .map(|entries| {
                        let mut it = entries.iter();
                        let mut strings = vec![];
                        if let Some(entry) = it.next() {
                            strings.push(entry.to_array_string(indent));
                        }
                        for entry in it {
                            strings.push(entry.to_indented_string(indent + 1));
                        }
                        strings.join("\n")
                    })
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
        if self.key.is_empty() {
            format!(
                "{}{}",
                Self::indent(indent),
                self.value.to_indented_string(indent + 1)
            )
        } else {
            format!(
                "{}{}: {}",
                Self::indent(indent),
                self.key,
                self.value.to_indented_string(indent + 1)
            )
        }
    }

    pub fn to_array_string(&self, indent: IndentSize) -> String {
        if self.key.is_empty() {
            format!(
                "{}- {}",
                Self::indent(indent),
                self.value.to_indented_string(indent + 1)
            )
        } else {
            format!(
                "{}- {}: {}",
                Self::indent(indent),
                self.key,
                self.value.to_indented_string(indent + 1)
            )
        }
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

fn count_indents(line: &String) -> IndentSize {
    ((line.len() - line.trim_start().len()) / 2)
        .try_into()
        .unwrap()
}

macro_rules! err_on_line {
    ($line:expr) => {
        format!("Invalid object format encountered. Line:\n{}", $line)
    };
    ($line:expr, $extra:expr) => {
        format!(
            "Invalid object format encountered. Line:\n{}\n{}",
            $line, $extra
        )
    };
}

pub fn parse_file(path: &Path) -> Result<Vec<UnityObject>, String> {
    match fs::get_file_lines(path) {
        Ok(lines) => parse(lines),
        Err(e) => Err(e.to_string()),
    }
}

pub fn parse(text: FileLines) -> Result<Vec<UnityObject>, String> {
    let mut objs = vec![];
    let mut lines = text.peekable();

    let mut current_object = UnityObject {
        id: "".to_owned(),
        class_id: 0,
        object_type_name: "".to_owned(),
        entries: vec![],
    };
    while let Some(line) = lines.peek() {
        let line = match line {
            Ok(l) => l,
            Err(e) => return Err(e.to_string()),
        };

        if line.starts_with('%') || line.trim().is_empty() {
            lines.next();
            continue;
        }

        // In this format, a comment like this will mean we encountered an object header
        if line.starts_with("--- ") {
            let obj_exists = !current_object.id.is_empty();
            if obj_exists {
                objs.push(current_object.clone());
            }
            // format for this header is
            // --- !u!{class_id} &{id}
            let mut parts = line.split(' ').skip(1);

            // class_id
            match parts.next() {
                Some(s) => match s.strip_prefix("!u!") {
                    Some(s) => match s.parse::<ClassId>() {
                        Ok(class_id) => current_object.class_id = class_id,
                        Err(e) => return Err(e.to_string()),
                    },
                    None => return Err(err_on_line!(line, "Prefix \"!u!\" not found in header.")),
                },
                None => return Err(err_on_line!(line, "Empty header.")),
            }
            // id
            match parts.next() {
                Some(s) => match s.strip_prefix('&') {
                    Some(id) => current_object.id = id.to_owned(),
                    None => return Err(err_on_line!(line, "Prefix \"&\" not found in header.")),
                },
                None => return Err(err_on_line!(line, "Object id missing from the header.")),
            }
            current_object.object_type_name = "".to_owned();
            current_object.entries = vec![];

            lines.next();
            continue;
        }
        let indents = count_indents(line);
        if indents == 0 {
            match line.split_once(':') {
                Some((name, _)) => current_object.object_type_name = name.to_owned(),
                None => return Err(err_on_line!(line)),
            }
            current_object.entries = vec![];
            lines.next();
            continue;
        }
        current_object
            .entries
            .append(&mut parse_single(&mut lines, indents)?);
    }
    let obj_exists = !current_object.id.is_empty();
    if obj_exists {
        objs.push(current_object);
    }
    Ok(objs)
}

fn parse_single<T, E>(
    iterator: &mut Peekable<T>,
    indents: IndentSize,
) -> Result<Vec<YamlEntry>, String>
where
    T: Iterator<Item = Result<String, E>>,
    E: std::fmt::Debug + Display,
{
    let (next, values) = parse_single_inner(iterator, indents)?;
    if next {
        iterator.next();
    }
    Ok(values)
}

macro_rules! next_line {
    ($iterator:expr) => {
        match $iterator.peek() {
            Some(line) => line,
            None => return Err("Error reading file, unexpected EOF.".to_owned()),
        }
    };
}

fn parse_single_inner<T, E>(
    iterator: &mut Peekable<T>,
    indents: IndentSize,
) -> Result<(bool, Vec<YamlEntry>), String>
where
    T: Iterator<Item = Result<String, E>>,
    E: std::fmt::Debug + Display,
{
    let mut next = true;
    let line = match next_line!(iterator) {
        Ok(line) => line,
        Err(e) => return Err(e.to_string()),
    };
    let mut entries = vec![];

    let Some(mut parts) = line.split_once(':') else {
        return Err(err_on_line!(line));
    };
    // special case, we are in an array and the value is an object
    let (left, right) = line.split_at(2 * indents as usize);
    if right.starts_with('{') && left.ends_with("- ") {
        parts = (left, right);
    };
    let key = parts.0[(2 * indents).into()..].to_owned();

    // value might need to be an empty string. in that case, pass a space, and later change it back to empty
    let mut value = if parts.1 == " " {
        parts.1
    } else {
        parts.1.trim_start()
    };
    if value.starts_with("- ") {
        value = &value[2..];
    }

    match value {
        "" => {
            iterator.next();
            let mut line = match next_line!(iterator) {
                Ok(line) => line,
                Err(e) => return Err(e.to_string()),
            };
            let mut line_indents = count_indents(line);
            // definitely an entry
            if line_indents == indents + 1 {
                let mut values = vec![];
                while line_indents > indents {
                    values.append(&mut parse_single_inner(iterator, indents + 1)?.1);
                    iterator.next();
                    line = match next_line!(iterator) {
                        Ok(line) => line,
                        Err(e) => return Err(e.to_string()),
                    };
                    line_indents = count_indents(line);
                }

                entries.push(YamlEntry {
                    key,
                    value: YamlValue::Entries(values),
                });
                next = false;
            // definitely an array
            } else if line_indents == indents && line.trim_start().starts_with('-') {
                let mut values = vec![];
                while line.trim_start().starts_with('-') && !line.starts_with("---") {
                    let mut inner_values = vec![];
                    loop {
                        let mut parsed = parse_single_inner(iterator, indents + 1)?;
                        inner_values.append(&mut parsed.1);
                        if parsed.0 {
                            iterator.next();
                        }
                        line = match next_line!(iterator) {
                            Ok(line) => line,
                            Err(e) => return Err(e.to_string()),
                        };
                        if count_indents(line) <= indents {
                            break;
                        }
                    }
                    values.push(inner_values);
                }

                entries.push(YamlEntry {
                    key,
                    value: YamlValue::Array(values),
                });
                next = false;
            // most likely an empty string
            } else if line_indents <= indents {
                entries.push(YamlEntry {
                    key,
                    value: YamlValue::Str(String::new()),
                });
                next = false;
            } else {
                return Err(err_on_line!(line));
            }
        }
        s if s.starts_with('{') => {
            let mut l = s.strip_prefix('{').unwrap().to_owned();
            while !l.ends_with('}') {
                iterator.next();
                let line = match next_line!(iterator) {
                    Ok(line) => line,
                    Err(e) => return Err(e.to_string()),
                };
                l += (" ".to_owned() + line.trim_start()).as_str();
            }
            l = l.strip_suffix('}').unwrap().to_owned();

            let mut values = vec![];
            for kvp in l.split(", ") {
                let kvp: Result<String, E> = Ok(kvp.to_owned());
                values.append(&mut parse_single_inner(&mut vec![kvp].into_iter().peekable(), 0)?.1);
            }
            let value = YamlValue::Object(values);
            entries.push(YamlEntry { key, value });
        }

        s if s.starts_with('"') || s.starts_with('\'') => {
            let quote = s.chars().next().unwrap();
            let indents = count_indents(line);
            let mut value = s.strip_prefix(quote).unwrap().to_owned();
            while !value.ends_with(quote) {
                iterator.next();
                let line = match next_line!(iterator) {
                    Ok(line) => line
                        .as_str()
                        .get((indents as usize + 3)..)
                        .unwrap_or(line.as_str()),
                    Err(e) => return Err(e.to_string()),
                };
                let join_symbol = match quote {
                    '\'' => "\r\n".to_owned(),
                    '"' => "\n".to_owned(),
                    _ => {
                        return Err(
                            "Unknown string block beginning symbol. Expected \" or '".to_owned()
                        )
                    }
                };
                value += (join_symbol + line).as_str();
            }
            let value = YamlValue::Str(value.strip_suffix(quote).unwrap().to_owned());
            entries.push(YamlEntry { key, value });
        }
        s => {
            let value = if let Ok(i) = s.parse::<i64>() {
                // in case of, for example, a Unity GUID with value 00000000000000000000000000000000
                if s.len() != i.to_string().len() && s != "-0" {
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
    Ok((next, entries))
}
