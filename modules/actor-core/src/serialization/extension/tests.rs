use alloc::string::{String, ToString};

use cellactor_utils_core_rs::sync::ArcShared;

use super::SERIALIZATION_EXTENSION;
use crate::{
  NoStdToolbox,
  serialization::{PekkoSerializable, error::SerializationError, registry::SerializerRegistry},
  system::ActorSystemGeneric,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct TestMessage(String);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct AutoPekkoMessage(u32);

impl PekkoSerializable for AutoPekkoMessage {}

fn decode(bytes: &[u8]) -> Result<TestMessage, SerializationError> {
  bincode::serde::decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    .map(|(value, _)| value)
    .map_err(|error| SerializationError::DeserializationFailed(error.to_string()))
}

#[test]
fn serialization_extension_roundtrip() {
  let system = ActorSystemGeneric::<NoStdToolbox>::new_empty();
  let serialization = system.register_extension(&SERIALIZATION_EXTENSION);
  let registry: ArcShared<SerializerRegistry<NoStdToolbox>> = serialization.registry();
  let serializer = registry.find_serializer_by_id(1).expect("built-in serializer");
  registry.bind_type::<TestMessage, _>(&serializer, Some("TestMessage".into()), decode).expect("bind");

  let message = TestMessage("hello".into());
  let payload = serialization.serialize(&message).expect("serialize");
  let roundtrip: TestMessage =
    serialization.deserialize(payload.bytes().as_ref(), payload.manifest()).expect("deserialize");
  assert_eq!(roundtrip, message);

  let boxed = serialization.deserialize_payload(&payload).expect("payload");
  assert!(boxed.downcast::<TestMessage>().is_ok());
}

#[test]
fn serialization_extension_registers_pekko_serializable_types() {
  let system = ActorSystemGeneric::<NoStdToolbox>::new_empty();
  let serialization = system.register_extension(&SERIALIZATION_EXTENSION);

  serialization.register_pekko_serializable::<AutoPekkoMessage>().expect("assign default");

  let sample = AutoPekkoMessage(7);
  let payload = serialization.serialize(&sample).expect("serialize");
  let recovered: AutoPekkoMessage =
    serialization.deserialize(payload.bytes().as_ref(), payload.manifest()).expect("deserialize");
  assert_eq!(recovered, sample);
}
