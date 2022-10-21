use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use ibc::core::ics24_host::{path, Path as IbcPath};

use ibc_proto::ibc::core::{
    channel::v1::{
        query_server::{Query as ChannelQuery, QueryServer as ChannelQueryServer},
        PacketState, QueryChannelClientStateRequest, QueryChannelClientStateResponse,
        QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
        QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
        QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
        QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
        QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
        QueryPacketAcknowledgementsRequest, QueryPacketAcknowledgementsResponse,
        QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
        QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
        QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
        QueryUnreceivedPacketsResponse,
    },
    client::v1::{
        query_server::{Query as ClientQuery, QueryServer as ClientQueryServer},
        ConsensusStateWithHeight, Height, IdentifiedClientState, QueryClientParamsRequest,
        QueryClientParamsResponse, QueryClientStateRequest, QueryClientStateResponse,
        QueryClientStatesRequest, QueryClientStatesResponse, QueryClientStatusRequest,
        QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
        QueryConsensusStateHeightsResponse, QueryConsensusStateRequest,
        QueryConsensusStateResponse, QueryConsensusStatesRequest, QueryConsensusStatesResponse,
        QueryUpgradedClientStateRequest, QueryUpgradedClientStateResponse,
        QueryUpgradedConsensusStateRequest, QueryUpgradedConsensusStateResponse,
    },
    connection::v1::{
        query_server::{Query as ConnectionQuery, QueryServer as ConnectionQueryServer},
        IdentifiedConnection as RawIdentifiedConnection, QueryClientConnectionsRequest,
        QueryClientConnectionsResponse, QueryConnectionClientStateRequest,
        QueryConnectionClientStateResponse, QueryConnectionConsensusStateRequest,
        QueryConnectionConsensusStateResponse, QueryConnectionRequest, QueryConnectionResponse,
        QueryConnectionsRequest, QueryConnectionsResponse,
    },
};

use tonic::{transport::Server, Request, Response, Status};

use crate::{IbcStore, Path, StoreHeight};

pub const CHAIN_REVISION_NUMBER: u64 = 0;

pub struct IbcGrpcService<Store: IbcStore> {
    store: Arc<Store>,
    addr: SocketAddr,
}

impl<Store> IbcGrpcService<Store>
where
    Store: IbcStore + 'static,
{
    pub fn new(store: Store, addr: String) -> Self {
        IbcGrpcService {
            store: Arc::new(store),
            addr: addr.parse().unwrap(),
        }
    }

    pub async fn run(self) {
        log::info!("ibc run");

        let ibc_client_service = self.client_service();
        let ibc_conn_service = self.connection_service();
        let ibc_channel_service = self.channel_service();

        Server::builder()
            .add_service(ibc_client_service)
            .add_service(ibc_conn_service)
            .add_service(ibc_channel_service)
            .serve(self.addr)
            .await
            .unwrap();
    }

    pub fn client_service(&self) -> ClientQueryServer<IbcClientService<Store>> {
        ClientQueryServer::new(IbcClientService::new(Arc::clone(&self.store)))
    }

    pub fn connection_service(&self) -> ConnectionQueryServer<IbcConnectionService<Store>> {
        ConnectionQueryServer::new(IbcConnectionService::new(Arc::clone(&self.store)))
    }

    pub fn channel_service(&self) -> ChannelQueryServer<IbcChannelService<Store>> {
        ChannelQueryServer::new(IbcChannelService::new(Arc::clone(&self.store)))
    }
}

pub struct IbcClientService<Store: IbcStore> {
    store: Arc<Store>,
}

impl<Store: IbcStore> IbcClientService<Store> {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl<Store: IbcStore + 'static> ClientQuery for IbcClientService<Store> {
    /// Queries an IBC light client.
    async fn client_state(
        &self,
        _request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        unimplemented!()
    }

    /// Queries all the IBC light clients of a chain.
    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        log::info!("Got client states request: {:?}", request);

        let path = "clients"
            .to_owned()
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{:?}", e)))?;

        let client_state_paths = |path: Path| -> Option<path::ClientStatePath> {
            match path.try_into() {
                Ok(IbcPath::ClientState(p)) => Some(p),
                _ => None,
            }
        };

        let keys = self
            .store
            .get_paths_by_prefix(&path)
            .map_err(Status::internal)?;
        let mut client_states = Vec::with_capacity(keys.len());

        // Todo: fixme after the light client state defined.
        for path in keys.into_iter().filter_map(client_state_paths) {
            client_states.push(
                self.store
                    .get_client_state(StoreHeight::Latest, &path)
                    .map(|_client_state| IdentifiedClientState {
                        client_id: path.0.to_string(),
                        client_state: None,
                    })
                    .map_err(Status::data_loss)?,
            );
        }

        Ok(Response::new(QueryClientStatesResponse {
            client_states,
            pagination: None,
        }))
    }

    /// Queries a consensus state associated with a client state at
    /// a given height.
    async fn consensus_state(
        &self,
        _request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        unimplemented!()
    }

    /// Queries all the consensus state associated with a given
    /// client.
    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        log::info!("Got consensus states request: {:?}", request);

        let path = format!("clients/{}/consensusStates", request.get_ref().client_id)
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{:?}", e)))?;

        let keys = self
            .store
            .get_paths_by_prefix(&path)
            .map_err(Status::internal)?;
        let mut consensus_states = Vec::with_capacity(keys.len());

        // Todo: fixme after light client consensus state defined.
        for path in keys.into_iter() {
            if let Ok(IbcPath::ClientConsensusState(path)) = path.try_into() {
                let _consensus_state = self
                    .store
                    .get_consensus_state(StoreHeight::Latest, &path)
                    .map_err(Status::data_loss)?;
                consensus_states.push(ConsensusStateWithHeight {
                    height: Some(Height {
                        revision_number: path.epoch,
                        revision_height: path.height,
                    }),
                    consensus_state: None,
                });
            } else {
                panic!("unexpected path")
            }
        }

        Ok(Response::new(QueryConsensusStatesResponse {
            consensus_states,
            pagination: None,
        }))
    }

    /// Queries the height of every consensus states associated with a given client.
    async fn consensus_state_heights(
        &self,
        _request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        unimplemented!()
    }

    /// Queries the status of an IBC client.
    async fn client_status(
        &self,
        _request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        unimplemented!()
    }

    /// Queries all parameters of the ibc client.
    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        unimplemented!()
    }

    /// Queries an Upgraded IBC light client.
    async fn upgraded_client_state(
        &self,
        _request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
        unimplemented!()
    }

    /// Queries an Upgraded IBC consensus state.
    async fn upgraded_consensus_state(
        &self,
        _request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        unimplemented!()
    }
}

pub struct IbcConnectionService<Store: IbcStore> {
    connection_end_adapter: Arc<Store>,
    connection_ids_adapter: Arc<Store>,
}

impl<Store: IbcStore> IbcConnectionService<Store> {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            connection_end_adapter: Arc::clone(&store),
            connection_ids_adapter: Arc::clone(&store),
        }
    }
}

#[tonic::async_trait]
impl<Store: IbcStore + 'static> ConnectionQuery for IbcConnectionService<Store> {
    /// Queries an IBC connection end.
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        let conn_id = ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|_| Status::invalid_argument("invalid connection id"))?;
        let conn: Option<ConnectionEnd> = self
            .connection_end_adapter
            .get_connection_end(StoreHeight::Latest, &path::ConnectionsPath(conn_id))
            .map_err(Status::data_loss)?;
        Ok(Response::new(QueryConnectionResponse {
            connection: conn.map(|c| c.into()),
            proof: vec![],
            proof_height: None,
        }))
    }

    /// Queries all the IBC connections of a chain.
    async fn connections(
        &self,
        _request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        let connection_path_prefix: Path = String::from("connections")
            .try_into()
            .expect("'connections' expected to be a valid Path");

        let connection_paths = self
            .connection_end_adapter
            .get_paths_by_prefix(&connection_path_prefix)
            .map_err(Status::internal)?;

        let mut identified_connections: Vec<RawIdentifiedConnection> =
            Vec::with_capacity(connection_paths.len());

        for path in connection_paths.into_iter() {
            match path.try_into() {
                Ok(IbcPath::Connections(connections_path)) => {
                    let connection_end = self
                        .connection_end_adapter
                        .get_connection_end(StoreHeight::Latest, &connections_path)
                        .map_err(Status::data_loss)?;
                    identified_connections.push(
                        IdentifiedConnectionEnd::new(connections_path.0, connection_end.unwrap())
                            .into(),
                    );
                }
                _ => panic!("unexpected path"),
            }
        }

        Ok(Response::new(QueryConnectionsResponse {
            connections: identified_connections,
            pagination: None,
            height: None,
        }))
    }

    /// Queries the connection paths associated with a client state.
    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        let client_id = request
            .get_ref()
            .client_id
            .parse()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))?;
        let path = path::ClientConnectionsPath(client_id);
        let connection_ids = self
            .connection_ids_adapter
            .get_connection_ids(StoreHeight::Latest, &path)
            .unwrap_or_default()
            .iter()
            .map(|conn_id| conn_id.to_string())
            .collect();

        Ok(Response::new(QueryClientConnectionsResponse {
            connection_paths: connection_ids,
            proof: vec![],
            proof_height: None,
        }))
    }

    /// Queries the client state associated with the connection.
    async fn connection_client_state(
        &self,
        _request: Request<QueryConnectionClientStateRequest>,
    ) -> Result<Response<QueryConnectionClientStateResponse>, Status> {
        todo!()
    }

    /// Queries the consensus state associated with the connection.
    async fn connection_consensus_state(
        &self,
        _request: Request<QueryConnectionConsensusStateRequest>,
    ) -> Result<Response<QueryConnectionConsensusStateResponse>, Status> {
        todo!()
    }
}

pub struct IbcChannelService<Store: IbcStore> {
    channel_end_adapter: Arc<Store>,
    packet_commitment_adapter: Arc<Store>,
    packet_ack_adapter: Arc<Store>,
    packet_receipt_adapter: Arc<Store>,
}

impl<Store: IbcStore> IbcChannelService<Store> {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            channel_end_adapter: Arc::clone(&store),
            packet_commitment_adapter: Arc::clone(&store),
            packet_ack_adapter: Arc::clone(&store),
            packet_receipt_adapter: Arc::clone(&store),
        }
    }
}

#[tonic::async_trait]
impl<Store: IbcStore + 'static> ChannelQuery for IbcChannelService<Store> {
    /// Queries an IBC Channel.
    async fn channel(
        &self,
        request: Request<QueryChannelRequest>,
    ) -> Result<Response<QueryChannelResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let channel_opt = self
            .channel_end_adapter
            .get_channel_end(
                StoreHeight::Latest,
                &path::ChannelEndsPath(port_id, channel_id),
            )
            .map_err(Status::data_loss)?
            .map(|channel_end: ChannelEnd| channel_end.into());

        Ok(Response::new(QueryChannelResponse {
            channel: channel_opt,
            proof: vec![],
            proof_height: None,
        }))
    }

    /// Queries all the IBC channels of a chain.
    async fn channels(
        &self,
        _request: Request<QueryChannelsRequest>,
    ) -> Result<Response<QueryChannelsResponse>, Status> {
        let channel_path_prefix: Path = String::from("channelEnds/ports")
            .try_into()
            .expect("'channelEnds/ports' expected to be a valid Path");

        let channel_paths = self
            .channel_end_adapter
            .get_paths_by_prefix(&channel_path_prefix)
            .map_err(Status::internal)?;
        let mut identified_channels = Vec::with_capacity(channel_paths.len());

        for path in channel_paths.into_iter() {
            match path.try_into() {
                Ok(IbcPath::ChannelEnds(channels_path)) => {
                    let channel_end = self
                        .channel_end_adapter
                        .get_channel_end(StoreHeight::Latest, &channels_path)
                        .map_err(Status::data_loss)?
                        .expect("channel path returned by get_keys() had no associated channel");
                    identified_channels.push(
                        IdentifiedChannelEnd::new(channels_path.0, channels_path.1, channel_end)
                            .into(),
                    );
                }
                _ => panic!("unexpected path"),
            }
        }

        Ok(Response::new(QueryChannelsResponse {
            channels: identified_channels,
            pagination: None,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.channel_end_adapter.current_height(),
            }),
        }))
    }

    /// Queries all the channels associated with a connection end.
    async fn connection_channels(
        &self,
        request: Request<QueryConnectionChannelsRequest>,
    ) -> Result<Response<QueryConnectionChannelsResponse>, Status> {
        let conn_id = ConnectionId::from_str(&request.get_ref().connection)
            .map_err(|_| Status::invalid_argument("invalid connection id"))?;

        let path = "channelEnds"
            .to_owned()
            .try_into()
            .expect("'commitments/ports' expected to be a valid Path");

        let keys = self
            .channel_end_adapter
            .get_paths_by_prefix(&path)
            .map_err(Status::internal)?;
        let mut identified_channels = Vec::with_capacity(keys.len());

        for path in keys.into_iter() {
            if let Ok(IbcPath::ChannelEnds(path)) = path.try_into() {
                if let Some(channel_end) = self
                    .channel_end_adapter
                    .get_channel_end(StoreHeight::Latest, &path)
                    .map_err(Status::data_loss)?
                {
                    if channel_end.connection_hops.first() == Some(&conn_id) {
                        identified_channels
                            .push(IdentifiedChannelEnd::new(path.0, path.1, channel_end).into());
                    }
                }
            }
        }

        Ok(Response::new(QueryConnectionChannelsResponse {
            channels: identified_channels,
            pagination: None,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.channel_end_adapter.current_height(),
            }),
        }))
    }

    /// Queries for the client state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        _request: Request<QueryChannelClientStateRequest>,
    ) -> Result<Response<QueryChannelClientStateResponse>, Status> {
        todo!()
    }

    /// Queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        _request: Request<QueryChannelConsensusStateRequest>,
    ) -> Result<Response<QueryChannelConsensusStateResponse>, Status> {
        todo!()
    }

    async fn packet_commitment(
        &self,
        _request: Request<QueryPacketCommitmentRequest>,
    ) -> Result<Response<QueryPacketCommitmentResponse>, Status> {
        todo!()
    }

    /// Returns all the packet commitments hashes associated with a channel.
    async fn packet_commitments(
        &self,
        request: Request<QueryPacketCommitmentsRequest>,
    ) -> Result<Response<QueryPacketCommitmentsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let commitment_paths = {
            let prefix: Path = String::from("commitments/ports")
                .try_into()
                .expect("'commitments/ports' expected to be a valid Path");
            self.packet_commitment_adapter
                .get_paths_by_prefix(&prefix)
                .map_err(Status::internal)?
        };

        let matching_commitment_paths = |path: Path| -> Option<path::CommitmentsPath> {
            match path.try_into() {
                Ok(IbcPath::Commitments(p))
                    if p.port_id == port_id && p.channel_id == channel_id =>
                {
                    Some(p)
                }
                _ => None,
            }
        };

        let mut packet_states = Vec::with_capacity(commitment_paths.len());

        for path in commitment_paths
            .into_iter()
            .filter_map(matching_commitment_paths)
        {
            let commitment = self
                .packet_commitment_adapter
                .get_packet_commitment(StoreHeight::Latest, &path)
                .map_err(Status::data_loss)?
                .unwrap();
            let data = commitment.into_vec();
            if !data.is_empty() {
                packet_states.push(PacketState {
                    port_id: path.port_id.to_string(),
                    channel_id: path.channel_id.to_string(),
                    sequence: path.sequence.into(),
                    data,
                });
            }
        }

        Ok(Response::new(QueryPacketCommitmentsResponse {
            commitments: packet_states,
            pagination: None,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_commitment_adapter.current_height(),
            }),
        }))
    }

    /// Queries if a given packet sequence has been received on the queried chain
    async fn packet_receipt(
        &self,
        _request: Request<QueryPacketReceiptRequest>,
    ) -> Result<Response<QueryPacketReceiptResponse>, Status> {
        todo!()
    }

    /// Queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        _request: Request<QueryPacketAcknowledgementRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementResponse>, Status> {
        todo!()
    }

    /// Returns all the packet acknowledgements associated with a channel.
    async fn packet_acknowledgements(
        &self,
        request: Request<QueryPacketAcknowledgementsRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;

        let ack_paths = {
            let prefix: Path = String::from("acks/ports")
                .try_into()
                .expect("'acks/ports' expected to be a valid Path");
            self.packet_ack_adapter
                .get_paths_by_prefix(&prefix)
                .map_err(Status::internal)?
        };

        let matching_ack_paths = |path: Path| -> Option<path::AcksPath> {
            match path.try_into() {
                Ok(IbcPath::Acks(p)) if p.port_id == port_id && p.channel_id == channel_id => {
                    Some(p)
                }
                _ => None,
            }
        };

        let mut packet_states = Vec::with_capacity(ack_paths.len());

        for path in ack_paths.into_iter().filter_map(matching_ack_paths) {
            if let Some(commitment) = self
                .packet_ack_adapter
                .get_acknowledgement_commitment(StoreHeight::Latest, &path)
                .map_err(Status::data_loss)?
            {
                let data = commitment.into_vec();
                if !data.is_empty() {
                    packet_states.push(PacketState {
                        port_id: path.port_id.to_string(),
                        channel_id: path.channel_id.to_string(),
                        sequence: path.sequence.into(),
                        data,
                    });
                }
            }
        }

        Ok(Response::new(QueryPacketAcknowledgementsResponse {
            acknowledgements: packet_states,
            pagination: None,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_ack_adapter.current_height(),
            }),
        }))
    }

    /// Returns all the unreceived IBC packets associated with
    /// a channel and sequences.
    ///
    /// QUESTION. Currently only works for unordered channels; ordered channels
    /// don't use receipts. However, ibc-go does it this way. Investigate if
    /// this query only ever makes sense on unordered channels.
    async fn unreceived_packets(
        &self,
        request: Request<QueryUnreceivedPacketsRequest>,
    ) -> Result<Response<QueryUnreceivedPacketsResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;
        let sequences_to_check: Vec<u64> = request.packet_commitment_sequences;

        let unreceived_sequences: Vec<u64> = sequences_to_check
            .into_iter()
            .filter(|seq| {
                let receipts_path = path::ReceiptsPath {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                    sequence: Sequence::from(*seq),
                };
                let packet_receipt: Option<()> = self
                    .packet_receipt_adapter
                    .get_opt(StoreHeight::Latest, &receipts_path)
                    .ok()
                    .flatten();
                packet_receipt.is_none()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedPacketsResponse {
            sequences: unreceived_sequences,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_receipt_adapter.current_height(),
            }),
        }))
    }

    /// Returns all the unreceived IBC acknowledgements
    /// associated with a channel and sequences.
    async fn unreceived_acks(
        &self,
        request: Request<QueryUnreceivedAcksRequest>,
    ) -> Result<Response<QueryUnreceivedAcksResponse>, Status> {
        let request = request.into_inner();
        let port_id = PortId::from_str(&request.port_id)
            .map_err(|_| Status::invalid_argument("invalid port id"))?;
        let channel_id = ChannelId::from_str(&request.channel_id)
            .map_err(|_| Status::invalid_argument("invalid channel id"))?;
        let sequences_to_check: Vec<u64> = request.packet_ack_sequences;

        let unreceived_sequences: Vec<u64> = sequences_to_check
            .into_iter()
            .filter(|seq| {
                // To check if we received an acknowledgement, we check if we still have the
                // sent packet commitment (upon receiving an ack, the sent
                // packet commitment is deleted).
                let commitments_path = path::CommitmentsPath {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                    sequence: Sequence::from(*seq),
                };

                self.packet_commitment_adapter
                    .get_packet_commitment(StoreHeight::Latest, &commitments_path)
                    .ok()
                    .flatten()
                    .is_some()
            })
            .collect();

        Ok(Response::new(QueryUnreceivedAcksResponse {
            sequences: unreceived_sequences,
            height: Some(Height {
                revision_number: CHAIN_REVISION_NUMBER,
                revision_height: self.packet_commitment_adapter.current_height(),
            }),
        }))
    }

    /// Returns the next receive sequence for a given channel.
    async fn next_sequence_receive(
        &self,
        _request: Request<QueryNextSequenceReceiveRequest>,
    ) -> Result<Response<QueryNextSequenceReceiveResponse>, Status> {
        todo!()
    }
}
