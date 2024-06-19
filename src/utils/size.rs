use bevy::prelude::*;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Size(pub Vec2);

impl Size {
    #[allow(unused)]
    pub const fn square(size: f32) -> Self {
        Size(Vec2::new(size, size))
    }
    #[allow(unused)]
    pub const fn width(&self) -> f32 {
        self.0.x
    }

    #[allow(unused)]
    pub const fn height(&self) -> f32 {
        self.0.y
    }
}
