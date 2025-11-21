//! std provisioning module wiring.

pub mod block_reflector;
pub mod failover_controller;
pub mod partition_manager_bridge;
pub mod placement_supervisor_bridge;
pub mod provider_event;
pub mod provider_registry;
pub mod provider_store;
pub mod provider_stream;
pub mod provider_stream_driver;
pub mod provider_stream_runner;
pub mod provider_validator;
pub mod provider_watch_hub;
pub mod provisioning_error;
pub mod provisioning_metrics;
pub mod remoting_bridge;
pub mod remoting_health;
pub mod remoting_port;
