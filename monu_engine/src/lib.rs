//! # Monu Engine
//! Игровой движок на базе Bevy ECS.
//!
//! Архитектура:
//! - `app`    — фабрика приложений (общая конфигурация для editor/game)
//! - `world`  — компоненты и плагины мира/сцены
//! - `camera` — камера редактора (WASD + мышь)
//! - `ecs`    — утилиты над Bevy ECS

pub mod app;
pub mod camera;
pub mod ecs;
pub mod world;

#[cfg(feature = "vulkan_raw")]
pub mod vulkan_ctx;

pub use app::MonuApp;
pub use world::WorldPlugin;

/// Версия движка
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
