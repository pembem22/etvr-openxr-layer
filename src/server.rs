use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use rosc::OscPacket;

#[derive(Debug)]
pub struct EyeGazeData {
    pub l_pitch: f32,
    pub r_pitch: f32,
    pub l_yaw: f32,
    pub r_yaw: f32,
    pub time: SystemTime,
}

pub struct OSCServer {
    pub eye_gaze_data: Arc<Mutex<EyeGazeData>>,
}

impl OSCServer {
    pub fn new() -> OSCServer {
        OSCServer {
            eye_gaze_data: Arc::new(Mutex::new(EyeGazeData {
                l_pitch: 0.0,
                r_pitch: 0.0,
                l_yaw: 0.0,
                r_yaw: 0.0,
                time: SystemTime::UNIX_EPOCH,
            })),
        }
    }

    pub fn run(&self) {
        let socket = UdpSocket::bind("0.0.0.0:9000").unwrap();
        let eye_gaze_data = self.eye_gaze_data.clone();

        std::thread::spawn(move || {
            println!("OSC socket loop");
            loop {
                // Receives a single datagram message on the socket. If `buf` is too small to hold
                // the message, it will be cut off.
                let mut buf = [0; rosc::decoder::MTU];
                match socket.recv_from(&mut buf) {
                    Ok((size, _addr)) => {
                        // println!("Received packet with size {} from: {}", size, addr);
                        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();

                        match packet {
                            OscPacket::Message(msg) => {
                                // println!("OSC address: {}", msg.addr);
                                // println!("OSC arguments: {:?}", msg.args);
                                if msg.addr == "/tracking/eye/LeftRightPitchYaw" {
                                    let [l_pitch, l_yaw, r_pitch, r_yaw]: [f32; 4] = msg
                                        .args
                                        .iter()
                                        .map(|a| a.clone().float().unwrap().to_radians() * -1.0)
                                        .collect::<Vec<f32>>()
                                        .try_into()
                                        .unwrap();

                                    *eye_gaze_data.lock().unwrap() = EyeGazeData {
                                        l_pitch,
                                        l_yaw,
                                        r_pitch,
                                        r_yaw,
                                        time: SystemTime::now(),
                                    }
                                }
                                if msg.addr == "/tracking/eye/LeftRightVec" {
                                    let [l_x, l_y, l_z]: [f32; 3] = msg.args[0..3]
                                        .iter()
                                        .map(|a| a.clone().float().unwrap())
                                        .collect::<Vec<f32>>()
                                        .try_into()
                                        .unwrap();
                                    let [r_x, r_y, r_z]: [f32; 3] = msg.args[3..6]
                                        .iter()
                                        .map(|a| a.clone().float().unwrap())
                                        .collect::<Vec<f32>>()
                                        .try_into()
                                        .unwrap();

                                    // println!();
                                    // println!("/tracking/eye/LeftRightVec");
                                    // println!("{:+2.3} {:+2.3} {:+2.3}", l_x, l_y, l_z);
                                    // println!("{:+2.3} {:+2.3} {:+2.3}", r_x, r_y, r_z);
                                    // println!(
                                    //     "{:?}",
                                    //     EyeGazeData {
                                    //         l_pitch: l_y.atan2((l_x * l_x + l_z * l_z).sqrt()),
                                    //         l_yaw: (-l_x).atan2(l_z),
                                    //         r_pitch: r_y.atan2((r_x * r_x + r_z * r_z).sqrt()),
                                    //         r_yaw: (-r_x).atan2(r_z),
                                    //         time: SystemTime::now(),
                                    //     }
                                    // );

                                    *eye_gaze_data.lock().unwrap() = EyeGazeData {
                                        l_pitch: l_y.atan2((l_x * l_x + l_z * l_z).sqrt()),
                                        l_yaw: (-l_x).atan2(l_z),
                                        r_pitch: r_y.atan2((r_x * r_x + r_z * r_z).sqrt()),
                                        r_yaw: (-r_x).atan2(r_z),
                                        time: SystemTime::now(),
                                    }
                                }
                            }
                            OscPacket::Bundle(bundle) => {
                                println!("OSC Bundle: {:?}", bundle);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error receiving from socket: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
