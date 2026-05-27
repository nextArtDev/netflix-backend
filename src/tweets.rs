use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Tweet {
    pub id: Uuid,
    pub author_id: String,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar_url: String,
}
#[derive(Debug, Deserialize)]
pub struct CreateTweet {
    pub author_id: Uuid,
    pub content: String,
}
pub async fn get_all(pool: &PgPool) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        SELECT 
            tweets.id, 
            tweets.author_id, 
            tweets.content, 
            tweets.created_at,
            users.username AS author_username,
            users.display_name AS author_display_name,
            users.avatar_url AS author_avatar_url
        FROM tweets
        JOIN users ON tweets.author_id = users.id
        ORDER BY tweets.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}
pub async fn get_by_username(pool: &PgPool, username: &str) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        SELECT 
            tweets.id, 
            tweets.author_id, 
            tweets.content, 
            tweets.created_at,
            users.username AS author_username,
            users.display_name AS author_display_name,
            users.avatar_url AS author_avatar_url
        FROM tweets
        JOIN users ON tweets.author_id = users.id
        WHERE users.username = $1
        ORDER BY tweets.created_at DESC
        "#,
        username
    )
    .fetch_all(pool)
    .await
}
pub async fn get_feed(pool: &PgPool, user_id: Uuid) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        SELECT 
            tweets.id, 
            tweets.author_id, 
            tweets.content, 
            tweets.created_at,
            users.username AS author_username,
            users.display_name AS author_display_name,
            users.avatar_url AS author_avatar_url
        FROM tweets
        JOIN users ON tweets.author_id = users.id
        WHERE tweets.author_id IN (
            SELECT followed_id FROM follows WHERE follower_id = $1
        )
        ORDER BY tweets.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}
pub async fn create(pool: &PgPool, input: CreateTweet) -> Result<Tweet, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        WITH inserted AS (
            INSERT INTO tweets (author_id, content) 
            VALUES ($1, $2) 
            RETURNING *
        )
        SELECT 
            inserted.id, 
            inserted.author_id, 
            inserted.content, 
            inserted.created_at,
            users.username AS author_username,
            users.display_name AS author_display_name,
            users.avatar_url AS author_avatar_url
        FROM inserted
        JOIN users ON inserted.author_id = users.id
        "#,
        input.author_id,
        input.content
    )
    .fetch_one(pool)
    .await
}
pub async fn delete(pool: &PgPool, tweet_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM tweets WHERE id = $1 RETURNING id", tweet_id).fetch_optional(pool)
        // .execute(pool)
        .await?;
    Ok(())
}