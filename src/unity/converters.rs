use crate::unity::Id;
use unity_yaml_rust::{yaml::Hash, Yaml};

pub fn field_name_to_readable(name: &str) -> String {
    let name = name
        .strip_prefix("m_")
        .unwrap_or(name)
        .strip_prefix('_')
        .unwrap_or(name)
        .strip_suffix(">k__BackingField")
        .unwrap_or(name);

    name.chars()
        .fold(String::new(), |acc, ch| {
            if acc.is_empty() {
                acc + ch.to_uppercase().to_string().as_str()
            } else if ch == '_' {
                acc + " "
            } else if (ch.is_uppercase() && acc.chars().last().unwrap().is_lowercase())
                || (ch.is_numeric() && !acc.chars().last().unwrap().is_numeric())
            {
                acc + " " + ch.to_string().as_str()
            } else if ch == '<' {
                // for <name>k__BackingField
                acc
            } else if acc.chars().last().unwrap().is_whitespace() {
                acc + ch.to_uppercase().to_string().as_str()
            } else {
                acc + ch.to_string().as_str()
            }
        })
        .trim_end()
        .to_owned()
}

pub trait AsFileId {
    fn as_file_id(&self) -> Option<Id>;
}

impl AsFileId for Yaml {
    fn as_file_id(&self) -> Option<Id> {
        match self {
            Yaml::Integer(i) => Some(i.to_string()),
            Yaml::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

pub trait AsF32 {
    fn as_f32(&self) -> Option<f32>;
}

impl AsF32 for Yaml {
    fn as_f32(&self) -> Option<f32> {
        match self {
            Yaml::Real(f) => f.parse::<f32>().ok(),
            Yaml::Integer(i) => Some(*i as f32),
            _ => None,
        }
    }
}

pub trait GetFromStr {
    fn get_from_str(&self, s: &str) -> Option<&Yaml>;
}

impl GetFromStr for Hash {
    fn get_from_str(&self, s: &str) -> Option<&Yaml> {
        self.get(&Yaml::String(s.to_owned()))
    }
}

pub(super) mod helpers {
    use crate::unity::{
        converters::{AsF32, GetFromStr},
        vector::{Vector2, Vector3, Vector4},
    };
    use unity_yaml_rust::yaml::Hash;

    pub fn obj_to_vec4(yaml: &Hash) -> Option<Vector4> {
        let x = yaml.get_from_str("x")?.as_f32()?;
        let y = yaml.get_from_str("y")?.as_f32()?;
        let z = yaml.get_from_str("z")?.as_f32()?;
        let w = yaml.get_from_str("w")?.as_f32()?;
        Some(Vector4 { x, y, z, w })
    }

    pub fn obj_to_vec3(yaml: &Hash) -> Option<Vector3> {
        let x = yaml.get_from_str("x")?.as_f32()?;
        let y = yaml.get_from_str("y")?.as_f32()?;
        let z = yaml.get_from_str("z")?.as_f32()?;
        Some(Vector3 { x, y, z })
    }

    pub fn obj_to_vec2(yaml: &Hash) -> Option<Vector2> {
        let x = yaml.get_from_str("x")?.as_f32()?;
        let y = yaml.get_from_str("y")?.as_f32()?;
        Some(Vector2 { x, y })
    }
}
