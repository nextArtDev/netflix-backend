use serde::{Deserialize, Serialize};
use sqlx::PgPool;
 
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ToggleFollow {
    pub follower_id: Uuid,
    pub followed_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct FollowResult {
    pub is_following: bool,
   
}

pub async fn toggle(pool:&PgPool, input:ToggleFollow)-> Result<bool,sqlx::Error>{
    sqlx::query_scalar!(
        r#"
        WITH deleted AS(
            DELETE FROM follows
            WHERE follower_id = $1 AND followed_id = $2
            RETURNING *
        ),
        inserted AS(
            INSERT INTO follows (follower_id, followed_id)
            SELECT $1, $2
            WHERE NOT EXISTS (SELECT 1 FROM deleted)
            RETURNING 1
        )
        SELECT EXISTS (SELECT 1 FROM inserted) AS "is_following!"
         "#,
        input.follower_id,
        input.followed_id
    )
    .fetch_one(pool)
    .await
}