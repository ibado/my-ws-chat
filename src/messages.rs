use crate::MyMessage;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct MessageRepo {
    db_pool: PgPool,
}

impl MessageRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { db_pool: pool.clone() }
    }

    pub async fn store_msg(&self, msg: MyMessage, sender: u32, addressee: u32) -> Option<()> {
        if let MyMessage::Msg { msg: payload, .. } = msg {
            sqlx::query!(
                "INSERT INTO messages (payload, sender_id, addressee_id) VALUES ($1, $2, $3);",
                payload,
                sender as i32,
                addressee as i32,
            )
            .fetch_one(&self.db_pool)
            .await
            .map(|_| ())
            .ok()
        } else {
            None
        }
    }

    pub async fn get_messages(
        &self,
        sender: u32,
        addressee: u32,
    ) -> Vec<(MyMessage, PrimitiveDateTime)> {
        sqlx::query!(
            r#"
            SELECT payload, sender_id as author, timestamp FROM messages
            WHERE sender_id = $1 AND addressee_id = $2 OR sender_id = $2 AND addressee_id = $1
            ORDER BY timestamp;
            "#,
            sender as i32,
            addressee as i32,
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .iter()
        .map(|r| {
            (
                MyMessage::Msg {
                    msg: r.payload.clone(),
                    author_id: r.author as u32,
                },
                r.timestamp,
            )
        })
        .collect()
    }
}
