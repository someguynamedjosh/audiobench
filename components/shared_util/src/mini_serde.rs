use crate::{prelude::FloatUtil, Version};
use bitvec::prelude::*;

#[scones::make_constructor]
pub struct MiniSer {
    #[value(BitVec::new())]
    bits: BitVec<Lsb0, u8>,
    #[value(String::new())]
    pub debug_content: String,
    #[value(false)]
    pause_debug_content: bool,
}

impl MiniSer {
    pub fn finish(self) -> Vec<u8> {
        self.bits.into_vec()
    }

    pub fn note(&mut self, note: &str) {
        if self.pause_debug_content {
            return;
        }
        self.debug_content.push_str(note);
    }

    pub fn bool(&mut self, value: bool) {
        self.bits.push(value);
        self.note(if value { "true " } else { "false " });
    }

    pub fn blob(&mut self, data: &[u8]) {
        for byte in data {
            self.u8(*byte);
        }
    }

    pub fn version(&mut self, v: Version) {
        self.note("(");
        self.u4(v.maj);
        self.u5(v.min);
        self.u7(v.patch);
        self.note(&format!("): {} ", v));
    }

    fn uint(&mut self, value: usize, num_bits: u8) {
        assert!(num_bits <= 63);
        assert!(value < 0b1 << num_bits);
        let mut selector = 0b1;
        for _ in 0..num_bits {
            self.bits.push(value & selector > 0);
            selector <<= 1;
        }
        self.note(&format!("{} ", value));
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
        self.uint(value as usize, 5);
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
        self.pause_debug_content = true;
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
        self.pause_debug_content = false;
        self.note(&format!("{} ", value));
    }
    pub fn u32(&mut self, value: u32) {
        self.uint(value as usize, 32);
    }
    pub fn i32(&mut self, value: i32) {
        self.pause_debug_content = true;
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
        self.pause_debug_content = false;
        self.note(&format!("{} ", value));
    }
    pub fn f32(&mut self, value: f32) {
        self.pause_debug_content = true;
        for byte in &value.to_le_bytes() {
            self.u8(*byte);
        }
        self.pause_debug_content = false;
        self.note(&format!("{} ", value));
    }
    pub fn f32_in_range(&mut self, value: f32, min: f32, max: f32) {
        let og = value;
        let value = value.from_range(min, max);
        // Value is now between 0 and 1.
        let value = value.to_range(0.0, 0x10000 as f32).min(0xFFFF as f32);
        // Value is now between 0 and 0xFFFF
        self.u16(value as u16);
        self.note(&format!("({}) ", og));
    }
    pub fn str(&mut self, str: &str) {
        let bytes = str.as_bytes();
        assert!(bytes.len() < std::u16::MAX as usize);
        self.note("(");
        self.u16(bytes.len() as u16);
        for byte in bytes {
            self.u8(*byte);
        }
        self.note(&format!("): \"{}\" ", str));
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

    pub fn end(mut self) -> Vec<u8> {
        let mut remainder = Vec::new();
        // Keep reading u8s as long as we can read at least 8 more bits.
        while (self.bits.len() - self.read_ptr) >= 8 {
            remainder.push(self.u8().unwrap());
        }
        let extra = self.bits.len() - self.read_ptr;
        assert!(extra < 8);
        if extra > 0 {
            remainder.push(self.uint(extra as _).unwrap() as u8);
        }
        remainder
    }

    pub fn bool(&mut self) -> Result<bool, ()> {
        if self.read_ptr >= self.bits.len() {
            return Err(());
        }
        let res = Ok(self.bits[self.read_ptr]);
        self.read_ptr += 1;
        res
    }

    pub fn version(&mut self) -> Result<Version, ()> {
        Ok(Version {
            maj: self.u4()?,
            min: self.u5()?,
            patch: self.u7()?,
        })
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
        Ok(self.uint(16)? as u16)
    }
    pub fn i16(&mut self) -> Result<i16, ()> {
        let bytes = [self.u8()?, self.u8()?];
        Ok(i16::from_le_bytes(bytes))
    }
    pub fn u32(&mut self) -> Result<u32, ()> {
        Ok(self.uint(32)? as u32)
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
