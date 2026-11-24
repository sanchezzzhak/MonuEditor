//! Камера редактора: свободный полёт (WASD + зажатая ПКМ).

use bevy::prelude::*;
use bevy::window::CursorMoved;
use std::f32::consts::FRAC_PI_2;
use bevy::input::mouse::MouseWheel;


/// Плагин камеры редактора
pub struct EditorCameraPlugin;

impl Plugin for EditorCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fly_camera_movement, fly_camera_look))
            .register_type::<FlyCamera>();
    }
}

/// Компонент-маркер полётной камеры + настройки скорости
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlyCamera {
    /// Скорость, м/с
    pub speed: f32,
    /// Множитель ускорения (Shift)
    pub sprint_multiplier: f32,
    /// Чувствительность мыши
    pub sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sprint_multiplier: 3.0,
            sensitivity: 0.0022,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

fn axis(pressed_pos: bool, pressed_neg: bool) -> f32 {
    pressed_pos as i8 as f32 - pressed_neg as i8 as f32
}

fn fly_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut q: Query<(&mut Transform, &mut FlyCamera), With<FlyCamera>>,
    mut wheel_events: MessageReader<MouseWheel>
) {
    let forward = axis(keys.pressed(KeyCode::KeyW), keys.pressed(KeyCode::KeyS));
    let right = axis(keys.pressed(KeyCode::KeyD), keys.pressed(KeyCode::KeyA));
    let up = axis(keys.pressed(KeyCode::Space), keys.pressed(KeyCode::ControlLeft));

    if forward == 0.0 && right == 0.0 && up == 0.0 {
        return;
    }

    let wheel_delta: f32 = wheel_events.read().map(|ev| ev.y).sum();
    if mouse.pressed(MouseButton::Right) && wheel_delta != 0.0 {
        for (_, mut cam) in &mut q {
            let factor = if wheel_delta > 0.0 { 1.25 } else { 0.8 };
            cam.speed = (cam.speed * factor).clamp(0.5, 2000.0);
        }
    }

    for (mut tf, cam) in &mut q {
        // Движение разрешаем только при зажатой ПКМ — как в Unreal
        if !mouse.pressed(MouseButton::Right) {
            continue;
        }
        let sprint = if keys.pressed(KeyCode::ShiftLeft) { cam.sprint_multiplier } else { 1.0 };
        let dir = tf.rotation * Vec3::new(right, up, -forward).normalize_or_zero();
        tf.translation += dir * cam.speed * sprint * time.delta_secs();
    }
}

fn fly_camera_look(
    mouse: Res<ButtonInput<MouseButton>>,
    mut events: MessageReader<CursorMoved>,
    mut q: Query<(&mut Transform, &mut FlyCamera), With<FlyCamera>>,
) {
    if !mouse.pressed(MouseButton::Right) {
        return;
    }
    let Some(delta) = events.read().last().and_then(|ev| ev.delta) else { return };

    for (mut tf, mut cam) in &mut q {
        cam.yaw -= delta.x * cam.sensitivity;
        cam.pitch -= delta.y * cam.sensitivity;
        cam.pitch = cam.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
        tf.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
    }
}
