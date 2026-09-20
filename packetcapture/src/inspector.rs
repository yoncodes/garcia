use std::path::PathBuf;

use common::network::{
    kcp::Kcp,
    packet::{ClientPacket, ServerPacket},
};
use protocol::{
    cg::{C2gGetSeed, Id as GatewayId},
    proids,
    prost::Message,
};

use crate::{Direction, capture::Capture, key_recovery::RecoveredStreams};

pub(crate) struct Inspector {
    conversation: Option<u32>,
    client_to_gateway: Option<Kcp>,
    gateway_to_client: Option<Kcp>,
    client_stream: Vec<u8>,
    gateway_stream: Vec<u8>,
    client_seeds: Option<C2gGetSeed>,
    crypto: Option<RecoveredStreams>,
    capture_base: PathBuf,
    capture: Option<Capture>,
    now: u32,
}

impl Inspector {
    pub(crate) fn new(capture_base: impl Into<PathBuf>) -> Self {
        Self {
            conversation: None,
            client_to_gateway: None,
            gateway_to_client: None,
            client_stream: Vec::new(),
            gateway_stream: Vec::new(),
            client_seeds: None,
            crypto: None,
            capture_base: capture_base.into(),
            capture: None,
            now: 0,
        }
    }

    pub(crate) fn observe(&mut self, direction: Direction, datagram: &[u8]) {
        let Some(conversation) = Kcp::conversation(datagram) else {
            return;
        };
        if self.conversation != Some(conversation) {
            self.reset(conversation);
        }

        self.now = self.now.wrapping_add(10);
        let messages = {
            let kcp = match direction {
                Direction::ClientToGateway => self.client_to_gateway.as_mut().unwrap(),
                Direction::GatewayToClient => self.gateway_to_client.as_mut().unwrap(),
            };
            if let Err(error) = kcp.input(datagram, self.now) {
                tracing::warn!(%error, direction = direction.label(), "KCP inspection failed");
                return;
            }
            kcp.update(self.now);
            kcp.take_output();
            let mut messages = Vec::new();
            while let Some(message) = kcp.recv() {
                messages.push(message);
            }
            messages
        };

        for message in messages {
            let packets = match direction {
                Direction::ClientToGateway => drain_packets(&mut self.client_stream, &message),
                Direction::GatewayToClient => drain_packets(&mut self.gateway_stream, &message),
            };
            for packet in packets {
                self.inspect_packet(direction, packet);
            }
        }
    }

    fn reset(&mut self, conversation: u32) {
        self.conversation = Some(conversation);
        self.client_to_gateway = Some(Kcp::new(conversation));
        self.gateway_to_client = Some(Kcp::new(conversation));
        self.client_stream.clear();
        self.gateway_stream.clear();
        self.client_seeds = None;
        self.crypto = None;
        self.now = 0;
        self.capture = Capture::start(&self.capture_base, conversation)
            .map_err(|error| tracing::warn!(%error, "could not create packet capture"))
            .ok();
    }

    fn inspect_packet(&mut self, direction: Direction, mut bytes: Vec<u8>) {
        if let Some(crypto) = &mut self.crypto
            && !crypto.decrypt(direction, &mut bytes)
        {
            tracing::warn!(direction = direction.label(), "could not decrypt packet");
            return;
        }

        match direction {
            Direction::ClientToGateway => self.inspect_client_packet(bytes),
            Direction::GatewayToClient => self.inspect_gateway_packet(bytes),
        }
    }

    fn inspect_client_packet(&mut self, bytes: Vec<u8>) {
        let Ok(packet) = ClientPacket::decode(&bytes) else {
            tracing::warn!(direction = "c2g", "could not decode packet");
            return;
        };
        let name = command_name(Direction::ClientToGateway, packet.proto_id);
        tracing::info!(
            direction = "C2G",
            cmd_id = packet.proto_id,
            cmd_name = name.as_str(),
            sequence = packet.timestamp,
            ack = packet.ack,
            payload_size = packet.payload.len(),
            "captured packet"
        );
        self.dump(Direction::ClientToGateway, packet.proto_id, &name, &bytes);

        if packet.proto_id == GatewayId::C2gGetSeedId as u16 {
            match C2gGetSeed::decode(packet.payload.as_slice()) {
                Ok(seeds) => self.client_seeds = Some(seeds),
                Err(error) => tracing::warn!(%error, "could not decode client seed exchange"),
            }
        }
    }

    fn inspect_gateway_packet(&mut self, bytes: Vec<u8>) {
        let Ok(packet) = ServerPacket::decode(&bytes) else {
            tracing::warn!(direction = "g2c", "could not decode packet");
            return;
        };
        let name = command_name(Direction::GatewayToClient, packet.proto_id);
        tracing::info!(
            direction = "G2C",
            cmd_id = packet.proto_id,
            cmd_name = name.as_str(),
            return_number = packet.return_number,
            payload_size = packet.payload.len(),
            "captured packet"
        );
        self.dump(Direction::GatewayToClient, packet.proto_id, &name, &bytes);

        if self.crypto.is_none() && packet.proto_id == GatewayId::G2cGetSeedId as u16 {
            let recovered = protocol::cg::G2cGetSeed::decode(packet.payload.as_slice())
                .ok()
                .and_then(|server| {
                    self.client_seeds
                        .as_ref()
                        .and_then(|client| RecoveredStreams::from_exchange(client, &server))
                });
            if let Some(crypto) = recovered {
                self.crypto = Some(crypto);
                tracing::info!("recovered RC4 streams");
            } else {
                tracing::warn!("could not recover RC4 streams");
            }
        }
    }

    fn dump(&mut self, direction: Direction, command_id: u16, name: &str, bytes: &[u8]) {
        if let Some(capture) = &mut self.capture
            && let Err(error) = capture.write(direction, command_id, name, bytes)
        {
            tracing::warn!(%error, "could not write packet capture");
        }
    }
}

fn drain_packets(stream: &mut Vec<u8>, chunk: &[u8]) -> Vec<Vec<u8>> {
    stream.extend_from_slice(chunk);
    let mut packets = Vec::new();
    let mut offset = 0;

    while stream.len() - offset >= 2 {
        let body_length = u16::from_be_bytes([stream[offset], stream[offset + 1]]) as usize;
        let packet_length = body_length + 2;
        if stream.len() - offset < packet_length {
            break;
        }
        packets.push(stream[offset..offset + packet_length].to_vec());
        offset += packet_length;
    }

    stream.drain(..offset);
    packets
}

fn command_name(direction: Direction, command_id: u16) -> String {
    let name = GatewayId::try_from(i32::from(command_id))
        .map(|id| id.as_str_name())
        .or_else(|_| proids::Id::try_from(i32::from(command_id)).map(|id| id.as_str_name()))
        .or_else(|_| proids::NotifyId::try_from(i32::from(command_id)).map(|id| id.as_str_name()))
        .unwrap_or("UNKNOWN");
    if name != "UNKNOWN" || matches!(direction, Direction::ClientToGateway) {
        return name.to_owned();
    }

    command_id
        .checked_sub(1)
        .map(|id| command_name(Direction::ClientToGateway, id))
        .filter(|name| name != "UNKNOWN")
        .map(|name| name.replacen("Param", "Res", 1))
        .unwrap_or_else(|| name.to_owned())
}
