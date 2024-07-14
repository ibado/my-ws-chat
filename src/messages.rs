use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct MessageRepo {
    db_pool: SqlitePool,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: u32,
    pub payload: String,
    pub is_sender: bool,
    pub sender: Option<String>,
}

impl MessageRepo {
    pub fn new(pool: &SqlitePool) -> Self {
        Self {
            db_pool: pool.clone(),
        }
    }

    pub async fn message_received(&self, message_id: u32) {
        let _ = sqlx::query!("UPDATE messages SET received = 1 WHERE id = ?", message_id)
            .fetch_one(&self.db_pool)
            .await;
    }

    pub async fn store_msg(&self, payload: String, sender: u32, addressee: u32) -> Option<u32> {
        sqlx::query!(
            r#"
            INSERT INTO messages (payload, sender_id, addressee_id, timestamp)
            VALUES (?, ?, ?, datetime('now')) RETURNING id;
            "#,
            payload,
            sender,
            addressee,
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|r| r.id as u32)
        .ok()
    }

    pub async fn get_unreceived_msgs(&self, addressee: u32) -> Vec<Message> {
        sqlx::query!(
            r#"
            SELECT m.id, m.payload, u.nickname FROM messages m
            INNER JOIN users u ON u.id = m.sender_id
            WHERE addressee_id = ? AND received = 0
            ORDER BY datetime(timestamp);
            "#,
            addressee,
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .iter()
        .map(|r| Message {
            id: r.id as u32,
            payload: r.payload.clone(),
            is_sender: true,
            sender: Some(r.nickname.clone()),
        })
        .collect()
    }

    pub async fn get_messages(&self, sender: u32, addressee: u32) -> Vec<Message> {
        sqlx::query!(
            r#"
            SELECT id, payload, sender_id as author FROM messages
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
            id: r.id as u32,
            payload: r.payload.clone(),
            is_sender: r.author == (sender as i64),
            sender: None,
        })
        .collect()
    }
}
