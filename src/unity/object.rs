use super::{
    vector::{Quaternion, Vector2, Vector3, Vector4},
    Id,
};
use crate::class_id::CLASS_IDS;
use std::{cmp::Ordering, collections::HashMap};
use unity_yaml_rust::Yaml;

pub trait GetId {
    fn get_id(&self) -> &Id;
}

#[derive(Debug, Clone)]
pub enum Object {
    GameObject(GameObject),
    Component(Component),
}

impl GetId for Object {
    fn get_id(&self) -> &Id {
        match self {
            Object::GameObject(go) => go.get_id(),
            Object::Component(c) => c.get_id(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: Id,
    pub name: String,
    pub component_ids: Vec<Id>,
    pub active: bool,
    pub layer: u8,
    pub tag: String,
    pub transform_id: Id,
}

impl GetId for GameObject {
    fn get_id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct MonoBehaviour {
    pub id: String,
    // pub name: String, // this needs to be read from a meta file
    pub enabled: bool,
    pub fields: HashMap<String, Field>,
    pub game_object_id: Id,
}

impl GetId for MonoBehaviour {
    fn get_id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub enum Component {
    MonoBehaviour(MonoBehaviour),
    Transform(Transform),
}

impl GetId for Component {
    fn get_id(&self) -> &Id {
        match self {
            Component::MonoBehaviour(c) => c.get_id(),
            Component::Transform(c) => c.get_id(),
        }
    }
}

impl Component {
    pub fn get_name(&self) -> String {
        match self {
            Component::MonoBehaviour(m) => m.id.clone(), // TODO: change this to component name, needs to be read from the meta file
            Component::Transform(t) => t.get_name(),
        }
    }

    pub fn get_game_object_id(&self) -> &Id {
        match self {
            Component::MonoBehaviour(m) => &m.game_object_id,
            Component::Transform(t) => t.get_game_object_id(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transform3D {
    pub id: Id,
    pub local_rotation: Quaternion,
    pub local_position: Vector3,
    pub local_scale: Vector3,
    pub root_order: i64,
    pub father_id: Id,
    pub children_ids: Vec<Id>,
    pub game_object_id: Id,
}

impl GetId for Transform3D {
    fn get_id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone, Default)]
pub struct RectTransform {
    pub id: Id,
    pub local_rotation: Quaternion,
    pub local_position: Vector3,
    pub local_scale: Vector3,
    pub anchor_min: Vector2,
    pub anchor_max: Vector2,
    pub anchored_position: Vector2,
    pub size_delta: Vector2,
    pub pivot: Vector2,
    pub root_order: i64,
    pub father_id: Id,
    pub children_ids: Vec<Id>,
    pub game_object_id: Id,
}

impl GetId for RectTransform {
    fn get_id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub enum Transform {
    Transform3D(Transform3D),
    RectTransform(RectTransform),
}

impl GetId for Transform {
    fn get_id(&self) -> &Id {
        match self {
            Transform::Transform3D(t) => t.get_id(),
            Transform::RectTransform(t) => t.get_id(),
        }
    }
}

impl Transform {
    pub fn get_name(&self) -> String {
        match self {
            Transform::Transform3D(_) => "Transform".to_owned(),
            Transform::RectTransform(_) => "RectTransform".to_owned(),
        }
    }

    pub fn get_children_ids(&self) -> &Vec<Id> {
        match self {
            Transform::Transform3D(t) => &t.children_ids,
            Transform::RectTransform(t) => &t.children_ids,
        }
    }

    pub fn get_father_id(&self) -> &Id {
        match self {
            Transform::Transform3D(t) => &t.father_id,
            Transform::RectTransform(t) => &t.father_id,
        }
    }

    pub fn get_game_object_id(&self) -> &Id {
        match self {
            Transform::Transform3D(t) => &t.game_object_id,
            Transform::RectTransform(t) => &t.game_object_id,
        }
    }

    pub fn has_parent(&self) -> bool {
        self.get_father_id() != "0"
    }

    pub fn get_root_order(&self) -> i64 {
        match self {
            Transform::Transform3D(t) => t.root_order,
            Transform::RectTransform(t) => t.root_order,
        }
    }

    pub fn partial_cmp_by_root_order(&self, other: &Transform) -> Ordering {
        self.get_root_order()
            .partial_cmp(&other.get_root_order())
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Field {
    Vector2(Vector2),
    Vector3(Vector3),
    Vector4(Vector4),
    F64(f64),
    I64(i64),
    Str(String),
    Bool(bool),
    Yaml(Yaml),
}
