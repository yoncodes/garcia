use std::{collections::HashMap, sync::OnceLock};

use common::network::crypto::{DH_BASE, DH_PRIME, KEY_SALT, Rc4, crypt_packet_body, mod_pow};
use protocol::cg::{C2gGetSeed, G2cGetSeed};

use crate::Direction;

// ceil(sqrt(DH_PRIME - 1)), the search width required by baby-step giant-step.
const BABY_STEP_COUNT: i64 = 46_341;

pub(crate) struct RecoveredStreams {
    client_to_gateway: Rc4,
    gateway_to_client: Rc4,
}

impl RecoveredStreams {
    pub(crate) fn from_exchange(client: &C2gGetSeed, server: &G2cGetSeed) -> Option<Self> {
        let client_send_secret = discrete_logarithm(i64::from(client.send_seed))?;
        let client_receive_secret = discrete_logarithm(i64::from(client.receive_seed))?;
        let client_to_gateway_key =
            mod_pow(i64::from(server.send_seed), client_send_secret, DH_PRIME);
        let gateway_to_client_key = mod_pow(
            i64::from(server.receive_seed),
            client_receive_secret,
            DH_PRIME,
        );
        Some(Self {
            client_to_gateway: Rc4::new(format!("{KEY_SALT}{client_to_gateway_key}").as_bytes()),
            gateway_to_client: Rc4::new(format!("{KEY_SALT}{gateway_to_client_key}").as_bytes()),
        })
    }

    pub(crate) fn decrypt(&mut self, direction: Direction, packet: &mut [u8]) -> bool {
        let cipher = match direction {
            Direction::ClientToGateway => &mut self.client_to_gateway,
            Direction::GatewayToClient => &mut self.gateway_to_client,
        };
        crypt_packet_body(packet, cipher).is_ok()
    }
}

fn discrete_logarithm(value: i64) -> Option<i64> {
    static BABY_STEPS: OnceLock<HashMap<i64, i64>> = OnceLock::new();
    let baby_steps = BABY_STEPS.get_or_init(|| {
        let mut steps = HashMap::with_capacity(BABY_STEP_COUNT as usize);
        let mut value = 1_i64;
        for exponent in 0..BABY_STEP_COUNT {
            steps.entry(value).or_insert(exponent);
            value = value * DH_BASE % DH_PRIME;
        }
        steps
    });
    let factor = mod_pow(DH_BASE, DH_PRIME - 1 - BABY_STEP_COUNT, DH_PRIME);
    let mut giant = value.rem_euclid(DH_PRIME);
    for index in 0..=BABY_STEP_COUNT {
        if let Some(baby) = baby_steps.get(&giant) {
            let exponent = index * BABY_STEP_COUNT + baby;
            if mod_pow(DH_BASE, exponent, DH_PRIME) == value {
                return Some(exponent);
            }
        }
        giant = giant * factor % DH_PRIME;
    }
    None
}
