use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    tcp_connection::TcpContract,
};

const PING_PACKET: u8 = 0;
const PONG_PACKET: u8 = 1;
const CONNECT_PACKET: u8 = 2;
const CONNECTED_PACKET: u8 = 3;
const CAN_NOT_CONNECT_PACKET: u8 = 4;
const DISCONNECTED_PACKET: u8 = 5;
const PAYLOAD: u8 = 6;
const GREETING: u8 = 7;

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

impl TunnelTcpContract {
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
            TunnelTcpContract::Disconnected(id) => {
                let mut result = Vec::with_capacity(5);
                result.push(DISCONNECTED_PACKET);
                crate::common_serializers::serialize_u32(&mut result, *id);
                result
            }
            TunnelTcpContract::Payload { id, payload } => {
                let mut result = Vec::with_capacity(5 + payload.len());
                result.push(PAYLOAD);
                crate::common_serializers::serialize_u32(&mut result, *id);
                result.extend_from_slice(payload);
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
            DISCONNECTED_PACKET => {
                let id = socket_reader.read_u32().await?;
                Ok(Self::Disconnected(id))
            }
            PAYLOAD => {
                let id = socket_reader.read_u32().await?;
                let payload = socket_reader.read_byte_array().await?;
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
}
