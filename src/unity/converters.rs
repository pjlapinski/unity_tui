use crate::{
    class_id::CLASS_IDS,
    unity::{
        object::Field, yaml::YamlUnityDocument, Component, GameObject, Id, MonoBehaviour, Object,
        RectTransform, Transform, Transform3D,
    },
    util::hash_map,
};
use std::collections::{HashMap, HashSet};
use unity_yaml_rust::{yaml::Hash, Yaml};

const LAST_COMMON_MONO_FIELD_NAME: &str = "m_EditorClassIdentifier";

trait AsFileId {
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

trait AsF32 {
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

trait GetFromStr {
    fn get_from_str(&self, s: &str) -> Option<&Yaml>;
}

impl GetFromStr for Hash {
    fn get_from_str(&self, s: &str) -> Option<&Yaml> {
        self.get(&Yaml::String(s.to_owned()))
    }
}

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

pub fn construct_repository(yaml: Vec<YamlUnityDocument>) -> Repository {
    let mut repo = hash_map![];
    for doc in yaml.iter() {
        if let Some(class_name) = CLASS_IDS.get(&doc.class_id) {
            match *class_name {
                "GameObject" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::GameObject(game_object_from_yaml(doc, class_name).unwrap()),
                    );
                }
                // TODO: this does not account for built-in components, i.e. Camera
                "MonoBehaviour" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::Component(Component::MonoBehaviour(
                            monobehaviour_from_yaml(doc, class_name).unwrap(),
                        )),
                    );
                }
                "Transform" | "RectTransform" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::Component(Component::Transform(
                            transform_from_yaml(doc, class_name).unwrap(),
                        )),
                    );
                }
                _ => {} // TODO: other types of serialized entities, like RenderSettings
            }
        }
    }

    repo.into()
}

fn game_object_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<GameObject> {
    let map = doc.document.as_hash()?;
    let map = map.get_from_str(class_name)?.as_hash()?;
    let name = map.get_from_str("m_Name")?.as_str()?.to_owned();
    let active = map.get_from_str("m_IsActive")?.as_i64()? > 0;
    let layer = map.get_from_str("m_Layer")?.as_i64()? as u8;
    let tag = map.get_from_str("m_TagString")?.as_str()?.to_owned();
    let component_ids = map
        .get_from_str("m_Component")?
        .as_vec()?
        .to_vec()
        .iter()
        .map(|y| {
            let map = y
                .as_hash()
                .unwrap()
                .get_from_str("component")
                .unwrap()
                .as_hash()
                .unwrap();
            map.get_from_str("fileID").unwrap().as_file_id().unwrap()
        })
        .collect();

    Some(GameObject {
        id: doc.id.clone(),
        name,
        component_ids,
        active,
        layer,
        tag,
        transform_id: "".to_owned(),
    })
}

fn monobehaviour_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<MonoBehaviour> {
    let map = doc.document.as_hash()?;
    let map = map.get_from_str(class_name)?.as_hash()?;
    let mut past_common = false;
    let mut comp = MonoBehaviour {
        id: doc.id.clone(),
        enabled: false,
        fields: hash_map![],
        game_object_id: "".to_string(),
    };

    for (key, value) in map.iter() {
        let key = key.as_str()?;
        match key {
            "m_Enabled" => comp.enabled = value.as_i64()? > 0,
            "m_GameObject" => {
                comp.game_object_id = value.as_hash()?.get_from_str("fileID")?.as_file_id()?
            }
            LAST_COMMON_MONO_FIELD_NAME => past_common = true,
            _ if past_common => match value {
                Yaml::Hash(map) => {
                    if let Some(vec4) = helpers::obj_to_vec4(map) {
                        comp.fields.insert(key.to_owned(), Field::Vector4(vec4));
                    } else if let Some(vec3) = helpers::obj_to_vec3(map) {
                        comp.fields.insert(key.to_owned(), Field::Vector3(vec3));
                    } else if let Some(vec2) = helpers::obj_to_vec2(map) {
                        comp.fields.insert(key.to_owned(), Field::Vector2(vec2));
                    } else {
                        comp.fields
                            .insert(key.to_owned(), Field::Yaml(value.clone()));
                    }
                }
                Yaml::Real(f) => {
                    let f = f.parse();
                    assert!(f.is_ok());
                    comp.fields.insert(key.to_owned(), Field::F64(f.unwrap()));
                }
                Yaml::Integer(i) => {
                    comp.fields.insert(key.to_owned(), Field::I64(*i));
                }
                Yaml::String(s) => {
                    comp.fields.insert(key.to_owned(), Field::Str(s.clone()));
                }
                Yaml::Boolean(b) => {
                    comp.fields.insert(key.to_owned(), Field::Bool(*b));
                }
                Yaml::BadValue => return None,
                _ => {
                    comp.fields
                        .insert(key.to_owned(), Field::Yaml(value.clone()));
                }
            },
            _ => {}
        }
    }
    Some(comp)
}

fn transform_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<Transform> {
    match class_name {
        "Transform" => Some(Transform::Transform3D(transform_3d_from_yaml(
            doc, class_name,
        )?)),
        "RectTransform" => Some(Transform::RectTransform(rect_transform_from_yaml(
            doc, class_name,
        )?)),
        _ => None,
    }
}

fn transform_3d_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<Transform3D> {
    let map = doc.document.as_hash()?;
    let map = map.get_from_str(class_name)?.as_hash()?;
    let local_rotation = helpers::obj_to_vec4(map.get_from_str("m_LocalRotation")?.as_hash()?)?;
    let local_position = helpers::obj_to_vec3(map.get_from_str("m_LocalPosition")?.as_hash()?)?;
    let local_scale = helpers::obj_to_vec3(map.get_from_str("m_LocalScale")?.as_hash()?)?;
    let game_object_id = map
        .get_from_str("m_GameObject")?
        .as_hash()?
        .get_from_str("fileID")?
        .as_file_id()?;

    let father_id = map
        .get_from_str("m_Father")?
        .as_hash()?
        .get_from_str("fileID")?
        .as_file_id()?;

    let children_ids = map
        .get_from_str("m_Children")?
        .as_vec()?
        .to_vec()
        .iter()
        .map(|y| {
            y.as_hash()
                .unwrap()
                .get_from_str("fileID")
                .unwrap()
                .as_file_id()
                .unwrap()
        })
        .collect();

    Some(Transform3D {
        id: doc.id.clone(),
        local_rotation,
        local_position,
        local_scale,
        father_id,
        children_ids,
        game_object_id,
    })
}

fn rect_transform_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<RectTransform> {
    let map = doc.document.as_hash()?;
    let map = map.get_from_str(class_name)?.as_hash()?;
    let local_rotation = helpers::obj_to_vec4(map.get_from_str("m_LocalRotation")?.as_hash()?)?;
    let local_position = helpers::obj_to_vec3(map.get_from_str("m_LocalPosition")?.as_hash()?)?;
    let local_scale = helpers::obj_to_vec3(map.get_from_str("m_LocalScale")?.as_hash()?)?;
    let anchor_min = helpers::obj_to_vec2(map.get_from_str("m_AnchorMin")?.as_hash()?)?;
    let anchor_max = helpers::obj_to_vec2(map.get_from_str("m_AnchorMax")?.as_hash()?)?;
    let anchored_position =
        helpers::obj_to_vec2(map.get_from_str("m_AnchoredPosition")?.as_hash()?)?;
    let size_delta = helpers::obj_to_vec2(map.get_from_str("m_SizeDelta")?.as_hash()?)?;
    let pivot = helpers::obj_to_vec2(map.get_from_str("m_Pivot")?.as_hash()?)?;
    let game_object_id = map
        .get_from_str("m_GameObject")?
        .as_hash()?
        .get_from_str("fileID")?
        .as_file_id()?;

    let father_id = map
        .get_from_str("m_Father")?
        .as_hash()?
        .get_from_str("fileID")?
        .as_file_id()?;

    let children_ids = map
        .get_from_str("m_Children")?
        .as_vec()?
        .to_vec()
        .iter()
        .map(|y| {
            y.as_hash()
                .unwrap()
                .get_from_str("fileID")
                .unwrap()
                .as_file_id()
                .unwrap()
        })
        .collect();

    Some(RectTransform {
        id: doc.id.clone(),
        local_rotation,
        local_position,
        local_scale,
        anchor_min,
        anchor_max,
        anchored_position,
        size_delta,
        pivot,
        father_id,
        children_ids,
        game_object_id,
    })
}

pub struct Repository(HashMap<Id, Object>);

impl Repository {
    /// Returns all Ids that point to GameObjects
    pub fn get_game_object_ids(&self) -> HashSet<Id> {
        self.0
            .iter()
            .filter_map(|(id, obj)| match obj {
                Object::GameObject(_) => Some(id.clone()),
                _ => None,
            })
            .collect()
    }

    /// Returns all Ids of Transforms that do not have a parent
    pub fn get_unparented_transforms(&self) -> Vec<&Transform> {
        self.0
            .iter()
            .filter_map(|(_, obj)| match obj {
                Object::Component(Component::Transform(transform)) => Some(transform),
                _ => None,
            })
            .filter(|transform| transform.has_parent())
            .collect()
    }

    /// Returns a GameObject. Returns none if id was not found or if found object is not a GameObject
    pub fn get_game_object(&self, id: &Id) -> Option<&GameObject> {
        match self.get(id)? {
            Object::GameObject(obj) => Some(obj),
            _ => None,
        }
    }

    /// Returns a Component. Returns none if id was not found or if found object is not a Component
    pub fn get_component(&self, id: &Id) -> Option<&Component> {
        match self.get(id)? {
            Object::Component(comp) => Some(comp),
            _ => None,
        }
    }

    /// Returns a Transform. Returns none if id was not found or if found object is not a Transform
    pub fn get_transform(&self, id: &Id) -> Option<&Transform> {
        match self.get_component(id)? {
            Component::Transform(trans) => Some(trans),
            _ => None,
        }
    }

    /// Returns a MonoBehaviour. Returns none if id was not found or if found object is not a MonoBehaviour
    pub fn get_monobehaviour(&self, id: &Id) -> Option<&MonoBehaviour> {
        match self.get_component(id)? {
            Component::MonoBehaviour(mono) => Some(mono),
            _ => None,
        }
    }

    pub fn get(&self, id: &Id) -> Option<&Object> {
        self.0.get(id)
    }
}

impl From<HashMap<Id, Object>> for Repository {
    fn from(map: HashMap<Id, Object>) -> Self {
        Self(map)
    }
}

mod helpers {
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
