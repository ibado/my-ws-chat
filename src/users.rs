use crate::types::Result;
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct UserRepo {
    db_pool: SqlitePool,
}

pub struct User {
    pub id: u32,
    pub password_hash: String,
}

impl UserRepo {
    pub fn new(pool: &SqlitePool) -> Self {
        Self {
            db_pool: pool.clone(),
        }
    }

    pub async fn store(&self, nickname: String, password_hash: String) -> Result<()> {
        sqlx::query!(
            "INSERT INTO users (nickname, password_hash) VALUES (?, ?);",
            nickname,
            password_hash,
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| eprintln!("Error inserting user: {e}"))
        .map(|_| ())
    }

    pub async fn get_by_nickname(&self, nickname: &str) -> Result<Option<User>> {
        match sqlx::query!(
            "SELECT id, password_hash FROM users WHERE nickname = ?",
            nickname,
        )
        .fetch_one(&self.db_pool)
        .await
        {
            Ok(r) => Ok(Some(User {
                id: r.id as u32,
                password_hash: r.password_hash,
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(eprintln!("Error fetching user: {e}")),
        }
    }
}
