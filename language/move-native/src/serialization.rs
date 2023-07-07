// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::conv::*;
use crate::rt_types::*;
use crate::target_defs;
use ethnum::U256;
use borsh::{BorshSerialize, BorshDeserialize};
use borsh::maybestd::io::Result as BorshResult;
use alloc::vec::Vec;

/// A type to serialize u256s.
///
/// Because ethnum::U256 isn't compatible with borsh.
/// This has the same repr(transparent) definition as ethnum::U256.
#[derive(BorshSerialize, BorshDeserialize)]
#[repr(transparent)]
struct U256Placeholder([u128; 2]);

fn borsh_to_vec<T: BorshSerialize>(v: &T) -> Vec<u8> {
    borsh::to_vec(v).expect("serialization_failure")
}

fn borsh_from_slice<T: BorshDeserialize>(buf: &mut &[u8]) -> T {
    BorshDeserialize::deserialize(buf).expect("deserialization failure")
}

pub unsafe fn serialize(type_v: &MoveType, v: &AnyValue) -> MoveByteVector {
    let v = borrow_move_value_as_rust_value(type_v, v);
    let s = match v {
        BorrowedTypedMoveValue::Bool(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U8(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U16(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U32(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U64(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U128(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::U256(v) => {
            let v = U256Placeholder(v.0);
            borsh_to_vec(&v.0)
        }
        BorrowedTypedMoveValue::Address(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::Signer(v) => {
            borsh_to_vec(v)
        }
        BorrowedTypedMoveValue::Vector(t, v) => {
            serialize_vector(&t, v)
        }
        BorrowedTypedMoveValue::Struct(t, v) => {
            serialize_struct(&t, v)
        }
        BorrowedTypedMoveValue::Reference(_, _) => {
            todo!("impossible case?");
        }
    };
    rust_vec_to_move_byte_vec(s)
}

pub unsafe fn deserialize(type_v: &MoveType, bytes: &MoveByteVector, v: *mut AnyValue) {
    let bytes = borrow_move_byte_vec_as_rust_vec(bytes);
    let bytes = &mut &bytes[..];
    deserialize_from_slice(type_v, bytes, v);
    assert!(bytes.is_empty());
}

unsafe fn deserialize_from_slice(type_v: &MoveType, bytes: &mut &[u8], v: *mut AnyValue) {
    let v = raw_borrow_move_value_as_rust_value(type_v, v);
    match v {
        RawBorrowedTypedMoveValue::Bool(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U8(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U16(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U32(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U64(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U128(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::U256(vptr) => {
            let v: U256Placeholder = borsh_from_slice(bytes);
            *vptr = U256(v.0);
        }
        RawBorrowedTypedMoveValue::Address(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::Signer(vptr) => {
            *vptr = borsh_from_slice(bytes);
        }
        RawBorrowedTypedMoveValue::Vector(t, vptr) => {
            let v = deserialize_vector(&t, bytes);
            // Have to use `write` here because an assignment
            // will cause Rust to run the dtor on the uninitialized vptr,
            // setting off the "drop bomb".
            core::ptr::write(vptr, v);
        }
        RawBorrowedTypedMoveValue::Struct(t, vptr) => {
            deserialize_struct(&t, bytes, vptr);
        }
        RawBorrowedTypedMoveValue::Reference(_, _) => {
            todo!("impossible case?");
        }
    }
}

unsafe fn serialize_vector(type_elt: &MoveType, v: &MoveUntypedVector) -> Vec<u8> {
    let v = borrow_typed_move_vec_as_rust_vec(type_elt, v);
    match v {
        TypedMoveBorrowedRustVec::Bool(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U8(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U16(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U32(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U64(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U128(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::U256(v) => {
            let v: &Vec<U256> = &*v;
            // safety: U256Placeholder and U256 have the same well-defined representation.
            let v: &Vec<U256Placeholder> = core::mem::transmute(v);
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::Address(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::Signer(v) => {
            borsh_to_vec(&*v)
        }
        TypedMoveBorrowedRustVec::Vector(t, v) => {
            // fixme lots of allocations here
            let len: u32 = v.len().try_into().expect("overlong vector");
            let mut buf = borsh_to_vec(&len);
            for elt in v.iter() {
                let mut elt_buf = serialize_vector(&t, elt);
                buf.append(&mut elt_buf);
            }
            buf
        }
        TypedMoveBorrowedRustVec::Struct(v) => {
            todo!()
        }
        TypedMoveBorrowedRustVec::Reference(_, _) => {
            todo!("impossible case?");
        }
    }
}

unsafe fn deserialize_vector(type_elt: &MoveType, bytes: &mut &[u8]) -> MoveUntypedVector {
    match type_elt.type_desc {
        TypeDesc::Bool => {
            let v: Vec<bool> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U8 => {
            let v: Vec<u8> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U16 => {
            let v: Vec<u16> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U32 => {
            let v: Vec<u32> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U64 => {
            let v: Vec<u64> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U128 => {
            let v: Vec<u128> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::U256 => {
            unsafe {
                let v: Vec<U256Placeholder> = borsh_from_slice(bytes);
                // safety: U256Placeholder and U256 have the same well-defined representation.
                let v: Vec<U256> = core::mem::transmute(v);
                rust_vec_to_move_vec(v)
            }
        }
        TypeDesc::Address => {
            let v: Vec<MoveAddress> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::Signer => {
            let v: Vec<MoveSigner> = borsh_from_slice(bytes);
            rust_vec_to_move_vec(v)
        }
        TypeDesc::Vector => {
            let vecinfo = &(*type_elt.type_info).vector;
            let inner_elt_type = vecinfo.element_type;
            let len: u32 = borsh_from_slice(bytes);
            let mut v: Vec<MoveUntypedVector> = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let eltv = deserialize_vector(&inner_elt_type, bytes);
                v.push(eltv);
            }
            rust_vec_to_move_vec(v)
        }
        TypeDesc::Struct => {
            todo!()
        }
        TypeDesc::Reference => {
            todo!("impossible case?");
        }
    }
}

// fixme this allocates more than it should
unsafe fn serialize_struct(t: &MoveType, v: &AnyValue) -> Vec<u8> {
    let mut buf = Vec::new();
    let structinfo = &(*(t.type_info)).struct_;
    for (ft, fv, _) in walk_struct_fields(structinfo, v) {
        let field_buf = serialize(ft, fv);
        let mut field_buf = move_byte_vec_to_rust_vec(field_buf);
        buf.append(&mut field_buf);
    }
    buf
}

unsafe fn deserialize_struct(t: &MoveType, bytes: &mut &[u8], v: *mut AnyValue) {
    let structinfo = &(*(t.type_info)).struct_;
    for (ft, fv, _) in walk_struct_fields_mut(structinfo, v) {
        deserialize_from_slice(ft, bytes, fv);
    }
}
