use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde_json::Value;
use std::env;
use tokio::sync::OnceCell;

const TTL_SECS: u64 = 300; // 5 minutos

static REDIS: OnceCell<ConnectionManager> = OnceCell::const_new();

pub async fn connection() -> ConnectionManager {
    REDIS
        .get_or_init(|| async {
            let url = env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
            let client = Client::open(url).expect("failed to create Redis client");
            ConnectionManager::new(client)
                .await
                .expect("failed to connect Redis ConnectionManager")
        })
        .await
        .clone()
}

pub async fn get(key: &str) -> Option<Value> {
    let mut conn = connection().await;
    let result: Option<String> = conn.get(key).await.ok()?;
    result.and_then(|s| serde_json::from_str(&s).ok())
}

pub async fn set(key: &str, value: &Value) {
    set_with_ttl(key, value, TTL_SECS).await;
}

pub async fn set_with_ttl(key: &str, value: &Value, ttl: u64) {
    let mut conn = connection().await;
    if let Ok(serialized) = serde_json::to_string(value) {
        let _: Result<(), _> = conn.set_ex(key, serialized, ttl).await;
    }
}

#[allow(dead_code)]
pub async fn del(key: &str) {
    let mut conn = connection().await;
    let _: Result<(), _> = conn.del(key).await;
}

/// Retorna true se a conexão com o Redis está ativa.
pub async fn ping() -> bool {
    let url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
    let Ok(client) = Client::open(url) else {
        return false;
    };
    match ConnectionManager::new(client).await {
        Ok(mut conn) => redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map(|r| r == "PONG")
            .unwrap_or(false),
        Err(_) => false,
    }
}
pub async fn del_pattern(pattern: &str) {
    let mut conn = connection().await;
    let mut cursor: u64 = 0;
    loop {
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(100u64)
            .query_async(&mut conn)
            .await
            .unwrap_or((0, vec![]));

        for key in keys {
            let _: Result<(), _> = conn.del(&key).await;
        }

        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
    }
}
