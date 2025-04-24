use std::net::UdpSocket;
use anyhow::Result;
use rosc::{OscMessage, OscPacket, OscType};

pub struct OscClient {
    socket: UdpSocket,
    addr: String,
}

impl OscClient {
    pub fn new(ip: &str, port: u16) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let addr = format!("{}:{}", ip, port);
        Ok(Self { socket, addr })
    }

    pub fn send_chatbox_message(&self, message: &str, _fx_sound: bool, slim_mode: bool) -> Result<()> {
        let mut msg = message.to_string();
        if slim_mode {
            msg.push_str("\u{0003}\u{001f}");
        }
        let msg_buf = rosc::encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/chatbox/input".to_string(),
            args: vec![
                OscType::String(msg),
                OscType::Bool(true),
                OscType::Bool(false),
            ],
        }))?;
        self.socket.send_to(&msg_buf, &self.addr)?;
        Ok(())
    }
}