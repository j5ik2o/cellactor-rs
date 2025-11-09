#![cfg_attr(all(not(test), target_os = "none"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec};

use bincode::config::{Config, standard};
use cellactor_actor_core_rs::{
  NoStdToolbox,
  actor_prim::{Actor, ActorContextGeneric},
  error::ActorError,
  event_stream::{EventStreamEvent, EventStreamSubscriber, SerializationEventKind},
  messaging::AnyMessageView,
  props::Props,
  serialization::{
    AggregateSchemaBuilder, Bytes, FieldPath, FieldPathDisplay, FieldPathSegment, SERIALIZATION_EXTENSION,
    Serialization, SerializationError, SerializedPayload, SerializerHandle, SerializerImpl, TraversalPolicy,
  },
  system::ActorSystem,
};
use cellactor_utils_core_rs::sync::{ArcShared, NoStdMutex};
use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Customer {
  name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OrderLine {
  sku:      String,
  quantity: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PurchaseOrder {
  customer: Customer,
  line:     OrderLine,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExternalLeaf {
  code: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExternalOrder {
  payload: ExternalLeaf,
}

#[cfg(not(target_os = "none"))]
fn main() {
  let props = Props::from_fn(|| SampleGuardian);
  let system = ActorSystem::new(&props).expect("actor system");
  let serialization = system.extension(&SERIALIZATION_EXTENSION).expect("serialization extension");
  let registry = serialization.registry();

  let serializer = register_demo_serializer(&serialization);
  bind_type::<Customer>(&registry, &serializer, "demo.customer", decode_customer);
  bind_type::<OrderLine>(&registry, &serializer, "demo.line", decode_order_line);
  bind_type::<PurchaseOrder>(&registry, &serializer, "demo.order", decode_order);
  bind_type::<ExternalOrder>(&registry, &serializer, "demo.external", decode_external_order);
  register_order_schema(&registry);
  register_external_schema(&registry);

  serialization.telemetry().config().set_debug_trace_enabled(true);
  serialization.telemetry().config().set_latency_threshold_us(0);

  let subscriber = ArcShared::new(PrintSubscriber::new());
  let dyn_sub: ArcShared<dyn EventStreamSubscriber<NoStdToolbox>> = subscriber.clone();
  let _subscription = system.subscribe_event_stream(&dyn_sub);

  let order = PurchaseOrder {
    customer: Customer { name: String::from("Alice") },
    line:     OrderLine { sku: String::from("SKU-123"), quantity: 2 },
  };

  let payload = serialization.serialize(&order).expect("serialize order");
  println!("manifest={} bytes={}", payload.manifest(), payload.bytes().len());

  let decoded = decode_purchase_order(serialization.clone(), &payload).expect("decode order");
  println!("decoded name={} sku={}", decoded.customer.name, decoded.line.sku);

  trigger_external_serializer(&serialization);

  println!(
    "telemetry success={} failure={}",
    serialization.telemetry().counters().success_total(),
    serialization.telemetry().counters().failure_total()
  );

  system.terminate().expect("terminate");
  system.run_until_terminated();
}

#[cfg(target_os = "none")]
fn main() {}

struct SampleGuardian;

impl Actor for SampleGuardian {
  fn receive(
    &mut self,
    _ctx: &mut ActorContextGeneric<'_, NoStdToolbox>,
    _message: AnyMessageView<'_, NoStdToolbox>,
  ) -> Result<(), ActorError> {
    Ok(())
  }
}

fn register_demo_serializer(serialization: &ArcShared<Serialization<NoStdToolbox>>) -> SerializerHandle {
  let handle = SerializerHandle::new(DemoSerializer);
  serialization.registry().register_serializer(handle.clone()).expect("register serializer");
  handle
}

fn bind_type<T>(
  registry: &ArcShared<cellactor_actor_core_rs::serialization::SerializerRegistry<NoStdToolbox>>,
  handle: &SerializerHandle,
  manifest: &str,
  decoder: fn(&[u8]) -> Result<T, SerializationError>,
) where
  T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static, {
  registry.bind_type::<T, _>(handle, Some(manifest.to_string()), decoder).expect("bind type");
}

fn register_order_schema(
  registry: &ArcShared<cellactor_actor_core_rs::serialization::SerializerRegistry<NoStdToolbox>>,
) {
  let mut builder = AggregateSchemaBuilder::<PurchaseOrder>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("order").expect("display"),
  );
  builder
    .add_value_field::<Customer, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("order.customer").expect("display"),
      false,
      |order| &order.customer,
    )
    .expect("customer field");
  builder
    .add_value_field::<OrderLine, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(1)]).expect("path"),
      FieldPathDisplay::from_str("order.line").expect("display"),
      false,
      |order| &order.line,
    )
    .expect("line field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

fn register_external_schema(
  registry: &ArcShared<cellactor_actor_core_rs::serialization::SerializerRegistry<NoStdToolbox>>,
) {
  let mut builder = AggregateSchemaBuilder::<ExternalOrder>::new(
    TraversalPolicy::DepthFirst,
    FieldPathDisplay::from_str("external").expect("display"),
  );
  builder
    .add_value_field::<ExternalLeaf, _>(
      FieldPath::from_segments(&[FieldPathSegment::new(0)]).expect("path"),
      FieldPathDisplay::from_str("external.payload").expect("display"),
      true,
      |order| &order.payload,
    )
    .expect("external field");
  registry.register_aggregate_schema(builder.finish().expect("schema")).expect("register schema");
}

struct PrintSubscriber {
  events: NoStdMutex<Vec<EventStreamEvent<NoStdToolbox>>>,
}

impl PrintSubscriber {
  fn new() -> Self {
    Self { events: NoStdMutex::new(Vec::new()) }
  }
}

impl EventStreamSubscriber<NoStdToolbox> for PrintSubscriber {
  fn on_event(&self, event: &EventStreamEvent<NoStdToolbox>) {
    if let EventStreamEvent::Serialization(runtime) = event {
      match runtime.kind() {
        | SerializationEventKind::Success => println!("success field_hash={}", runtime.field_path_hash()),
        | SerializationEventKind::DebugTrace(info) => {
          println!("trace manifest={} bytes={}", info.manifest(), info.size_bytes())
        },
        | SerializationEventKind::Latency(micros) => println!("latency={}µs", micros),
        | SerializationEventKind::Fallback(reason) => println!("fallback reason={:?}", reason),
        | other => println!("other telemetry={:?}", other),
      }
    }
    self.events.lock().push(event.clone());
  }
}

#[derive(Clone)]
struct DemoSerializer;

impl SerializerImpl for DemoSerializer {
  fn identifier(&self) -> u32 {
    101
  }

  fn serialize_erased(
    &self,
    value: &dyn ErasedSerialize,
  ) -> Result<cellactor_actor_core_rs::serialization::Bytes, SerializationError> {
    bincode::serde::encode_to_vec(value, bincode_config())
      .map(cellactor_actor_core_rs::serialization::Bytes::from_vec)
      .map_err(|err| SerializationError::SerializationFailed(err.to_string()))
  }

  fn deserialize(
    &self,
    _bytes: &[u8],
    manifest: &str,
  ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send>, SerializationError> {
    Err(SerializationError::UnknownManifest { serializer_id: self.identifier(), manifest: manifest.to_string() })
  }
}

fn decode_customer(bytes: &[u8]) -> Result<Customer, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|err| SerializationError::DeserializationFailed(err.to_string()))
}

fn decode_order_line(bytes: &[u8]) -> Result<OrderLine, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|err| SerializationError::DeserializationFailed(err.to_string()))
}

fn decode_order(bytes: &[u8]) -> Result<PurchaseOrder, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|err| SerializationError::DeserializationFailed(err.to_string()))
}

fn decode_external_order(bytes: &[u8]) -> Result<ExternalOrder, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode_config())
    .map(|(value, _)| value)
    .map_err(|err| SerializationError::DeserializationFailed(err.to_string()))
}

fn trigger_external_serializer(serialization: &ArcShared<Serialization<NoStdToolbox>>) {
  let order = ExternalOrder { payload: ExternalLeaf { code: 404 } };
  let payload = serialization.serialize(&order).expect("serialize external order");
  println!("external manifest={} bytes={}", payload.manifest(), payload.bytes().len());
}

fn bincode_config() -> impl Config {
  standard().with_fixed_int_encoding()
}

fn decode_purchase_order(
  serialization: ArcShared<Serialization<NoStdToolbox>>,
  payload: &SerializedPayload,
) -> Result<PurchaseOrder, SerializationError> {
  let mut cursor = payload.bytes().as_ref();
  if cursor.len() < 4 || &cursor[..4] != b"AGGR" {
    return Err(SerializationError::DeserializationFailed("invalid aggregate header".into()));
  }
  cursor = &cursor[4..];
  let (count, mut cursor) = read_u16(cursor)?;
  let mut customer = None;
  let mut line = None;

  for _ in 0..count {
    let (field_hash, rest) = read_u128(cursor)?;
    cursor = rest;
    let (serializer_id, rest) = read_u32(cursor)?;
    cursor = rest;
    let (manifest_len, rest) = read_u16(cursor)?;
    cursor = rest;
    if cursor.len() < manifest_len as usize {
      return Err(SerializationError::DeserializationFailed("manifest length overflow".into()));
    }
    let manifest = core::str::from_utf8(&cursor[..manifest_len as usize])
      .map_err(|_| SerializationError::DeserializationFailed("manifest utf8".into()))?
      .to_string();
    cursor = &cursor[manifest_len as usize..];
    let (payload_len, rest) = read_u32(cursor)?;
    cursor = rest;
    if cursor.len() < payload_len as usize {
      return Err(SerializationError::DeserializationFailed("payload length overflow".into()));
    }
    let child_bytes = cursor[..payload_len as usize].to_vec();
    cursor = &cursor[payload_len as usize..];

    let child_payload = SerializedPayload::new(serializer_id, manifest.clone(), Bytes::from_vec(child_bytes));
    let value = serialization.deserialize_payload(&child_payload)?;
    if manifest == "demo.customer" {
      customer = Some(
        value
          .downcast::<Customer>()
          .map_err(|_| SerializationError::DeserializationFailed("customer".into()))?
          .as_ref()
          .clone(),
      );
    } else if manifest == "demo.line" {
      line = Some(
        value
          .downcast::<OrderLine>()
          .map_err(|_| SerializationError::DeserializationFailed("line".into()))?
          .as_ref()
          .clone(),
      );
    } else {
      println!("unknown field hash {} manifest {}", field_hash, manifest);
    }
  }

  match (customer, line) {
    | (Some(customer), Some(line)) => Ok(PurchaseOrder { customer, line }),
    | _ => Err(SerializationError::DeserializationFailed("missing aggregate fields".into())),
  }
}

fn read_u16(cursor: &[u8]) -> Result<(u16, &[u8]), SerializationError> {
  if cursor.len() < 2 {
    return Err(SerializationError::DeserializationFailed("cursor underflow".into()));
  }
  let (value, rest) = cursor.split_at(2);
  Ok((u16::from_le_bytes(value.try_into().unwrap()), rest))
}

fn read_u32(cursor: &[u8]) -> Result<(u32, &[u8]), SerializationError> {
  if cursor.len() < 4 {
    return Err(SerializationError::DeserializationFailed("cursor underflow".into()));
  }
  let (value, rest) = cursor.split_at(4);
  Ok((u32::from_le_bytes(value.try_into().unwrap()), rest))
}

fn read_u128(cursor: &[u8]) -> Result<(u128, &[u8]), SerializationError> {
  if cursor.len() < 16 {
    return Err(SerializationError::DeserializationFailed("cursor underflow".into()));
  }
  let (value, rest) = cursor.split_at(16);
  Ok((u128::from_le_bytes(value.try_into().unwrap()), rest))
}
