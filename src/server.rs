use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use rosc::OscPacket;

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
                    Ok((size, addr)) => {
                        // println!("Received packet with size {} from: {}", size, addr);
                        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();

                        match packet {
                            OscPacket::Message(msg) => {
                                // println!("OSC address: {}", msg.addr);
                                // println!("OSC arguments: {:?}", msg.args);
                                if msg.addr == "/tracking/eye/LeftRightPitchYaw" {
                                    *eye_gaze_data.lock().unwrap() = EyeGazeData {
                                        l_pitch: msg.args[0].clone().float().unwrap(),
                                        l_yaw: msg.args[1].clone().float().unwrap(),
                                        r_pitch: msg.args[2].clone().float().unwrap(),
                                        r_yaw: msg.args[3].clone().float().unwrap(),
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
