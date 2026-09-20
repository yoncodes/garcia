use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

pub const DH_BASE: i64 = 3;
pub const DH_PRIME: i64 = 2_147_483_587;
pub const KEY_SALT: &str = "YUELONGYIN";

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    #[error("encrypted packet is missing its size field")]
    PacketTooShort,
    #[error("encrypted packet length mismatch: declared {expected}, received {actual}")]
    PacketLengthMismatch { expected: usize, actual: usize },
}

pub struct ServerSeeds {
    pub send: i32,
    pub receive: i32,
}

pub struct Crypto {
    send: Rc4,
    receive: Rc4,
}

impl Crypto {
    pub fn negotiate(client_send: i32, client_receive: i32) -> (Self, ServerSeeds) {
        let receive_secret = random_secret();
        let send_secret = random_secret();
        let seeds = ServerSeeds {
            send: mod_pow(DH_BASE, receive_secret, DH_PRIME) as i32,
            receive: mod_pow(DH_BASE, send_secret, DH_PRIME) as i32,
        };
        let receive_key = mod_pow(client_send as i64, receive_secret, DH_PRIME);
        let send_key = mod_pow(client_receive as i64, send_secret, DH_PRIME);
        (
            Self {
                send: Rc4::new(format!("{KEY_SALT}{send_key}").as_bytes()),
                receive: Rc4::new(format!("{KEY_SALT}{receive_key}").as_bytes()),
            },
            seeds,
        )
    }

    pub fn encrypt(&mut self, packet: &mut [u8]) -> Result<(), CryptoError> {
        crypt_packet_body(packet, &mut self.send)
    }

    pub fn decrypt(&mut self, packet: &mut [u8]) -> Result<(), CryptoError> {
        crypt_packet_body(packet, &mut self.receive)
    }
}

pub fn crypt_packet_body(packet: &mut [u8], crypt: &mut Rc4) -> Result<(), CryptoError> {
    if packet.len() < 2 {
        return Err(CryptoError::PacketTooShort);
    }
    let body_size = u16::from_be_bytes(packet[..2].try_into().unwrap()) as usize;
    if packet.len() != body_size + 2 {
        return Err(CryptoError::PacketLengthMismatch {
            expected: body_size + 2,
            actual: packet.len(),
        });
    }
    crypt.apply(&mut packet[2..]);
    Ok(())
}

pub fn mod_pow(mut base: i64, mut exponent: i64, modulus: i64) -> i64 {
    let mut result = 1;
    base %= modulus;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = result * base % modulus;
        }
        base = base * base % modulus;
        exponent >>= 1;
    }
    result
}

fn random_secret() -> i64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    hasher.write_u128(crate::time::ServerTime::now_nanos());
    (hasher.finish() % (DH_PRIME as u64 - 1) + 1) as i64
}

pub struct Rc4 {
    state: [u8; 256],
    i: u8,
    j: u8,
}

impl Rc4 {
    pub fn new(key: &[u8]) -> Self {
        assert!(!key.is_empty());
        let mut state = std::array::from_fn(|index| index as u8);
        let mut j = 0_u8;
        for i in 0..256 {
            j = j.wrapping_add(state[i]).wrapping_add(key[i % key.len()]);
            state.swap(i, j as usize);
        }
        Self { state, i: 0, j: 0 }
    }

    fn apply(&mut self, data: &mut [u8]) {
        for byte in data {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.state[self.i as usize]);
            self.state.swap(self.i as usize, self.j as usize);
            let index = self.state[self.i as usize].wrapping_add(self.state[self.j as usize]);
            *byte ^= self.state[index as usize];
        }
    }
}
