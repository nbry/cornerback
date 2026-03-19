use anyhow::Result;
use async_trait::async_trait;
use cornerback_core::store::EventStore;
use cornerback_core::{models::*, store::Paging};
use sqlx::{PgPool, Row};

pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn insert_event(&self, event: NewEvent) -> Result<Event> {
        let rec = sqlx::query(
            r#"
            INSERT INTO events (id, webhook_id, headers, body)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING id, webhook_id, headers, body, created_at
            "#,
        )
        .bind(event.webhook_id)
        .bind(event.headers)
        .bind(event.body)
        .fetch_one(&self.pool)
        .await?;

        Ok(Event {
            id: rec.try_get("id")?,
            webhook_id: rec.try_get("webhook_id")?,
            headers: rec.try_get("headers")?,
            body: rec.try_get("body")?,
            created_at: rec.try_get("created_at")?,
        })
    }

    async fn get_event(&self, id: EventId) -> Result<Option<Event>> {
        let rec = sqlx::query(
            r#"
            SELECT id, webhook_id, headers, body, created_at
            FROM events WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(row) => Ok(Some(Event {
                id: row.try_get("id")?,
                webhook_id: row.try_get("webhook_id")?,
                headers: row.try_get("headers")?,
                body: row.try_get("body")?,
                created_at: row.try_get("created_at")?,
            })),
            None => Ok(None),
        }
    }

    async fn list_events(
        &self,
        webhook_id: &str,
        Paging { limit, offset }: Paging,
    ) -> Result<Vec<Event>> {
        let rows = sqlx::query(
            r#"
            SELECT id, webhook_id, headers, body, created_at
            FROM events            WHERE webhook_id = $1
            ORDER BY created_at DESC
            LIMIT COALESCE($2, 100)
            OFFSET COALESCE($3, 0)
            "#,
        )
        .bind(webhook_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let recs = rows
            .into_iter()
            .map(|row| -> std::result::Result<Event, sqlx::Error> {
                Ok(Event {
                    id: row.try_get("id")?,
                    webhook_id: row.try_get("webhook_id")?,
                    headers: row.try_get("headers")?,
                    body: row.try_get("body")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(recs)
    }
}
