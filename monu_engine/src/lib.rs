//! # Monu Engine
//! Игровой движок на базе Bevy ECS.
//!
//! Архитектура:
//! - `app`    — фабрика приложений (общая конфигурация для editor/game)
//! - `world`  — компоненты и плагины мира/сцены
//! - `camera` — камера редактора (WASD + мышь)
//! - `ecs`    — утилиты над Bevy ECS
//! - `commands`  — команды
//! - `gizmos`  — гизмо

pub mod app;
pub mod camera;
pub mod commands;
pub mod ecs;
pub mod gizmos;
pub mod settings;
pub mod vulkan_ctx;
pub mod world;

pub use app::{AppConfig, MonuApp};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
