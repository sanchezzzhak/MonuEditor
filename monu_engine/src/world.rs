//! Мир и сцена: базовые компоненты любого уровня.

use bevy::prelude::*;

use crate::ecs::EntityTagExt;

/// Плагин мира: компоненты, теги, ресурсы состояния сцены
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneState>()
            .add_systems(Update, tag_fallback);
    }
}

/// Текущее состояние загруженной сцены
#[derive(Resource, Default)]
pub struct SceneState {
    pub loaded: bool,
    pub name: Option<String>,
}

/// Тег: объект принадлежит уровню (виден в hierarchy-панели редактора)
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct LevelObject;

/// Тег: статическая геометрия (не двигается, батчится)
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Static;

/// Тег: освещаемый динамический объект
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Dynamic;

/// Заглушка: всё, что имеет Transform и имя, но не помечено — считаем статикой.
/// Позже здесь будет реальная классификация для батчинга/окклюзии.
fn tag_fallback(
    mut commands: Commands,
    q: Query<Entity, (With<Name>, With<Transform>, Without<Static>, Without<Dynamic>, Without<LevelObject>)>,
) {
    for e in &q {
        commands.entity(e).tag_as_static();
    }
}

impl WorldPlugin {
    /// Зарегистрировать все отражённые (Reflect) типы движка в мире TypeRegistry.
    /// Нужно для инспектора редактора и сериализации сцен.
    pub fn register_types(app: &mut App) {
        app.register_type::<LevelObject>()
            .register_type::<Static>()
            .register_type::<Dynamic>();
    }
}
