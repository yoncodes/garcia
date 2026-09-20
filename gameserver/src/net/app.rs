use std::{collections::HashMap, sync::Mutex};

use configs::GameTables;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

use super::gm_command::GmCommandEnvelope;

pub(crate) struct AppState {
    pub db: SqlitePool,
    pub tables: GameTables,
    sessions: Mutex<HashMap<i64, mpsc::UnboundedSender<GmCommandEnvelope>>>,
}

impl AppState {
    pub fn new(db: SqlitePool, tables: GameTables) -> Self {
        Self {
            db,
            tables,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_session(&self, uid: i64, sender: mpsc::UnboundedSender<GmCommandEnvelope>) {
        self.sessions.lock().unwrap().insert(uid, sender);
    }

    pub fn unregister_session(&self, uid: i64, sender: &mpsc::UnboundedSender<GmCommandEnvelope>) {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions
            .get(&uid)
            .is_some_and(|current| current.same_channel(sender))
        {
            sessions.remove(&uid);
        }
    }

    pub fn session(&self, uid: i64) -> Option<mpsc::UnboundedSender<GmCommandEnvelope>> {
        self.sessions.lock().unwrap().get(&uid).cloned()
    }

    pub fn online_players(&self) -> Vec<i64> {
        let mut players = self
            .sessions
            .lock()
            .unwrap()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        players.sort_unstable();
        players
    }
}
