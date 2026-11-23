//! Утилиты над Bevy ECS: теги, спавн примитивов, будущий selection/undo.

use bevy::prelude::*;

use crate::world::{LevelObject, Static};

/// Расширение для EntityCommands: помечать сущности тегами движка
pub trait EntityTagExt {
    fn tag_as_static(&mut self) -> &mut Self;
    fn tag_as_level_object(&mut self) -> &mut Self;
}

impl EntityTagExt for EntityCommands<'_> {
    fn tag_as_static(&mut self) -> &mut Self {
        self.insert(Static);
        self
    }
    fn tag_as_level_object(&mut self) -> &mut Self {
        self.insert(LevelObject);
        self
    }
}

/// Помощники спавна примитивной геометрии (куб/плоскость) —
/// используются редактором при создании объектов из меню "Create".
pub struct Primitives;

impl Primitives {
    pub fn cube<'w>(
        commands: &'w mut Commands,
        meshes: &'w mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        name: &str,
        size: Vec3,
        pos: Vec3,
    ) -> EntityCommands<'w> {
        let mesh = meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)));
        let mat = materials.add(Color::srgb(0.55, 0.55, 0.6));
        commands.spawn((
            Name::new(name.to_string()),
            LevelObject,
            Static,
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(pos),
        ))
    }

    pub fn plane<'w>(
        commands: &'w mut Commands,
        meshes: &'w mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        size: f32,
    ) -> EntityCommands<'w> {
        let mesh = meshes.add(Mesh::from(Plane3d::default().mesh().size(size, size)));
        let mat = materials.add(Color::srgb(0.2, 0.2, 0.22));
        commands.spawn((
            Name::new("Ground"),
            LevelObject,
            Static,
            Mesh3d(mesh),
            MeshMaterial3d(mat),
        ))
    }
}
