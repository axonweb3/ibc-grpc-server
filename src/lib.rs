pub mod error;
mod service;
pub mod types;

use ibc::core::ics02_client::{client_state::ClientState, consensus_state::ConsensusState};
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentsPath, ConnectionsPath, ReceiptsPath,
};

use crate::error::ServerError;
use crate::service::IbcGrpcService;
use crate::types::{Path, StoreHeight};

pub type Result<T> = std::result::Result<T, ServerError>;

pub async fn run_ibc_grpc<Store>(store: Store, addr: String)
where
    Store: IbcStore + 'static,
{
    log::info!("ibc start");
    IbcGrpcService::new(store, addr).run().await;
}

pub trait IbcStore: Sync + Send {
    fn get_client_state(
        &self,
        height: StoreHeight,
        path: &ClientStatePath,
    ) -> Result<Option<Box<dyn ClientState>>>;

    fn get_consensus_state(
        &self,
        height: StoreHeight,
        path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>>;

    fn get_connection_end(
        &self,
        height: StoreHeight,
        path: &ConnectionsPath,
    ) -> Result<Option<ConnectionEnd>>;

    fn get_connection_ids(
        &self,
        height: StoreHeight,
        path: &ClientConnectionsPath,
    ) -> Result<Vec<ConnectionId>>;

    fn get_acknowledgement_commitment(
        &self,
        height: StoreHeight,
        path: &AcksPath,
    ) -> Result<Option<AcknowledgementCommitment>>;

    fn get_channel_end(
        &self,
        height: StoreHeight,
        path: &ChannelEndsPath,
    ) -> Result<Option<ChannelEnd>>;

    fn get_opt(&self, height: StoreHeight, path: &ReceiptsPath) -> Result<Option<()>>;

    fn get_packet_commitment(
        &self,
        height: StoreHeight,
        path: &CommitmentsPath,
    ) -> Result<Option<PacketCommitment>>;

    fn get_paths_by_prefix(&self, key_prefix: &Path) -> Result<Vec<Path>>;

    fn current_height(&self) -> u64;
}
