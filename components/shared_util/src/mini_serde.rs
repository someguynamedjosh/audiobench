use crate::prelude::FloatUtil;
use bitvec::prelude::*;

#[scones::make_constructor]
pub struct MiniSer {
    #[value(BitVec::new())]
    bits: BitVec<Lsb0, u8>,
}

impl MiniSer {
    pub fn finish(self) -> Vec<u8> {
        self.bits.into_vec()
    }

    pub fn bool(&mut self, value: bool) {
        self.bits.push(value);
    }

    fn uint(&mut self, value: usize, num_bits: u8) {
        assert!(num_bits <= 63);
        assert!(value < 0b1 << num_bits);
        let mut selector = 0b1;
        for _ in 0..num_bits {
            self.bits.push(value & selector > 0);
            selector <<= 1;
        }
    }

    pub fn u1(&mut self, value: u8) {
        self.uint(value as usize, 1);
    }
    pub fn u2(&mut self, value: u8) {
        self.uint(value as usize, 2);
    }
    pub fn u3(&mut self, value: u8) {
        self.uint(value as usize, 3);
    }
    pub fn u4(&mut self, value: u8) {
        self.uint(value as usize, 4);
    }
    pub fn u5(&mut self, value: u8) {
        self.uint(value as usize, 4);
    }
    pub fn u6(&mut self, value: u8) {
        self.uint(value as usize, 6);
    }
    pub fn u7(&mut self, value: u8) {
        self.uint(value as usize, 7);
    }
    pub fn u8(&mut self, value: u8) {
        self.uint(value as usize, 8);
    }
    pub fn u16(&mut self, value: u16) {
        self.uint(value as usize, 16);
    }
    pub fn i16(&mut self, value: i16) {
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
    }
    pub fn u32(&mut self, value: u32) {
        self.uint(value as usize, 32);
    }
    pub fn i32(&mut self, value: i32) {
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
    }
    pub fn f32(&mut self, value: f32) {
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
    }
    pub fn f32_in_range(&mut self, value: f32, min: f32, max: f32) {
        let value = value.from_range(min, max);
        // Value is now between 0 and 1.
        let value = value.to_range(0.0, 0x10000 as f32).min(0xFFFF as f32);
        // Value is now between 0 and 0xFFFF
        self.u16(value as u16);
    }
    pub fn str(&mut self, str: &str) {
        let bytes = str.as_bytes();
        assert!(bytes.len() < std::u16::MAX as usize);
        self.u16(bytes.len() as u16);
        for byte in bytes {
            self.u8(*byte);
        }
    }
}

pub struct MiniDes {
    bits: BitVec<Lsb0, u8>,
    read_ptr: usize,
}

impl MiniDes {
    pub fn start(data: Vec<u8>) -> Self {
        Self {
            bits: BitVec::from_vec(data),
            read_ptr: 0,
        }
    }

    pub fn bool(&mut self) -> Result<bool, ()> {
        self.read_ptr += 1;
        if self.read_ptr >= self.bits.len() {
            return Err(());
        }
        Ok(self.bits[self.read_ptr])
    }

    fn uint(&mut self, num_bits: u8) -> Result<usize, ()> {
        assert!(num_bits <= 63);
        let mut bit = 0b1;
        let mut value = 0;
        for _ in 0..num_bits {
            if self.bool()? {
                value |= bit;
            }
            bit <<= 1;
        }
        Ok(value)
    }

    pub fn u1(&mut self) -> Result<u8, ()> {
        Ok(self.uint(1)? as u8)
    }
    pub fn u2(&mut self) -> Result<u8, ()> {
        Ok(self.uint(2)? as u8)
    }
    pub fn u3(&mut self) -> Result<u8, ()> {
        Ok(self.uint(3)? as u8)
    }
    pub fn u4(&mut self) -> Result<u8, ()> {
        Ok(self.uint(4)? as u8)
    }
    pub fn u5(&mut self) -> Result<u8, ()> {
        Ok(self.uint(5)? as u8)
    }
    pub fn u6(&mut self) -> Result<u8, ()> {
        Ok(self.uint(6)? as u8)
    }
    pub fn u7(&mut self) -> Result<u8, ()> {
        Ok(self.uint(7)? as u8)
    }
    pub fn u8(&mut self) -> Result<u8, ()> {
        Ok(self.uint(8)? as u8)
    }
    pub fn u16(&mut self) -> Result<u16, ()> {
        Ok(self.uint(8)? as u16)
    }
    pub fn i16(&mut self) -> Result<i16, ()> {
        let bytes = [self.u8()?, self.u8()?];
        Ok(i16::from_le_bytes(bytes))
    }
    pub fn u32(&mut self) -> Result<u32, ()> {
        Ok(self.uint(8)? as u32)
    }
    pub fn i32(&mut self) -> Result<i32, ()> {
        let bytes = [self.u8()?, self.u8()?, self.u8()?, self.u8()?];
        Ok(i32::from_le_bytes(bytes))
    }
    pub fn f32(&mut self) -> Result<f32, ()> {
        let bytes = [self.u8()?, self.u8()?, self.u8()?, self.u8()?];
        Ok(f32::from_le_bytes(bytes))
    }
    pub fn f32_in_range(&mut self, min: f32, max: f32) -> Result<f32, ()> {
        let value = self.u16()? as f32;
        // Value is now between 0 and 0xFFFF
        let value = value.from_range(0.0, 0xFFFF as f32);
        // Value is now between 0 and 1.
        Ok(value.to_range(min, max))
    }
    pub fn str(&mut self) -> Result<String, ()> {
        let length = self.u16()?;
        let mut bytes = Vec::new();
        for _ in 0..length {
            bytes.push(self.u8()?);
        }
        String::from_utf8(bytes).map_err(|_| ())
    }
}
