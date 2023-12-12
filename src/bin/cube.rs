use bevy::prelude::*;
use bevy_whisker_reader::serial_plugin::*;

#[derive(Component)]
struct Target;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SerialPlugin))
        .add_systems(Startup, setup_3d)
        .add_systems(Update, get_sensor_sample)
        .run();
}

fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    }, Target));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
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
        transform.translation.x = x_norm * 2.0;
        transform.translation.y = (z_norm * 2.0) + 0.5;
        transform.translation.z = y_norm * 2.0;
        info!("{message:?}");
    }
}