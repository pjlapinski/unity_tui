use std::fmt::Debug;

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
pub type Quaternion = Vector4;

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vec3<T>
where
    T: Debug + Default + Copy + Clone + PartialEq + PartialOrd,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vec2<T>
where
    T: Debug + Default + Copy + Clone + PartialEq + PartialOrd,
{
    pub x: T,
    pub y: T,
}

pub type Vector3 = Vec3<f32>;
#[allow(dead_code)]
pub type Vector3Int = Vec3<i32>;
pub type Vector2 = Vec2<f32>;
#[allow(dead_code)]
pub type Vector2Int = Vec2<i32>;
