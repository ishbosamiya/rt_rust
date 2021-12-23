use blend::Instance;

pub mod id;
pub mod mesh;
pub mod object;

pub trait FromBlend {
    fn from_blend_instance(instance: &Instance) -> Option<Self>
    where
        Self: std::marker::Sized;
}
