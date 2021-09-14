use bytes::{Buf, BufMut};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{convert::TryInto, fmt};

pub trait Codec: Sized {
    fn decode<B: Buf>(buf: &mut B) -> Self;
    fn encode<B: BufMut>(&self, buf: &mut B);
}

pub trait BufExt {
    fn get<T: Codec>(&mut self) -> T;
    fn get_variable_length(&mut self) -> u64;
}

impl Codec for u8 {
    fn decode<B: Buf>(buf: &mut B) -> u8 {
        buf.get_u8()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self);
    }
}

impl Codec for u16 {
    fn decode<B: Buf>(buf: &mut B) -> u16 {
        buf.get_u16()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u16(*self);
    }
}

impl Codec for u32 {
    fn decode<B: Buf>(buf: &mut B) -> u32 {
        buf.get_u32()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32(*self);
    }
}

impl Codec for u64 {
    fn decode<B: Buf>(buf: &mut B) -> u64 {
        buf.get_u64()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u64(*self);
    }
}

impl Codec for Ipv4Addr {
    fn decode<B: Buf>(buf: &mut B) -> Ipv4Addr {
        let mut octets = [0; 4];
        buf.copy_to_slice(&mut octets);
        octets.into()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(&self.octets());
    }
}

impl Codec for Ipv6Addr {
    fn decode<B: Buf>(buf: &mut B) -> Ipv6Addr {
        let mut octets = [0; 16];
        buf.copy_to_slice(&mut octets);
        octets.into()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(&self.octets());
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VarInt(u64);

impl VarInt {
    pub const MAX: VarInt = VarInt((1 << 62) - 1);
    pub const MAX_SIZE: usize = 0;

    pub fn from_u32(x: u32) -> Self {
        VarInt(x as u64)
    }

    pub fn from_u64(x: u64) -> Self {
        VarInt(x)
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn size(self) -> usize {
        let x = self.0;
        if x < 2u64.pow(6) {
            1
        } else if x < 2u64.pow(14) {
            2
        } else if x < 2u64.pow(30) {
            4
        } else if x < 2u64.pow(62) {
            8
        } else {
            unreachable!("VarInt malformed, size not computed");
        }
    }

    pub fn size_encoded(first: u8) -> usize {
        2usize.pow((first >> 6) as u32)
    }
}

impl From<VarInt> for u64 {
    fn from(x: VarInt) -> u64 {
        x.0
    }
}

impl From<u8> for VarInt {
    fn from(x: u8) -> Self {
        VarInt(x.into())
    }
}

impl From<u16> for VarInt {
    fn from(x: u16) -> Self {
        VarInt(x.into())
    }
}

impl From<u32> for VarInt {
    fn from(x: u32) -> Self {
        VarInt(x.into())
    }
}

impl fmt::Debug for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Codec for VarInt {
    fn decode<B: Buf>(r: &mut B) -> Self {
        let mut buf = [0; 8];
        buf[0] = r.get_u8();
        let tag = buf[0] >> 6;
        buf[0] &= 0b0011_1111;
        let x = match tag {
            0b00 => u64::from(buf[0]),
            0b01 => {
                r.copy_to_slice(&mut buf[1..2]);
                u64::from(u16::from_be_bytes(buf[..2].try_into().unwrap()))
            }
            0b10 => {
                r.copy_to_slice(&mut buf[1..4]);
                u64::from(u32::from_be_bytes(buf[..4].try_into().unwrap()))
            }
            0b11 => {
                r.copy_to_slice(&mut buf[1..8]);
                u64::from_be_bytes(buf)
            }
            _ => unreachable!(),
        };
        VarInt(x)
    }

    fn encode<B: BufMut>(&self, w: &mut B) {
        let x = self.0;
        if x < 2u64.pow(6) {
            w.put_u8(x as u8);
        } else if x < 2u64.pow(14) {
            w.put_u16(0b01 << 14 | x as u16);
        } else if x < 2u64.pow(30) {
            w.put_u32(0b10 << 30 | x as u32);
        } else if x < 2u64.pow(62) {
            w.put_u64(0b11 << 62 | x);
        } else {
            unreachable!("VarInt malformed")
        }
    }
}
