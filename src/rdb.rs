use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};

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
        Ok(RDB {
            buffer,
            data: vec![],
            version: "".to_string(),
            hash_size: 0,
            expire_hash_size: 0,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        let (mut pointer, buffer_size) = (0, self.buffer.len());
        let mut metadata = HashMap::new();
        let mut data = vec![];

        let _magic_str = String::from_utf8(self.buffer[pointer..pointer + 5].to_vec())?;
        pointer += 5;

        let rdb_version = String::from_utf8(self.buffer[pointer..pointer + 4].to_vec())?;
        pointer += 4;

        while pointer < buffer_size {
            match self.buffer[pointer] {
                0xFA => {
                    let (k, p) = self.string_encoding(pointer + 1)?;
                    let (v, p) = self.string_encoding(p)?;
                    pointer = p;
                    metadata.insert(k, v);
                }
                0xFE => {
                    pointer += 1;
                    // println!("DB number: {}", self.buffer[pointer]);
                    pointer += 1;
                }
                0xFB => {
                    let (expire_size, p, _) = self.length_encoding(pointer)?;
                    let (hash_size, p, _) = self.length_encoding(p)?;
                    pointer = p + 1;

                    self.expire_hash_size = expire_size;
                    self.hash_size = hash_size;

                    for _ in 0..hash_size {
                        let _val_type = self.buffer[pointer];
                        pointer += 1;
                        let (key, p) = self.string_encoding(pointer)?;
                        let (val, p) = self.string_encoding(p)?;
                        data.push((
                            key,
                            val,
                            SystemTime::now() + Duration::from_secs(u32::MAX as _),
                        ));
                        pointer = p;
                    }
                }
                0xFF => break,
                _ => pointer += 1,
            }
        }

        self.version = rdb_version;
        self.data = data;
        Ok(())
    }

    fn length_encoding(&self, p: usize) -> Result<(usize, usize, bool)> {
        let code = self.buffer[p] & 192;
        match code {
            0 => Ok((self.buffer[p] as usize, p + 1, false)),
            64 => {
                let (mut byte1, byte2) = (self.buffer[p], self.buffer[p + 1]);
                byte1 ^= 64;
                let val = (byte2 as u16) << 8 | (byte1 as u16);
                return Ok((val as usize, p + 2, false));
            }
            128 => {
                let p = p + 1;
                let (b1, b2, b3, b4) = (
                    self.buffer[p + 3],
                    self.buffer[p + 2],
                    self.buffer[p + 1],
                    self.buffer[p],
                );
                let val = (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32);
                Ok((val as usize, p + 4, false))
            }
            192 => {
                let special_code = self.buffer[p] ^ 192;
                match special_code {
                    0 => Ok((self.buffer[p + 1] as usize, p + 2, true)),
                    1 => {
                        let (byte1, byte2) = (self.buffer[p + 1], self.buffer[p + 2]);
                        let val = ((byte2 as u16) << 8 | (byte1 as u16)) as usize;
                        Ok((val as usize, p + 2, true))
                    }
                    2 => {
                        let (b1, b2, b3, b4) = (
                            self.buffer[p + 4],
                            self.buffer[p + 3],
                            self.buffer[p + 2],
                            self.buffer[p + 1],
                        );

                        let val =
                            (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32);
                        Ok((val as usize, p + 5, true))
                    }
                    3 => {
                        // TODO Need to implement code 3
                        Ok((0, p + 1, true))
                    }
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
