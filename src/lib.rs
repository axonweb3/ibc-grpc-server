//! # ibc-grpc-server
//!
//! `ibc-grpc-server` exposes the grpc Query service required by cosmos IBC standards. At present,
//! the query requirements of ics02_client、ics03_connection and ics04_channel are nearly ready.

pub mod error;
mod service;
pub mod types;

use ibc::core::ics02_client::{client_consensus::AnyConsensusState, client_state::AnyClientState};
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

/// Run the gRPC server
pub async fn run_ibc_grpc<Store>(store: Store, addr: String)
where
    Store: IbcStore + 'static,
{
    log::info!("Starting ibc grpc server.");
    IbcGrpcService::new(store, addr).run().await;
}

pub trait IbcStore: Sync + Send {
    /// Return an IBC light client state by height and path.
    fn get_client_state(
        &self,
        height: StoreHeight,
        path: &ClientStatePath,
    ) -> Result<Option<AnyClientState>>;

    /// Return a consensus state associated with a client state by height and path.
    fn get_consensus_state(
        &self,
        height: StoreHeight,
        path: &ClientConsensusStatePath,
    ) -> Result<Option<AnyConsensusState>>;

    /// Return an IBC connection end by height and path.
    fn get_connection_end(
        &self,
        height: StoreHeight,
        path: &ConnectionsPath,
    ) -> Result<Option<ConnectionEnd>>;

    /// Return the connection ids associated with a client state by height and path.
    fn get_connection_ids(
        &self,
        height: StoreHeight,
        path: &ClientConnectionsPath,
    ) -> Result<Vec<ConnectionId>>;

    /// Return the packet acknowledgement by height and path.
    fn get_acknowledgement_commitment(
        &self,
        height: StoreHeight,
        path: &AcksPath,
    ) -> Result<Option<AcknowledgementCommitment>>;

    /// Return an IBC Channel by height and path.
    fn get_channel_end(
        &self,
        height: StoreHeight,
        path: &ChannelEndsPath,
    ) -> Result<Option<ChannelEnd>>;

    fn get_opt(&self, height: StoreHeight, path: &ReceiptsPath) -> Result<Option<()>>;

    /// Return the packet commitment associated with a channel by height and path.
    fn get_packet_commitment(
        &self,
        height: StoreHeight,
        path: &CommitmentsPath,
    ) -> Result<Option<PacketCommitment>>;

    /// Return all paths with same prefix.
    fn get_paths_by_prefix(&self, key_prefix: &Path) -> Result<Vec<Path>>;

    /// Return the current height of the chain.
    fn current_height(&self) -> u64;
}
