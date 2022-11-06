type IndentSize = u8;
type ClassId = u32;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum YamlValue {
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

#[allow(dead_code)]
impl YamlValue {
    pub fn to_string(&self, indent: IndentSize) -> String {
        match self {
            YamlValue::Int(i) => i.to_string(),
            YamlValue::Float(f) => f.to_string(),
            YamlValue::Str(s) => s.to_string(),
            YamlValue::Object(o) => {
                let s = o
                    .iter()
                    .map(|entry| format!("{}: {}", entry.key, entry.value.to_string(0)))
                    .collect::<Vec<String>>()
                    .join(",");
                format!("{{{}}}", s)
            }
            YamlValue::Entries(e) => {
                let s = e
                    .iter()
                    .map(|entry| entry.to_string(indent + 1))
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

#[allow(dead_code)]
impl YamlEntry {
    fn indent(size: IndentSize) -> String {
        "  ".repeat(size as usize)
    }

    pub fn to_string(&self, indent: IndentSize) -> String {
        format!(
            "{}{}: {}",
            Self::indent(indent),
            self.key,
            self.value.to_string(indent + 1)
        )
    }

    pub fn to_array_string(&self, indent: IndentSize) -> String {
        format!(
            "{}- {}: {}",
            Self::indent(indent),
            self.key,
            self.value.to_string(indent + 1)
        )
    }
}

fn count_indents(line: &str) -> IndentSize {
    ((line.len() - line.trim_start().len()) / 2)
        .try_into()
        .unwrap()
}

//pub fn parse(text: &str) -> Result<Vec<UnityObject>, &'static str> {
pub fn parse(text: &str) -> Vec<UnityObject> {
    let mut objs = vec![];
    let mut lines = text.lines().skip_while(|line| line.starts_with('%'));

    let mut current_object = UnityObject {
        id: "".to_owned(),
        class_id: 0,
        object_type_name: "".to_owned(),
        entries: vec![],
    };
    while let Some(mut line) = lines.next() {
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
            continue;
            //objs.push(UnityObject { id, class_id, entry: () })
        }
        let indents = count_indents(line);
        if indents == 0 {
            current_object.object_type_name = line.split_once(':').unwrap().0.to_owned();
            current_object.entries = vec![];
            continue;
        }
        let parts = line.split_once(':').unwrap();
        let key: String = parts.0.chars().skip((2 * indents).into()).collect();

        match parts.1.trim_start() {
            "" => todo!("Entries or Array"),
            s if s.starts_with(" {") => todo!("Object"),
            s => {
                let value = if let Ok(i) = s.parse::<i64>() {
                    YamlValue::Int(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    YamlValue::Float(f)
                } else {
                    YamlValue::Str(s.to_owned())
                };
                current_object.entries.push(YamlEntry { key, value });
            }
        }
    }
    let obj_exists = !current_object.id.is_empty();
    if obj_exists {
        objs.push(current_object.clone());
    }
    objs
}
