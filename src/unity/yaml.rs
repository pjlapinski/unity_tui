use crate::class_id::ClassId;
use crate::unity::Id;
use crate::util::{ErrTo, PairWith};
use std::io::Read;
use std::{fs::File, io::Error, path::Path};
use unity_yaml_rust::{Yaml, YamlLoader};

pub struct YamlUnityDocument {
    pub class_id: ClassId,
    pub id: Id,
    pub document: Yaml,
}

fn bugfix_remove_negative_ids(text: String) -> (String, Vec<usize>) {
    let mut indices = vec![];
    let mut occurrence = 0;

    text.lines()
        .map(|line| {
            if line.starts_with("--- !u!") {
                if let Some(ch) = line.chars().skip_while(|ch| *ch != '&').nth(1) {
                    occurrence += 1;
                    if ch == '-' {
                        indices.push(occurrence - 1);
                        line.replace("&-", "&")
                    } else {
                        line.to_owned()
                    }
                } else {
                    line.to_owned()
                }
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join("\r\n")
        .pair_with(indices)
}

fn bugfix_restore_negative_ids(
    mut docs: Vec<YamlUnityDocument>,
    indices: Vec<usize>,
) -> Vec<YamlUnityDocument> {
    for idx in indices.iter() {
        let doc = &mut docs[*idx];
        doc.id = format!("-{}", doc.id);
    }
    docs
}

pub fn parse_file(path: &Path) -> Result<Vec<YamlUnityDocument>, Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // BUG: document with negative Id cannot be parsed
    let (content, indices) = bugfix_remove_negative_ids(content);

    let parsed = YamlLoader::load_from_str(&content).err_to_io_err()?;

    let mut docs = vec![];
    for doc in parsed {
        match doc {
            Yaml::Original(s) => {
                if s.starts_with("%YAML") || s.starts_with("%TAG") {
                    continue;
                } else if s.starts_with("--- ") {
                    let mut parts = s.split(' ');
                    parts.next();
                    let class_id = parts
                        .next()
                        .unwrap()
                        .strip_prefix("!u!")
                        .unwrap()
                        .to_string()
                        .parse::<ClassId>()
                        .unwrap();
                    let id = parts.next().unwrap().strip_prefix('&').unwrap().to_string();
                    docs.push(YamlUnityDocument {
                        class_id,
                        id,
                        document: Yaml::Null,
                    })
                } else if let Some(d) = docs.last_mut() {
                    if d.document == Yaml::Null {
                        d.document = Yaml::Original(s)
                    }
                }
            }
            doc => {
                if let Some(d) = docs.last_mut() {
                    if d.document == Yaml::Null {
                        d.document = doc
                    }
                }
            }
        }
    }

    docs = bugfix_restore_negative_ids(docs, indices);

    Ok(docs)
}
