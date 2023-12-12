use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;
use bevy::prelude::*;
use flume::Receiver;

#[derive(Debug)]
pub struct SensorSample {
    pub x: u16,
    pub y: u16,
    pub z: u16
}

#[derive(Resource)]
pub struct SensorReader(pub Receiver<SensorSample>);

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

pub struct SerialPlugin;

impl Plugin for SerialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_serialport);
    }
}