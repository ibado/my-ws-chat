use crate::MyMessage;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct MessageRepo {
    db_pool: PgPool,
}

impl MessageRepo {
    pub async fn new() -> Option<Self> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var is missing!");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .ok()?;
        Some(Self { db_pool: pool })
    }

    pub async fn store_msg(&self, msg: MyMessage, sender: String, addressee: String) -> Option<()> {
        if let MyMessage::Msg { msg: payload } = msg {
            sqlx::query!(
                "INSERT INTO messages (payload, sender_id, addressee_id) VALUES ($1, $2, $3);",
                payload,
                sender,
                addressee,
            )
            .fetch_one(&self.db_pool)
            .await;
            Some(())
        } else {
            None
        }
    }

    pub async fn get_messages(&self, sender: String, addressee: String) -> Vec<MyMessage> {
        sqlx::query!(
            "SELECT * FROM messages WHERE sender_id = $1 AND addressee_id = $2;",
            sender,
            addressee,
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .iter()
        .map(|r| MyMessage::Msg {
            msg: r.payload.clone(),
        })
        .collect()
    }
}
