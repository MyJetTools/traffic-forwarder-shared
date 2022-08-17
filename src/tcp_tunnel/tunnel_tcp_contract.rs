use my_tcp_sockets::tcp_connection::TcpContract;

pub enum TunnelTcpContract {
    Ping,
    Pong,
    ConnectTo { id: u32, url: String },
    Connected(u32),
    CanNotConnect { id: u32, reason: String },
    Disconnected(u32),
    Payload { id: u32, payload: Vec<u8> },
    Greeting(String),
}

impl TcpContract for TunnelTcpContract {
    fn is_pong(&self) -> bool {
        match self {
            TunnelTcpContract::Pong => true,
            _ => false,
        }
    }
}
