use ac_aln_rt::{errors::AlnError, session::Session};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

pub struct SessionStore {
    redis: ConnectionManager,
}

impl SessionStore {
    pub async fn new(redis_url: &str) -> Result<Self, AlnError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AlnError::Redis(e.to_string()))?;
        let conn = client
            .get_tokio_connection_manager()
            .await
            .map_err(|e| AlnError::Redis(e.to_string()))?;
        Ok(Self { redis: conn })
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<Session>, AlnError> {
        let raw: Option<String> = self
            .redis
            .get(key)
            .await
            .map_err(|e| AlnError::Redis(e.to_string()))?;
        if let Some(json) = raw {
            let session: Session = serde_json::from_str(&json)
                .map_err(|e| AlnError::Redis(e.to_string()))?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn set(&mut self, key: &str, session: &Session) -> Result<(), AlnError> {
        let json = serde_json::to_string(session).map_err(|e| AlnError::Redis(e.to_string()))?;
        self.redis
            .set(key, json)
            .await
            .map_err(|e| AlnError::Redis(e.to_string()))
    }
}
