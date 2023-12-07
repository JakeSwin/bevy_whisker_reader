use bevy::prelude::*;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;
use flume::Receiver;

#[derive(Resource)]
struct SensorReader(Receiver<SensorSample>);

#[derive(Component)]
struct Target;

#[derive(Debug)]
struct SensorSample {
    x: u16,
    y: u16,
    z: u16
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_serialport, setup_3d))
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

fn setup_serialport(mut commands: Commands) {
    match serialport::available_ports() {
        Ok(ports) => {
            if ports.len() != 1 {
                return;
            }
            for port in ports {
                info!("{port:?}");
                let (tx, rx) = flume::unbounded::<SensorSample>();
                commands.insert_resource(SensorReader(rx));
                let _ = thread::spawn(move || {
                    let serial_port = serialport::new(port.port_name, 115200)
                        .timeout(Duration::from_millis(1))
                        .open()
                        .expect("Failed to open Serial Port");
                    let mut reader = BufReader::new(serial_port);
                    let mut buff = String::new();
                    loop {
                        match reader.read_line(&mut buff) {
                            Ok(_) => (),
                            _ => continue,
                        };
                        let split = buff.split(" ").into_iter().nth(0).unwrap();
                        // tx.send(buff.copy()).unwrap();
                        match hex::decode(split) {
                            Ok(decoded) => {
                                let buff_16 = decoded.as_slice();
                                let x = ((buff_16[1] as u16) << 8) | (buff_16[2] as u16);
                                let y = ((buff_16[3] as u16) << 8) | (buff_16[4] as u16);
                                let z = ((buff_16[5] as u16) << 8) | (buff_16[6] as u16);
                                // info!("Decoded: {decoded:?} - {x}")
                                tx.send(SensorSample { x, y, z}).unwrap();
                            },
                            Err(e) => error!("Could not decode {e:?}")
                        }
                        // info!("got sample {buff}");
                        buff.clear();
                    }
                });
            }
        },
        Err(_) => error!("No Ports Detected")
    }
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
        let z_norm: f32 = 2.0 * (message.z as f32 / max_u16) - 1.0;
        transform.translation.x = x_norm * 2.0;
        transform.translation.y = (z_norm * 2.0) + 0.5;
        transform.translation.z = y_norm * 2.0;
        info!("{message:?}");
    }
}