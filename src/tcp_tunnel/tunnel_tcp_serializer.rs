use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};

use super::tunnel_tcp_contract::TunnelTcpContract;

pub struct TunnelTcpSerializer {}

impl TunnelTcpSerializer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<TunnelTcpContract> for TunnelTcpSerializer {
    fn serialize(&self, contract: TunnelTcpContract) -> Vec<u8> {
        contract.serialize()
    }

    fn get_ping(&self) -> TunnelTcpContract {
        TunnelTcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<TunnelTcpContract, ReadingTcpContractFail> {
        TunnelTcpContract::deserialize(socket_reader).await
    }
}
