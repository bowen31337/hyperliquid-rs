//! gRPC server implementation for Hyperliquid SDK
//!
//! This module provides gRPC endpoints for the Hyperliquid SDK.

use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status, Code};

// Import generated protobuf code
pub mod pb {
    tonic::include_proto!("hyperliquid");
}

use pb::{
    hyperliquid_service_server::{HyperliquidService, HyperliquidServiceServer},
    MetaRequest, MetaResponse, UserStateRequest, UserStateResponse,
    AllMidsRequest, AllMidsResponse, L2BookRequest, L2BookResponse,
    TradesRequest, TradesResponse, CandlesRequest, CandlesResponse,
    QueryOrderRequest, QueryOrderResponse, PlaceOrderRequest, PlaceOrderResponse,
    CancelOrderRequest, CancelOrderResponse, ModifyOrderRequest, ModifyOrderResponse,
    OpenOrdersRequest, OpenOrdersResponse, StreamsSubscriptionRequest,
    StreamResponse, Error as GrpcError,
};

// Import core functionality
use hyperliquid_core::{InfoClient, HttpClient, Config};
use hyperliquid_core::types::*;

/// gRPC server implementation
#[derive(Clone)]
pub struct HyperliquidGrpcServer {
    info_client: InfoClient,
}

impl HyperliquidGrpcServer {
    /// Create a new gRPC server with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::mainnet();
        let http_client = HttpClient::new_with_config(config.http.clone());
        let info_client = InfoClient::new(http_client);

        Ok(Self { info_client })
    }

    /// Convert core error to gRPC error
    fn map_error(error: hyperliquid_core::error::HyperliquidError) -> Status {
        match error {
            hyperliquid_core::error::HyperliquidError::Network(e) => {
                Status::new(Code::Internal, format!("Network error: {}", e))
            }
            hyperliquid_core::error::HyperliquidError::Api { code, message } => {
                Status::new(Code::Internal, format!("API error {}: {}", code, message))
            }
            hyperliquid_core::error::HyperliquidError::Signing(e) => {
                Status::new(Code::InvalidArgument, format!("Signing error: {}", e))
            }
            hyperliquid_core::error::HyperliquidError::Serialization(e) => {
                Status::new(Code::Internal, format!("Serialization error: {}", e))
            }
            hyperliquid_core::error::HyperliquidError::Deserialization(e) => {
                Status::new(Code::Internal, format!("Deserialization error: {}", e))
            }
            hyperliquid_core::error::HyperliquidError::Timeout => {
                Status::new(Code::DeadlineExceeded, "Request timeout")
            }
            hyperliquid_core::error::HyperliquidError::RateLimit => {
                Status::new(Code::ResourceExhausted, "Rate limit exceeded")
            }
            hyperliquid_core::error::HyperliquidError::Config(e) => {
                Status::new(Code::InvalidArgument, format!("Configuration error: {}", e))
            }
            hyperliquid_core::error::HyperliquidError::WebSocket(e) => {
                Status::new(Code::Internal, format!("WebSocket error: {}", e))
            }
        }
    }

    /// Convert core types to protobuf types
    fn convert_meta_response(meta: Meta) -> MetaResponse {
        MetaResponse {
            assets: meta
                .universe
                .into_iter()
                .map(|asset| pb::AssetMeta {
                    name: asset.name,
                    only_isolated: asset.only_isolated,
                    sz_decimals: asset.sz_decimals,
                    szi_decimals: asset.szi_decimals,
                    px_decimals: asset.px_decimals,
                    lot_sz_px_decimals: asset.lot_sz_px_decimals,
                    type_pb: asset.r#type,
                })
                .collect(),
            exchange: Some(pb::ExchangeMeta {
                chain_id: meta.chain_id,
                exchange_address: meta.exchange,
                chain_type: meta.chain_type,
                vault_addrs: meta.vault_addrs,
            }),
        }
    }

    fn convert_user_state_response(user_state: UserState) -> UserStateResponse {
        UserStateResponse {
            user_state: Some(pb::UserState {
                address: user_state.address,
                positions: user_state
                    .positions
                    .into_iter()
                    .map(|pos| pb::AssetPosition {
                        coin: pos.coin,
                        szi: pos.szi,
                        entry_px: pos.entry_px,
                        leverage: pos.leverage,
                        liquidation_px: pos.liquidation_px,
                        break_even_px: pos.break_even_px,
                        position_value: pos.position_value,
                    })
                    .collect(),
                margin_summary: Some(pb::MarginSummary {
                    account_value: user_state.margin_summary.account_value,
                    total_margin_used: user_state.margin_summary.total_margin_used,
                    total_ntl_pos: user_state.margin_summary.total_ntl_pos,
                    total_raw_usd: user_state.margin_summary.total_raw_usd,
                }),
            }),
        }
    }
}

#[tonic::async_trait]
impl HyperliquidService for HyperliquidGrpcServer {
    async fn get_meta(
        &self,
        request: Request<MetaRequest>,
    ) -> Result<Response<MetaResponse>, Status> {
        let _request = request.into_inner();

        match self.info_client.meta().await {
            Ok(meta) => {
                let response = Self::convert_meta_response(meta);
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn get_user_state(
        &self,
        request: Request<UserStateRequest>,
    ) -> Result<Response<UserStateResponse>, Status> {
        let request = request.into_inner();

        match self.info_client.user_state(&request.address).await {
            Ok(user_state) => {
                let response = Self::convert_user_state_response(user_state);
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn get_all_mids(
        &self,
        _request: Request<AllMidsRequest>,
    ) -> Result<Response<AllMidsResponse>, Status> {
        match self.info_client.all_mids().await {
            Ok(mids) => {
                let response = AllMidsResponse {
                    mids: mids.into_iter().collect(),
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn get_l2_book(
        &self,
        request: Request<L2BookRequest>,
    ) -> Result<Response<L2BookResponse>, Status> {
        let request = request.into_inner();

        match self.info_client.l2_book(&request.coin, request.levels as u32).await {
            Ok(book) => {
                let response = L2BookResponse {
                    coin: book.coin,
                    levels: book
                        .levels
                        .into_iter()
                        .map(|level| pb::OrderLevel {
                            price: level.price,
                            size: level.size,
                        })
                        .collect(),
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn get_trades(
        &self,
        request: Request<TradesRequest>,
    ) -> Result<Response<TradesResponse>, Status> {
        let request = request.into_inner();

        match self.info_client.trades(&request.coin, request.count as u32).await {
            Ok(trades) => {
                let response = TradesResponse {
                    trades: trades
                        .into_iter()
                        .map(|trade| pb::Trade {
                            coin: trade.coin,
                            side: trade.side,
                            px: trade.px,
                            sz: trade.sz,
                            time: trade.time,
                            hash: trade.hash,
                        })
                        .collect(),
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn get_candles(
        &self,
        request: Request<CandlesRequest>,
    ) -> Result<Response<CandlesResponse>, Status> {
        let request = request.into_inner();

        match self.info_client.candles_snapshot(
            &request.coin,
            &request.interval,
            request.start_time,
            request.end_time,
        ).await {
            Ok(candles) => {
                let response = CandlesResponse {
                    candles: candles
                        .into_iter()
                        .map(|candle| pb::Candle {
                            start: candle.start,
                            tx_hash: candle.tx_hash,
                            px: candle.px,
                            interval: candle.interval,
                            coin: candle.coin,
                            vol: candle.vol,
                            open: candle.open,
                            high: candle.high,
                            low: candle.low,
                            close: candle.close,
                        })
                        .collect(),
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn query_order(
        &self,
        request: Request<QueryOrderRequest>,
    ) -> Result<Response<QueryOrderResponse>, Status> {
        let request = request.into_inner();

        match self.info_client.query_order_by_oid(&request.address, request.oid).await {
            Ok(order) => {
                let response = QueryOrderResponse {
                    found: true,
                    order: Some(pb::OrderDetails {
                        coin: order.coin,
                        is_buy: order.is_buy,
                        sz: order.sz,
                        limit_px: order.limit_px,
                        oid: order.oid,
                        status: order.status,
                    }),
                };
                Ok(Response::new(response))
            }
            Err(hyperliquid_core::error::HyperliquidError::Api { code: 404, .. }) => {
                let response = QueryOrderResponse {
                    found: false,
                    order: None,
                };
                Ok(Response::new(response))
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }

    async fn place_order(
        &self,
        _request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        // TODO: Implement order placement
        let response = PlaceOrderResponse {
            success: false,
            order_id: "".to_string(),
            message: "Not implemented yet".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn cancel_order(
        &self,
        _request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        // TODO: Implement order cancellation
        let response = CancelOrderResponse {
            success: false,
            message: "Not implemented yet".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn modify_order(
        &self,
        _request: Request<ModifyOrderRequest>,
    ) -> Result<Response<ModifyOrderResponse>, Status> {
        // TODO: Implement order modification
        let response = ModifyOrderResponse {
            success: false,
            message: "Not implemented yet".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn get_open_orders(
        &self,
        _request: Request<OpenOrdersRequest>,
    ) -> Result<Response<OpenOrdersResponse>, Status> {
        // TODO: Implement open orders retrieval
        let response = OpenOrdersResponse {
            orders: vec![],
        };
        Ok(Response::new(response))
    }

    type SubscribeToStreamsStream = tokio_stream::wrappers::ReceiverStream<Result<StreamResponse, Status>>;

    async fn subscribe_to_streams(
        &self,
        _request: Request<StreamsSubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeToStreamsStream>, Status> {
        // TODO: Implement streaming subscriptions
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Send error for now
        let error = GrpcError {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "Streaming not implemented yet".to_string(),
        };

        let stream_response = StreamResponse {
            response: Some(pb::stream_response::Response::Error(error)),
        };

        let _ = tx.send(Ok(stream_response)).await;

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

/// Start the gRPC server
pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let server = HyperliquidGrpcServer::new().await?;

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(HyperliquidServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}