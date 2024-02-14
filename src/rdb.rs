use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod op_code {
    pub const AUX: u8 = 0xFA;
    pub const RESIZEDB: u8 = 0xFB;
    pub const EXPIRETIME_MS: u8 = 0xFC;
    pub const EXPIRETIME: u8 = 0xFD;
    pub const SELECTDB: u8 = 0xFE;
    pub const EOF: u8 = 0xFF;
}
pub mod length_code {
    pub const N6BIT: u8 = 0;
    pub const N14BIT: u8 = 64;
    pub const NEXT4BYTES: u8 = 128;
    pub const SPECIAL: u8 = 192;
}

pub mod int_str_code {
    pub const N8BIT: u8 = 0;
    pub const N16BIT: u8 = 1;
    pub const N32BIT: u8 = 2;
    pub const COMPRESSED: u8 = 3;
}

// Everything I need to know https://rdb.fnordig.de/file_format.html
pub struct RDB {
    buffer: Vec<u8>,
    pub data: Vec<(String, String, SystemTime)>,
    pub version: String,
    pub hash_size: usize,
    pub expire_hash_size: usize,
}

impl RDB {
    pub fn new(dir: String, dbfilename: String) -> Result<RDB> {
        let buffer = fs::read(format!("{}/{}", dir, dbfilename))?;
        let mut rdb = RDB {
            buffer,
            data: vec![],
            version: "".to_string(),
            hash_size: 0,
            expire_hash_size: 0,
        };
        rdb.load()?;
        Ok(rdb)
    }

    pub fn load(&mut self) -> Result<()> {
        println!("Parsing dump file...");

        let (mut pointer, buffer_size) = (0, self.buffer.len());
        let mut metadata = HashMap::new();
        let mut data = vec![];

        let magic_str = String::from_utf8(self.buffer[pointer..pointer + 5].to_vec())?;
        println!("Magic String: {magic_str}");
        pointer += 5;

        let rdb_version = String::from_utf8(self.buffer[pointer..pointer + 4].to_vec())?;
        println!("RDB Version: {rdb_version}");
        pointer += 4;

        let mut expire_time = SystemTime::now() + Duration::from_secs(u32::MAX as _);

        while pointer < buffer_size {
            match self.buffer[pointer] {
                op_code::AUX => {
                    let (k, p) = self.string_encoding(pointer + 1)?;
                    let (v, p) = self.string_encoding(p)?;
                    println!("Metadata - {k} : {v}");
                    pointer = p;
                    metadata.insert(k, v);
                }
                op_code::SELECTDB => {
                    pointer += 1;
                    println!("DB number: {}", self.buffer[pointer]);
                    pointer += 1;
                }
                op_code::RESIZEDB => {
                    let (db_size, p, _) = self.length_encoding(pointer)?;
                    let (expire_size, p, _) = self.length_encoding(p)?;
                    pointer = p + 1;
                    self.hash_size = db_size;
                    self.expire_hash_size = expire_size;
                }
                op_code::EXPIRETIME => {
                    pointer += 1;
                    let mut b = [0; 8];
                    for i in 0..4 {
                        b[i] = self.buffer[pointer + i];
                    }
                    let val = u64::from_le_bytes(b);
                    expire_time = UNIX_EPOCH + Duration::from_secs(val);
                    pointer += 4;
                }
                op_code::EXPIRETIME_MS => {
                    pointer += 1;
                    let mut b = [0; 8];
                    for i in 0..8 {
                        b[i] = self.buffer[pointer + i];
                    }
                    let val = u64::from_le_bytes(b);
                    expire_time = UNIX_EPOCH + Duration::from_millis(val);
                    pointer += 8;
                }
                op_code::EOF => break,
                _ => {
                    let _val_type = self.buffer[pointer];
                    pointer += 1;
                    let (key, p) = self.string_encoding(pointer)?;
                    let (val, p) = self.string_encoding(p)?;
                    data.push((key, val, expire_time));
                    pointer = p;
                }
            }
        }

        self.version = rdb_version;
        self.data = data;
        Ok(())
    }

    fn length_encoding(&self, p: usize) -> Result<(usize, usize, bool)> {
        let code = self.buffer[p] & 192;
        match code {
            length_code::N6BIT => Ok((self.buffer[p] as usize, p + 1, false)),
            length_code::N14BIT => {
                let (mut byte1, byte2) = (self.buffer[p], self.buffer[p + 1]);
                byte1 ^= 64;
                let val = u16::from_le_bytes([byte1, byte2]);
                return Ok((val as usize, p + 2, false));
            }
            length_code::NEXT4BYTES => {
                let p = p + 1;
                let mut b = [0; 4];
                for i in 0..4 {
                    b[i] = self.buffer[p + i];
                }
                let val = u32::from_le_bytes(b);
                Ok((val as usize, p + 4, false))
            }
            length_code::SPECIAL => {
                let special_code = self.buffer[p] ^ 192;
                match special_code {
                    int_str_code::N8BIT => Ok((self.buffer[p + 1] as usize, p + 2, true)),
                    int_str_code::N16BIT => {
                        let (byte1, byte2) = (self.buffer[p + 1], self.buffer[p + 2]);
                        let val = ((byte2 as u16) << 8 | (byte1 as u16)) as usize;
                        Ok((val as usize, p + 2, true))
                    }
                    int_str_code::N32BIT => {
                        let p = p + 1;
                        let mut b = [0; 4];
                        for i in 0..4 {
                            b[i] = self.buffer[p + i];
                        }
                        let val = u32::from_le_bytes(b);
                        Ok((val as usize, p + 4, true))
                    }
                    int_str_code::COMPRESSED => todo!(),
                    _ => Ok((0, p + 1, true)),
                }
            }
            _ => return Ok((self.buffer[p] as usize, p + 1, false)),
        }
    }

    fn string_encoding(&self, p: usize) -> Result<(String, usize)> {
        let (l, p, special) = self.length_encoding(p)?;
        if special {
            return Ok((l.to_string(), p));
        }
        let s = self.buffer[p..p + l as usize].to_vec();
        let s = String::from_utf8(s)?;
        Ok((s, p + l as usize))
    }
}
