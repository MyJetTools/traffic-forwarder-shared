use std::str;

pub fn serialize_u32(data: &mut Vec<u8>, v: u32) {
    data.extend(&v.to_le_bytes());
}

pub fn serialize_pascal_string(data: &mut Vec<u8>, str: &str) {
    let str_len = str.len() as u8;
    data.push(str_len);
    data.extend(str.as_bytes());
}

pub fn serialize_payload(data: &mut Vec<u8>, payload: &[u8]) {
    let str_len = payload.len() as u32;
    serialize_u32(data, str_len);
    data.extend_from_slice(payload);
}
