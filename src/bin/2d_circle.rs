use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_whisker_reader::serial_plugin::*;

#[derive(Component)]
struct Target;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SerialPlugin))
        .add_systems(Startup, setup_2d)
        .add_systems(Update, get_sensor_sample)
        .run();
}

fn setup_2d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    }, Target));
}

fn get_sensor_sample(
    sensor_reader: Res<SensorReader>,
    mut target: Query<&mut Transform, With<Target>>
) {
    let mut transform = target.single_mut();
    for message in sensor_reader.0.drain() {
        let max_u16 = u16::MAX as f32;
        let x_norm: f32 = 2.0 * (message.x as f32 / max_u16) - 1.0;
        let y_norm: f32 = 2.0 * (message.y as f32 / max_u16) - 1.0;
        // let z_norm: f32 = 2.0 * (message.z as f32 / max_u16) - 1.0;
        let z_norm: f32 = message.z as f32 / max_u16;
        transform.translation.x = x_norm * 200.0;
        transform.translation.y = y_norm * 200.0;
        transform.scale = Vec3::ONE * (z_norm * 3.0);
        info!("{message:?}");
    }
}