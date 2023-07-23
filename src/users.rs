use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct UserRepo {
    db_pool: PgPool,
}

pub struct User {
    pub id: u32,
    pub password_hash: String,
}

impl UserRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self {
            db_pool: pool.clone(),
        }
    }

    pub async fn store(&self, nickname: String, password_hash: String) -> Option<()> {
        sqlx::query!(
            "INSERT INTO users (nickname, password_hash) VALUES ($1, $2);",
            nickname,
            password_hash,
        )
        .execute(&self.db_pool)
        .await
        .ok()
        .map(|_| ())
    }

    pub async fn get_by_nickname(&self, nickname: &str) -> Option<User> {
        sqlx::query!(
            "SELECT id, password_hash FROM users WHERE nickname = $1",
            nickname,
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|r| User {
            id: r.id as u32,
            password_hash: r.password_hash,
        })
        .ok()
    }
}
