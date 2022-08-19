use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    tcp_connection::TcpContract,
};

use crate::common_deserializers;

const PING_PACKET: u8 = 0;
const PONG_PACKET: u8 = 1;
const CONNECT_PACKET: u8 = 2;
const CONNECTED_PACKET: u8 = 3;
const CAN_NOT_CONNECT_PACKET: u8 = 4;
const DISCONNECTED_A_PACKET: u8 = 5;
const DISCONNECTED_B_PACKET: u8 = 6;
const PAYLOAD: u8 = 7;
const GREETING: u8 = 8;

pub enum TunnelTcpContract {
    Ping,
    Pong,
    ConnectTo { id: u32, url: String },
    Connected(u32),
    CanNotConnect { id: u32, reason: String },
    DisconnectedFromSideA(u32),
    DisconnectedFromSideB(u32),
    Payload { id: u32, payload: Vec<u8> },
    Greeting(String),
}

impl TunnelTcpContract {
    pub fn get_packet_name(&self) -> String {
        match self {
            TunnelTcpContract::Ping => "Ping".to_string(),
            TunnelTcpContract::Pong => "Pong".to_string(),
            TunnelTcpContract::ConnectTo { id, url } => format!("ConnectTo: {}/{}", id, url),
            TunnelTcpContract::Connected(id) => format!("Connected:{}", id),
            TunnelTcpContract::CanNotConnect { id, reason } => {
                format!("CanNptConnect:{}. Reason:{}", id, reason)
            }
            TunnelTcpContract::DisconnectedFromSideA(id) => {
                format!("Disconnected from side A {}", id)
            }
            TunnelTcpContract::DisconnectedFromSideB(id) => {
                format!("Disconnected from side B {}", id)
            }
            TunnelTcpContract::Payload { id, payload } => {
                format!("Payload from id {} with len:{}", id, payload.len())
            }
            TunnelTcpContract::Greeting(name) => {
                format!("Greeting from {}", name)
            }
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            TunnelTcpContract::Ping => {
                let mut data = Vec::with_capacity(1);
                data.push(PING_PACKET);
                data
            }
            TunnelTcpContract::Pong => {
                let mut data = Vec::with_capacity(1);
                data.push(PONG_PACKET);
                data
            }
            TunnelTcpContract::ConnectTo { id, url } => {
                let mut result = Vec::with_capacity(264);
                result.push(CONNECT_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                crate::common_serializers::serialize_pascal_string(&mut result, url);
                result
            }
            TunnelTcpContract::Connected(id) => {
                let mut result = Vec::with_capacity(9);
                result.push(CONNECTED_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                result
            }
            TunnelTcpContract::CanNotConnect { id, reason } => {
                let mut result = Vec::with_capacity(5 + 256);
                result.push(CAN_NOT_CONNECT_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                crate::common_serializers::serialize_pascal_string(&mut result, reason);
                result
            }
            TunnelTcpContract::DisconnectedFromSideA(id) => {
                let mut result = Vec::with_capacity(5);
                result.push(DISCONNECTED_A_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                result
            }
            TunnelTcpContract::DisconnectedFromSideB(id) => {
                let mut result = Vec::with_capacity(5);
                result.push(DISCONNECTED_B_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                result
            }
            TunnelTcpContract::Payload { id, payload } => {
                let mut result = Vec::with_capacity(5 + payload.len());
                result.push(PAYLOAD);
                crate::common_serializers::serialize_u32(&mut result, *id);
                crate::common_serializers::serialize_payload(&mut result, payload);
                result
            }

            TunnelTcpContract::Greeting(handshake_phrase) => {
                let mut result = Vec::with_capacity(2 + handshake_phrase.len());
                result.push(GREETING);

                crate::common_serializers::serialize_pascal_string(&mut result, handshake_phrase);
                result
            }
        }
    }

    pub async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        socket_reader: &mut TSocketReader,
    ) -> Result<Self, ReadingTcpContractFail> {
        let packet_type = socket_reader.read_byte().await?;

        match packet_type {
            PING_PACKET => Ok(Self::Ping),
            PONG_PACKET => Ok(Self::Pong),
            CONNECT_PACKET => {
                let id = socket_reader.read_u32().await?;
                let url = crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(Self::ConnectTo { id, url })
            }
            CONNECTED_PACKET => {
                let connection_id = socket_reader.read_u32().await?;
                Ok(Self::Connected(connection_id))
            }
            CAN_NOT_CONNECT_PACKET => {
                let id = socket_reader.read_u32().await?;
                let reason = crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(Self::CanNotConnect { id, reason })
            }
            DISCONNECTED_A_PACKET => {
                let id = socket_reader.read_u32().await?;
                Ok(Self::DisconnectedFromSideA(id))
            }
            DISCONNECTED_B_PACKET => {
                let id = socket_reader.read_u32().await?;
                Ok(Self::DisconnectedFromSideB(id))
            }
            PAYLOAD => {
                let id = socket_reader.read_u32().await?;
                let payload = common_deserializers::read_payload(socket_reader).await?;
                Ok(Self::Payload { id, payload })
            }
            GREETING => {
                let handshake_phrase =
                    crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(Self::Greeting(handshake_phrase))
            }
            _ => {
                panic!("Invalid packet type:{}", packet_type);
            }
        }
    }
}

impl TcpContract for TunnelTcpContract {
    fn is_pong(&self) -> bool {
        match self {
            TunnelTcpContract::Pong => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    use super::TunnelTcpContract;

    #[tokio::test]
    async fn test_connect() {
        let connect = TunnelTcpContract::ConnectTo {
            id: 5,
            url: "test:8080".to_string(),
        };

        let payload = connect.serialize();

        let mut socket_reader = SocketReaderInMem::new(payload);

        let result = TunnelTcpContract::deserialize(&mut socket_reader)
            .await
            .unwrap();

        if let TunnelTcpContract::ConnectTo { id, url } = result {
            assert_eq!(id, 5);
            assert_eq!(url, "test:8080");
        } else {
            panic!("Invalid contract");
        }
    }

    #[tokio::test]
    async fn test_connected() {
        let connect = TunnelTcpContract::Connected(6);

        let payload = connect.serialize();

        let mut socket_reader = SocketReaderInMem::new(payload);

        let result = TunnelTcpContract::deserialize(&mut socket_reader)
            .await
            .unwrap();

        if let TunnelTcpContract::Connected(result) = result {
            assert_eq!(result, 6);
        } else {
            panic!("Invalid contract");
        }
    }

    #[tokio::test]
    async fn test_can_no_connect() {
        let connect = TunnelTcpContract::CanNotConnect {
            id: 10,
            reason: "Reason".to_string(),
        };

        let payload = connect.serialize();

        let mut socket_reader = SocketReaderInMem::new(payload);

        let result = TunnelTcpContract::deserialize(&mut socket_reader)
            .await
            .unwrap();

        if let TunnelTcpContract::CanNotConnect { id, reason } = result {
            assert_eq!(id, 10);
            assert_eq!(reason, "Reason");
        } else {
            panic!("Invalid contract");
        }
    }

    #[tokio::test]
    async fn test_connect_and_ping() {
        let connect = TunnelTcpContract::ConnectTo {
            id: 5,
            url: "test:8080".to_string(),
        };

        let mut payload = connect.serialize();

        let ping = TunnelTcpContract::Ping;

        let ping_paylod = ping.serialize();
        payload.extend_from_slice(&ping_paylod);

        let mut socket_reader = SocketReaderInMem::new(payload);

        let result = TunnelTcpContract::deserialize(&mut socket_reader)
            .await
            .unwrap();

        if let TunnelTcpContract::ConnectTo { id, url } = result {
            assert_eq!(id, 5);
            assert_eq!(url, "test:8080");
        } else {
            panic!("Invalid contract");
        }

        let result = TunnelTcpContract::deserialize(&mut socket_reader)
            .await
            .unwrap();

        if let TunnelTcpContract::Ping = result {
        } else {
            panic!("Invalid contract");
        }
    }
}
