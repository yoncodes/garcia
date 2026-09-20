mod gm;

use std::sync::Arc;

use common::network::crypto::Crypto;
use protocol::{
    cg::{C2gConnectGameServer, C2gGetSeed, G2cConnectGameServer, G2cGetSeed, G2cHeartbeat, Id},
    cs::{DcNetWorkingNotifyBattlePassTask, DcNetWorkingNotifyTaskCycle},
    pbcommon::DcNetDataPhy,
    proids::{self, NotifyId},
    prost::Message,
};
use tokio::sync::mpsc;

use super::{
    app::AppState,
    context::{CommandReply, HandlerContext},
    error::{NetworkError, NetworkResult},
    gm_command::GmCommandEnvelope,
    packet::{ClientPacket, ServerPacket},
    router,
};

pub(super) struct GatewaySession {
    crypto: Option<Crypto>,
    context: HandlerContext,
    return_number: u32,
    gm_tx: mpsc::UnboundedSender<GmCommandEnvelope>,
    last_heat_poll: i32,
}

impl GatewaySession {
    pub fn new(state: Arc<AppState>, gm_tx: mpsc::UnboundedSender<GmCommandEnvelope>) -> Self {
        Self {
            crypto: None,
            context: HandlerContext::new(state),
            return_number: 0,
            gm_tx,
            last_heat_poll: 0,
        }
    }

    pub(super) async fn handle_packet(&mut self, bytes: &[u8]) -> NetworkResult<Vec<Vec<u8>>> {
        let mut bytes = bytes.to_vec();
        if let Some(crypto) = &mut self.crypto {
            crypto.decrypt(&mut bytes)?;
        }
        let packet = ClientPacket::decode(&bytes)?;
        let message = command_name(packet.proto_id);
        tracing::info!(
            cmd_id = packet.proto_id,
            cmd_name = message,
            sequence = packet.timestamp,
            ack = packet.ack,
            payload_size = packet.payload.len(),
            "received packet"
        );

        let outbound = match Id::try_from(i32::from(packet.proto_id)) {
            Ok(Id::C2gGetSeedId) => return self.exchange_keys(packet).map(|packet| vec![packet]),
            Ok(Id::C2gConnectGameServerId) => {
                let request = C2gConnectGameServer::decode(packet.payload.as_slice())?;
                tracing::info!(uid = request.uid, "game session authenticated");
                self.context.load_player(request.uid).await?;
                self.context
                    .state
                    .register_session(request.uid, self.gm_tx.clone());
                let state = self.context.state.clone();
                let now = common::time::ServerTime::now_seconds_i32();
                let battle_pass_updates = self.context.update_player()?.battle_pass_login_updates(
                    &state.tables,
                    now,
                    common::config().server.zone_offset,
                );
                let daily_updates = self.context.player()?.daily_login_updates(&state.tables);
                let heat = self
                    .context
                    .update_player()?
                    .refresh_heat(&state.tables, now)
                    .map_err(|error| NetworkError::InvalidHeatExchange(error.to_string()))?;
                self.last_heat_poll = now;
                self.context.save_player().await?;
                let mut replies = vec![encode_message_reply(
                    Id::G2cConnectGameServerId as u16,
                    G2cConnectGameServer { ret: 1 },
                )?];
                for task in battle_pass_updates {
                    replies.push(encode_message_reply(
                        NotifyId::DcNetWorkingNotifyBattlePassTask as u16,
                        DcNetWorkingNotifyBattlePassTask { task: Some(task) },
                    )?);
                }
                for task in daily_updates {
                    replies.push(encode_message_reply(
                        NotifyId::DcNetWorkingNotifyTaskCycle as u16,
                        DcNetWorkingNotifyTaskCycle { task: Some(task) },
                    )?);
                }
                replies.push(encode_message_reply(
                    NotifyId::DcNetDataPhy as u16,
                    DcNetDataPhy {
                        item: Some(heat.item),
                        cd_time: heat.cd_time,
                    },
                )?);
                replies
            }
            Ok(Id::C2gHeartbeatId) => vec![encode_message_reply(
                Id::G2cHeartbeatId as u16,
                G2cHeartbeat { ret: 1 },
            )?],
            _ => {
                if self.context.player.is_none() {
                    return Err(NetworkError::Unauthenticated);
                }
                if !router::dispatch_command(&mut self.context, packet).await? {
                    tracing::debug!(cmd_name = message, "unhandled gateway packet");
                    return Ok(Vec::new());
                }
                self.context.save_player().await?;
                let outbound = self.context.take_outbound();
                if outbound.is_empty() {
                    return Err(NetworkError::MissingHandlerReply);
                }
                outbound
            }
        };

        let mut responses = Vec::with_capacity(outbound.len());
        for reply in outbound {
            let mut response = self.encode_reply(reply)?;
            self.crypto
                .as_mut()
                .ok_or(NetworkError::MissingKeyExchange)?
                .encrypt(&mut response)?;
            responses.push(response);
        }
        Ok(responses)
    }

    pub fn player_uid(&self) -> Option<i64> {
        self.context.player_uid()
    }

    pub async fn poll_periodic(&mut self) -> NetworkResult<Vec<Vec<u8>>> {
        if self.context.player.is_none() {
            return Ok(Vec::new());
        }
        let now = common::time::ServerTime::now_seconds_i32();
        if now <= self.last_heat_poll {
            return Ok(Vec::new());
        }
        self.last_heat_poll = now;

        let state = self.context.state.clone();
        let heat_item_id = state.tables.cultivation_constants.heat_item_id;
        let previous = self
            .context
            .player()?
            .items
            .iter()
            .find(|item| item.item_id == heat_item_id)
            .map_or(0, |item| item.amount);
        let heat = self
            .context
            .update_player()?
            .refresh_heat(&state.tables, now)
            .map_err(|error| NetworkError::InvalidHeatExchange(error.to_string()))?;
        if heat.item.amount == previous {
            return Ok(Vec::new());
        }

        self.context.save_player().await?;
        let reply = encode_message_reply(
            NotifyId::DcNetDataPhy as u16,
            DcNetDataPhy {
                item: Some(heat.item),
                cd_time: heat.cd_time,
            },
        )?;
        let mut packet = self.encode_reply(reply)?;
        self.crypto
            .as_mut()
            .ok_or(NetworkError::MissingKeyExchange)?
            .encrypt(&mut packet)?;
        Ok(vec![packet])
    }

    pub fn unregister(&self) {
        if let Some(uid) = self.player_uid() {
            self.context.state.unregister_session(uid, &self.gm_tx);
        }
    }

    fn exchange_keys(&mut self, packet: ClientPacket) -> NetworkResult<Vec<u8>> {
        if self.crypto.is_some() {
            return Err(NetworkError::DuplicateKeyExchange);
        }
        let request = C2gGetSeed::decode(packet.payload.as_slice())?;
        let (crypto, seeds) = Crypto::negotiate(request.send_seed, request.receive_seed);

        let reply = encode_message_reply(
            Id::G2cGetSeedId as u16,
            G2cGetSeed {
                ret: 1,
                send_seed: seeds.send,
                receive_seed: seeds.receive,
            },
        )?;
        let response = self.encode_reply(reply)?;
        // The seed response is plaintext; both sides enable RC4 after receiving it.
        self.crypto = Some(crypto);
        Ok(response)
    }

    fn encode_reply(&mut self, reply: CommandReply) -> NetworkResult<Vec<u8>> {
        self.return_number = self.return_number.wrapping_add(1);
        encode_server_packet(self.return_number, reply.cmd_id, reply.body)
    }
}

pub(super) fn command_name(cmd_id: u16) -> &'static str {
    Id::try_from(i32::from(cmd_id))
        .map(|id| id.as_str_name())
        .or_else(|_| proids::Id::try_from(i32::from(cmd_id)).map(|id| id.as_str_name()))
        .or_else(|_| proids::NotifyId::try_from(i32::from(cmd_id)).map(|id| id.as_str_name()))
        .unwrap_or("UNKNOWN")
}

fn encode_message_reply(cmd_id: u16, message: impl Message) -> NetworkResult<CommandReply> {
    let mut payload = Vec::with_capacity(message.encoded_len());
    message.encode(&mut payload)?;
    Ok(CommandReply {
        cmd_id,
        body: payload,
    })
}

fn encode_server_packet(
    return_number: u32,
    cmd_id: u16,
    payload: Vec<u8>,
) -> NetworkResult<Vec<u8>> {
    Ok(ServerPacket {
        return_number,
        proto_id: cmd_id,
        payload,
    }
    .encode()?)
}

#[cfg(test)]
mod tests;
