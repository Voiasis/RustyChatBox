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
    pub fn send_chatbox_message(&self, message: &str, fx_sound: bool) -> Result<()> {
        let msg_buf = rosc::encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/chatbox/input".to_string(),
            args: vec![
                OscType::String(message.to_string()),
                OscType::Bool(true),
                OscType::Bool(fx_sound),
            ],
        }))?;
        self.socket.send_to(&msg_buf, &self.addr)?;
        Ok(())
    }
}