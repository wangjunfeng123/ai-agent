use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::session::model::Session;

// 管理session
#[async_trait::async_trait]
pub trait SessionManager: Send + Sync {
    async fn create(&self, session_id: &str, user_id: Option<&str>) -> anyhow::Result<Session>;

    async fn get(&self, session_id: &str) -> anyhow::Result<Option<Session>>;

    async fn save(&self, session: Session) -> anyhow::Result<()>;

    async fn get_or_create(
        &self,
        session_id: &str,
        user_id: Option<&str>,
    ) -> anyhow::Result<Session>;
}

pub struct ConsistencySessionManager {
    pub sessions: Mutex<HashMap<String, Session>>,
}

impl ConsistencySessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SessionManager for ConsistencySessionManager {
    async fn create(&self, session_id: &str, user_id: Option<&str>) -> anyhow::Result<Session> {
        let mut guard = self.sessions.lock().await;
        if guard.contains_key(session_id) {
            anyhow::bail!("session already exist:{session_id}");
        }
        let session = Session::new(session_id.to_owned(), user_id.map(str::to_owned));
        guard.insert(session_id.to_owned(), session.clone());
        Ok(session)
    }

    async fn get(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let guard = self.sessions.lock().await;
        Ok(guard.get(session_id).cloned())
    }

    async fn save(&self, session: Session) -> anyhow::Result<()> {
        let mut guard = self.sessions.lock().await;
        guard.insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn get_or_create(
        &self,
        session_id: &str,
        user_id: Option<&str>,
    ) -> anyhow::Result<Session> {
        let mut guard = self.sessions.lock().await;
        let session = guard
            .entry(session_id.to_owned())
            .or_insert_with(|| Session::new(session_id.to_owned(), user_id.map(str::to_string)))
            .clone();
        Ok(session)
    }
}
