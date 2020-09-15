use crate::util::*;

#[inline]
pub fn compose_u2(bit1: bool, bit0: bool) -> u8 {
    (if bit1 { 0b10 } else { 0b00 }) | (if bit0 { 0b01 } else { 0b00 })
}

#[inline]
pub fn decompose_u2(u2: u8) -> (bool, bool) {
    (u2 & 0b10 != 0, u2 & 0b01 != 0)
}

#[inline]
pub fn compose_u4(bit3: bool, bit2: bool, bit1: bool, bit0: bool) -> u8 {
    (if bit3 { 0b1000 } else { 0b0000 })
        | (if bit2 { 0b0100 } else { 0b0000 })
        | (if bit1 { 0b0010 } else { 0b0000 })
        | (if bit0 { 0b0001 } else { 0b0000 })
}

#[inline]
pub fn decompose_u4(u4: u8) -> (bool, bool, bool, bool) {
    (
        u4 & 0b1000 != 0,
        u4 & 0b0100 != 0,
        u4 & 0b0010 != 0,
        u4 & 0b0001 != 0,
    )
}

#[inline]
pub fn ser_str(buffer: &mut Vec<u8>, text: &str) {
    assert!(text.len() < std::u16::MAX as usize);
    ser_u16(buffer, text.len() as u16);
    buffer.reserve(text.len());
    for b in text.bytes() {
        buffer.push(b);
    }
}

#[inline]
pub fn ser_bool_slice(buffer: &mut Vec<u8>, value: &[bool]) {
    if value.len() == 0 {
        return;
    }
    let mut packed = vec![0; (value.len() + 7) / 8];
    for (index, value) in value.iter().cloned().enumerate() {
        let bit = if value { 0b1 } else { 0b0 };
        packed[index / 8] |= bit << (index % 8);
    }
    buffer.append(&mut packed);
}

#[inline]
pub fn ser_u2_slice(buffer: &mut Vec<u8>, value: &[u8]) {
    if value.len() == 0 {
        return;
    }
    let mut packed = vec![0; (value.len() + 3) / 4];
    for (index, value) in value.iter().cloned().enumerate() {
        debug_assert!(value <= 0b11);
        packed[index / 4] |= value << (index % 4 * 2);
    }
    buffer.append(&mut packed);
}

#[inline]
pub fn ser_u4_slice(buffer: &mut Vec<u8>, value: &[u8]) {
    if value.len() == 0 {
        return;
    }
    let mut packed = vec![0; (value.len() + 1) / 2];
    for (index, value) in value.iter().cloned().enumerate() {
        debug_assert!(value <= 0b1111);
        packed[index / 2] |= value << (index % 2 * 4);
    }
    buffer.append(&mut packed);
}

#[inline]
pub fn ser_u2_u6(buffer: &mut Vec<u8>, u2: u8, u6: u8) {
    debug_assert!(u2 <= 0b11);
    debug_assert!(u6 <= 0b111111);
    buffer.push((u2 << 6) | u6);
}

#[inline]
pub fn ser_u4_u12(buffer: &mut Vec<u8>, u4: u8, u12: u16) {
    debug_assert!(u4 <= 0xF);
    debug_assert!(u12 <= 0xFFF);
    ser_u16(buffer, ((u4 as u16) << 12) | u12);
}

#[inline]
pub fn ser_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

#[inline]
pub fn ser_i16(buffer: &mut Vec<u8>, value: i16) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
pub fn ser_u16(buffer: &mut Vec<u8>, value: u16) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
pub fn ser_i32(buffer: &mut Vec<u8>, value: i32) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
pub fn ser_f32(buffer: &mut Vec<u8>, value: f32) {
    for b in &value.to_be_bytes() {
        buffer.push(*b);
    }
}

#[inline]
pub fn advance_des(slice: &mut &[u8], amount: usize) {
    *slice = &slice[amount..];
}

#[inline]
pub fn des_str(slice: &mut &[u8]) -> Result<String, ()> {
    let len = des_u16(slice)? as usize;
    if slice.len() < len {
        return Err(());
    }
    let buffer = Vec::from(&slice[..len]);
    advance_des(slice, len);
    let string = String::from_utf8(buffer).map_err(|_| ())?;
    Ok(string)
}

#[inline]
pub fn des_bool_slice(slice: &mut &[u8], num_items: usize) -> Result<Vec<bool>, ()> {
    if slice.len() < (num_items + 7) / 8 {
        return Err(());
    }
    let mut res = Vec::with_capacity(num_items);
    if num_items == 0 {
        return Ok(res);
    }
    for index in 0..num_items {
        let bit = (slice[index / 8] >> (index % 8)) & 0b1;
        res.push(bit == 0b1);
    }
    advance_des(slice, (num_items + 7) / 8);
    Ok(res)
}

#[inline]
pub fn des_u2_slice(slice: &mut &[u8], num_items: usize) -> Result<Vec<u8>, ()> {
    if slice.len() < (num_items + 3) / 4 {
        return Err(());
    }
    let mut res = Vec::with_capacity(num_items);
    if num_items == 0 {
        return Ok(res);
    }
    for index in 0..num_items {
        res.push((slice[index / 4] >> (index % 4 * 2)) & 0b11);
    }
    advance_des(slice, (num_items + 3) / 4);
    Ok(res)
}

#[inline]
pub fn des_u4_slice(slice: &mut &[u8], num_items: usize) -> Result<Vec<u8>, ()> {
    if slice.len() < (num_items + 1) / 2 {
        return Err(());
    }
    let mut res = Vec::with_capacity(num_items);
    if num_items == 0 {
        return Ok(res);
    }
    for index in 0..num_items {
        res.push((slice[index / 2] >> (index % 2 * 4)) & 0b1111);
    }
    advance_des(slice, (num_items + 1) / 2);
    Ok(res)
}

#[inline]
pub fn des_u2_u6(slice: &mut &[u8]) -> Result<(u8, u8), ()> {
    let packed = des_u8(slice)?;
    Ok((packed >> 6, packed & 0b111111))
}

#[inline]
pub fn des_u4_u12(slice: &mut &[u8]) -> Result<(u16, u16), ()> {
    let packed = des_u16(slice)?;
    Ok((packed >> 12, packed & 0xFFF))
}

#[inline]
pub fn des_u8(slice: &mut &[u8]) -> Result<u8, ()> {
    if slice.len() < 1 {
        return Err(());
    }
    let res = u8::from_be_bytes([slice[0]]);
    advance_des(slice, 1);
    Ok(res)
}

#[inline]
pub fn des_i16(slice: &mut &[u8]) -> Result<i16, ()> {
    if slice.len() < 2 {
        return Err(());
    }
    let res = i16::from_be_bytes([slice[0], slice[1]]);
    advance_des(slice, 2);
    Ok(res)
}

#[inline]
pub fn des_u16(slice: &mut &[u8]) -> Result<u16, ()> {
    if slice.len() < 2 {
        return Err(());
    }
    let res = u16::from_be_bytes([slice[0], slice[1]]);
    advance_des(slice, 2);
    Ok(res)
}

#[inline]
pub fn des_i32(slice: &mut &[u8]) -> Result<i32, ()> {
    if slice.len() < 4 {
        return Err(());
    }
    let res = i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
    advance_des(slice, 4);
    Ok(res)
}

#[inline]
pub fn des_f32(slice: &mut &[u8]) -> Result<f32, ()> {
    if slice.len() < 4 {
        return Err(());
    }
    let res = f32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
    advance_des(slice, 4);
    Ok(res)
}

#[inline]
pub fn pack_value(value: f32, range: (f32, f32)) -> u16 {
    let value = value.from_range(range.0, range.1);
    // Value is now between 0 and 1.
    let value = value.to_range(0.0, 0x10000 as f32).min(0xFFFF as f32);
    // Value is now between 0 and 0xFFFF
    value as u16
}

#[inline]
pub fn unpack_value(value: u16, range: (f32, f32)) -> f32 {
    let value = value as f32;
    // Value is now between 0 and 0xFFFF
    let value = value.from_range(0.0, 0xFFFF as f32);
    // Value is now between 0 and 1.
    value.to_range(range.0, range.1)
}
