use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct MessageRepo {
    db_pool: SqlitePool,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub payload: String,
    pub is_sender: bool,
}

impl MessageRepo {
    pub fn new(pool: &SqlitePool) -> Self {
        Self {
            db_pool: pool.clone(),
        }
    }

    pub async fn store_msg(&self, payload: String, sender: u32, addressee: u32) -> Option<()> {
        sqlx::query!(
            "INSERT INTO messages (payload, sender_id, addressee_id, timestamp) VALUES (?, ?, ?, datetime('now'));",
            payload,
            sender,
            addressee,
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|_| ())
        .ok()
    }

    pub async fn get_messages(&self, sender: u32, addressee: u32) -> Vec<Message> {
        sqlx::query!(
            r#"
            SELECT payload, sender_id as author FROM messages
            WHERE sender_id = $1 AND addressee_id = $2 OR sender_id = $2 AND addressee_id = $1
            ORDER BY datetime(timestamp);
            "#,
            sender,
            addressee,
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .iter()
        .map(|r| Message {
            payload: r.payload.clone(),
            is_sender: r.author == (sender as i64),
        })
        .collect()
    }
}
