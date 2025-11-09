use alloc::{string::String, vec, vec::Vec};

use crate::serialization::{Bytes, FieldEnvelopeBuilder, FieldPayload};

#[test]
fn builder_packs_child_payloads() {
  let mut builder = FieldEnvelopeBuilder::new(42, "root.manifest");
  let first = FieldPayload::new(Bytes::from_vec(vec![1, 2, 3]), "child.one".into(), 7, 11u128);
  let second = FieldPayload::new(Bytes::from_vec(vec![9, 9]), "child.two".into(), 8, 22u128);
  builder.append_child(&first).expect("first child");
  builder.append_child(&second).expect("second child");

  let payload = builder.finalize().expect("finalize");
  assert_eq!(payload.serializer_id(), 42);
  assert_eq!(payload.manifest(), "root.manifest");

  let mut cursor = payload.bytes().as_ref();
  let magic = read_bytes(&mut cursor, 4);
  assert_eq!(magic.as_slice(), b"AGGR");
  assert_eq!(cursor.len() > 0, true);
  let count = read_u16(&mut cursor);
  assert_eq!(count, 2);

  let parsed_first = read_entry(&mut cursor);
  assert_eq!(parsed_first.serializer_id, 7);
  assert_eq!(parsed_first.manifest, "child.one");
  assert_eq!(parsed_first.field_hash, 11u128);
  assert_eq!(parsed_first.bytes, vec![1, 2, 3]);

  let parsed_second = read_entry(&mut cursor);
  assert_eq!(parsed_second.serializer_id, 8);
  assert_eq!(parsed_second.manifest, "child.two");
  assert_eq!(parsed_second.field_hash, 22u128);
  assert_eq!(parsed_second.bytes, vec![9, 9]);

  assert!(cursor.is_empty());
}

fn read_entry(buffer: &mut &[u8]) -> ParsedEntry {
  let field_hash = read_u128(buffer);
  let serializer_id = read_u32(buffer);
  let manifest_len = read_u16(buffer) as usize;
  let manifest = read_bytes(buffer, manifest_len);
  let payload_len = read_u32(buffer) as usize;
  let bytes = read_bytes(buffer, payload_len);
  ParsedEntry { field_hash, serializer_id, manifest: String::from_utf8(manifest).expect("utf8"), bytes }
}

fn read_bytes(buffer: &mut &[u8], len: usize) -> Vec<u8> {
  let (head, tail) = buffer.split_at(len);
  *buffer = tail;
  head.to_vec()
}

fn read_u16(buffer: &mut &[u8]) -> u16 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u16>());
  *buffer = tail;
  u16::from_le_bytes(value.try_into().expect("u16 slice"))
}

fn read_u32(buffer: &mut &[u8]) -> u32 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u32>());
  *buffer = tail;
  u32::from_le_bytes(value.try_into().expect("u32 slice"))
}

fn read_u128(buffer: &mut &[u8]) -> u128 {
  let (value, tail) = buffer.split_at(core::mem::size_of::<u128>());
  *buffer = tail;
  u128::from_le_bytes(value.try_into().expect("u128 slice"))
}

struct ParsedEntry {
  field_hash:    u128,
  serializer_id: u32,
  manifest:      String,
  bytes:         Vec<u8>,
}
