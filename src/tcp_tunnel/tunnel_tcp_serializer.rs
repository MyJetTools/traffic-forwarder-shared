use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};

use super::tunnel_tcp_contract::TunnelTcpContract;

const PING_PACKET: u8 = 0;
const PONG_PACKET: u8 = 1;
const CONNECT_PACKET: u8 = 2;
const CONNECTED_PACKET: u8 = 3;
const CAN_NOT_CONNECT_PACKET: u8 = 4;
const DISCONNECTED_PACKET: u8 = 5;
const PAYLOAD: u8 = 6;
const GREETING: u8 = 7;

pub struct TunnelTcpSerializer {}

impl TunnelTcpSerializer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<TunnelTcpContract> for TunnelTcpSerializer {
    fn serialize(&self, contract: TunnelTcpContract) -> Vec<u8> {
        self.serialize_ref(&contract)
    }
    fn serialize_ref(&self, contract: &TunnelTcpContract) -> Vec<u8> {
        match contract {
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

    fn get_ping(&self) -> TunnelTcpContract {
        TunnelTcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<TunnelTcpContract, ReadingTcpContractFail> {
        let packet_type = socket_reader.read_byte().await?;

        match packet_type {
            PING_PACKET => Ok(TunnelTcpContract::Ping),
            PONG_PACKET => Ok(TunnelTcpContract::Pong),
            CONNECT_PACKET => {
                let id = socket_reader.read_u32().await?;
                let url = crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(TunnelTcpContract::ConnectTo { id, url })
            }
            CONNECTED_PACKET => {
                let connection_id = socket_reader.read_u32().await?;
                Ok(TunnelTcpContract::Connected(connection_id))
            }
            CAN_NOT_CONNECT_PACKET => {
                let id = socket_reader.read_u32().await?;
                let reason = crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(TunnelTcpContract::CanNotConnect { id, reason })
            }
            DISCONNECTED_PACKET => {
                let id = socket_reader.read_u32().await?;
                Ok(TunnelTcpContract::Disconnected(id))
            }
            PAYLOAD => {
                let id = socket_reader.read_u32().await?;
                let payload = socket_reader.read_byte_array().await?;
                Ok(TunnelTcpContract::Payload { id, payload })
            }
            GREETING => {
                let handshake_phrase =
                    crate::common_deserializers::read_pascal_string(socket_reader).await?;
                Ok(TunnelTcpContract::Greeting(handshake_phrase))
            }
            _ => {
                panic!("Invalid packet type:{}", packet_type);
            }
        }
    }

    fn apply_packet(&mut self, _contract: &TunnelTcpContract) -> bool {
        true
    }
}
