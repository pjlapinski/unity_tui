use crate::{
    class_id::CLASS_IDS,
    unity::{
        converters::{
            helpers::{obj_to_vec2, obj_to_vec3, obj_to_vec4},
            AsFileId, GetFromStr,
        },
        object::Field,
        yaml::YamlUnityDocument,
        Component, GameObject, Id, MonoBehaviour, Object, RectTransform, Transform, Transform3D,
    },
    util::hash_map,
};
use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use unity_yaml_rust::Yaml;

pub struct Repository(LinkedHashMap<Id, Object>);

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
            .filter(|transform| !transform.has_parent())
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

impl From<LinkedHashMap<Id, Object>> for Repository {
    fn from(map: LinkedHashMap<Id, Object>) -> Self {
        Self(map)
    }
}

pub fn construct_repository(yaml: Vec<YamlUnityDocument>) -> Option<Repository> {
    let mut repo = LinkedHashMap::<Id, Object>::new();
    for doc in yaml.iter() {
        if let Some(class_name) = CLASS_IDS.get(&doc.class_id) {
            match *class_name {
                "GameObject" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::GameObject(game_object_from_yaml(doc, class_name)?),
                    );
                }
                // TODO: this does not account for built-in components, i.e. Camera
                "MonoBehaviour" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::Component(Component::MonoBehaviour(monobehaviour_from_yaml(
                            doc, class_name,
                        )?)),
                    );
                }
                "Transform" | "RectTransform" => {
                    repo.insert(
                        doc.id.clone(),
                        Object::Component(Component::Transform(transform_from_yaml(
                            doc, class_name,
                        )?)),
                    );
                }
                _ => {} // TODO: other types of serialized entities, like RenderSettings
            }
        }
    }

    Some(repo.into())
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
            map.get_from_str("fileID")?.as_file_id()
        })
        .collect::<Option<Vec<Id>>>()?;

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

const LAST_COMMON_MONO_FIELD_NAME: &str = "m_EditorClassIdentifier";

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
                    if let Some(vec4) = obj_to_vec4(map) {
                        comp.fields.insert(key.to_owned(), Field::Vector4(vec4));
                    } else if let Some(vec3) = obj_to_vec3(map) {
                        comp.fields.insert(key.to_owned(), Field::Vector3(vec3));
                    } else if let Some(vec2) = obj_to_vec2(map) {
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
    let local_rotation = obj_to_vec4(map.get_from_str("m_LocalRotation")?.as_hash()?)?;
    let local_position = obj_to_vec3(map.get_from_str("m_LocalPosition")?.as_hash()?)?;
    let local_scale = obj_to_vec3(map.get_from_str("m_LocalScale")?.as_hash()?)?;
    let root_order = map.get_from_str("m_RootOrder")?.as_i64()?;
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
        root_order,
        father_id,
        children_ids,
        game_object_id,
    })
}

fn rect_transform_from_yaml(doc: &YamlUnityDocument, class_name: &str) -> Option<RectTransform> {
    let map = doc.document.as_hash()?;
    let map = map.get_from_str(class_name)?.as_hash()?;
    let local_rotation = obj_to_vec4(map.get_from_str("m_LocalRotation")?.as_hash()?)?;
    let local_position = obj_to_vec3(map.get_from_str("m_LocalPosition")?.as_hash()?)?;
    let local_scale = obj_to_vec3(map.get_from_str("m_LocalScale")?.as_hash()?)?;
    let anchor_min = obj_to_vec2(map.get_from_str("m_AnchorMin")?.as_hash()?)?;
    let anchor_max = obj_to_vec2(map.get_from_str("m_AnchorMax")?.as_hash()?)?;
    let anchored_position = obj_to_vec2(map.get_from_str("m_AnchoredPosition")?.as_hash()?)?;
    let size_delta = obj_to_vec2(map.get_from_str("m_SizeDelta")?.as_hash()?)?;
    let pivot = obj_to_vec2(map.get_from_str("m_Pivot")?.as_hash()?)?;
    let root_order = map.get_from_str("m_RootOrder")?.as_i64()?;
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
        root_order,
        father_id,
        children_ids,
        game_object_id,
    })
}
