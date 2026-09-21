use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use common::time::ServerTime;
use configs::GameTables;
use database::models::game::player_state::PlayerRecord;
use protocol::prost::Message;

use crate::logic::Player;

use super::{
    app::AppState,
    error::{NetworkError, NetworkResult},
    packet::ClientPacket,
};

pub(crate) struct CommandReply {
    pub cmd_id: u16,
    pub body: Vec<u8>,
}

pub(crate) struct HandlerContext {
    pub state: Arc<AppState>,
    pub player: Option<Player>,
    last_saved_player: Option<PlayerRecord>,
    dirty: bool,
    reply: Option<CommandReply>,
    pushes: Vec<CommandReply>,
}

pub(crate) struct PlayerUpdate<'a> {
    player: &'a mut Player,
    tables: &'a GameTables,
}

impl<'a> PlayerUpdate<'a> {
    pub fn with_tables(self) -> (&'a mut Player, &'a GameTables) {
        (self.player, self.tables)
    }
}

impl Deref for PlayerUpdate<'_> {
    type Target = Player;

    fn deref(&self) -> &Self::Target {
        self.player
    }
}

impl DerefMut for PlayerUpdate<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.player
    }
}

impl HandlerContext {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            player: None,
            last_saved_player: None,
            dirty: false,
            reply: None,
            pushes: Vec::new(),
        }
    }

    pub async fn load_player(&mut self, uid: i64) -> NetworkResult<()> {
        let defaults = &common::config().account_defaults;
        let stored = database::db::player_state::load(&self.state.db, uid).await?;
        let mut player = match &stored {
            Some(record) => Player::from_record(record.clone(), &self.state.tables, defaults),
            None => Player::new(uid, &self.state.tables, defaults),
        };
        let last_login_time = ServerTime::now_seconds_i32();
        player.last_login_time = last_login_time;
        let record = player.to_record();
        let only_login_time_changed = stored.as_ref().is_some_and(|stored| {
            let mut expected = stored.clone();
            expected.last_login_time = last_login_time;
            expected == record
        });
        if only_login_time_changed {
            database::db::player_state::update_last_login(&self.state.db, uid, last_login_time)
                .await?;
        } else {
            database::db::player_state::save(&self.state.db, &record).await?;
        }
        self.last_saved_player = Some(record);
        self.player = Some(player);
        Ok(())
    }

    pub async fn save_player(&mut self) -> NetworkResult<()> {
        if self.dirty {
            let record = self.player()?.to_record();
            if self.last_saved_player.as_ref() != Some(&record) {
                database::db::player_state::save(&self.state.db, &record).await?;
                self.last_saved_player = Some(record);
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn player_uid(&self) -> Option<i64> {
        self.player.as_ref().map(|player| player.uid)
    }

    pub fn player(&self) -> NetworkResult<&Player> {
        self.player.as_ref().ok_or(NetworkError::Unauthenticated)
    }

    pub fn update_player(&mut self) -> NetworkResult<PlayerUpdate<'_>> {
        self.dirty = true;
        let player = self.player.as_mut().ok_or(NetworkError::Unauthenticated)?;
        Ok(PlayerUpdate {
            player,
            tables: &self.state.tables,
        })
    }

    pub fn send_reply(
        &mut self,
        request: &ClientPacket,
        message: impl Message,
    ) -> NetworkResult<()> {
        let mut body = Vec::with_capacity(message.encoded_len());
        message.encode(&mut body)?;
        self.reply = Some(CommandReply {
            cmd_id: request.proto_id + 1,
            body,
        });
        Ok(())
    }

    #[cfg(test)]
    pub fn take_reply(&mut self) -> Option<CommandReply> {
        self.reply.take()
    }

    pub fn push(&mut self, cmd_id: u16, message: impl Message) -> NetworkResult<()> {
        let mut body = Vec::with_capacity(message.encoded_len());
        message.encode(&mut body)?;
        self.pushes.push(CommandReply { cmd_id, body });
        Ok(())
    }

    pub fn take_outbound(&mut self) -> Vec<CommandReply> {
        let mut outbound = std::mem::take(&mut self.pushes);
        if let Some(reply) = self.reply.take() {
            outbound.push(reply);
        }
        outbound
    }
}
