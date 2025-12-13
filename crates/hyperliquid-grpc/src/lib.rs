//! Hyperliquid gRPC server crate
//!
//! This crate provides gRPC endpoints for the Hyperliquid SDK.
//! It's optional and can be used for internal services.

pub mod server;
pub mod pb;

pub use server::{HyperliquidGrpcServer, serve};
pub use pb::hyperliquid_service_server::HyperliquidServiceServer;