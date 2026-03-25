use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    AuditBucket, AuditLog, AuditQuery, AuditStats, DiscoveryQuery, DiscoveryResult, Event,
    EventQuery, NewAuditLog, SearchHistoryEntry, SearchSuggestion,
    TradeSearchQuery, TradeSearchResult,
};
use crate::fraud_service::FraudReport;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_event(&self, event: &Event) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO events (id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(event.id)
        .bind(&event.event_type)
        .bind(&event.contract_id)
        .bind(event.ledger)
        .bind(&event.transaction_hash)
        .bind(event.timestamp)
        .bind(&event.data)
        .bind(event.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_events(&self, query: &EventQuery) -> Result<Vec<Event>, AppError> {
        let mut sql = "SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at FROM events WHERE 1=1".to_string();
        let mut bindings = vec![];

        if let Some(event_type) = &query.event_type {
            sql.push_str(&format!(" AND event_type = ${}", bindings.len() + 1));
            bindings.push(event_type.as_str());
        }

        if let Some(trade_id) = query.trade_id {
            sql.push_str(&format!(" AND data->>'trade_id' = ${}", bindings.len() + 1));
            bindings.push(&trade_id.to_string());
        }

        if let Some(from_ledger) = query.from_ledger {
            sql.push_str(&format!(" AND ledger >= ${}", bindings.len() + 1));
            bindings.push(&from_ledger.to_string());
        }

        if let Some(to_ledger) = query.to_ledger {
            sql.push_str(&format!(" AND ledger <= ${}", bindings.len() + 1));
            bindings.push(&to_ledger.to_string());
        }

        sql.push_str(" ORDER BY ledger DESC, timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut query_builder = sqlx::query(&sql);

        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                event_type: row.get("event_type"),
                contract_id: row.get("contract_id"),
                ledger: row.get("ledger"),
                transaction_hash: row.get("transaction_hash"),
                timestamp: row.get("timestamp"),
                data: row.get("data"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }

    pub async fn get_event_by_id(&self, id: Uuid) -> Result<Event, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at
            FROM events WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::EventNotFound)?;

        Ok(Event {
            id: row.get("id"),
            event_type: row.get("event_type"),
            contract_id: row.get("contract_id"),
            ledger: row.get("ledger"),
            transaction_hash: row.get("transaction_hash"),
            timestamp: row.get("timestamp"),
            data: row.get("data"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_latest_ledger(&self, contract_id: &str) -> Result<Option<i64>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT MAX(ledger) as latest_ledger FROM events WHERE contract_id = $1
            "#,
        )
        .bind(contract_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.get("latest_ledger")))
    }

    pub async fn count_events(&self, query: &EventQuery) -> Result<i64, AppError> {
        let mut sql = "SELECT COUNT(*) FROM events WHERE 1=1".to_string();
        let mut bindings: Vec<String> = vec![];

        if let Some(event_type) = &query.event_type {
            sql.push_str(&format!(" AND event_type = ${}", bindings.len() + 1));
            bindings.push(event_type.clone());
        }
        if let Some(trade_id) = query.trade_id {
            sql.push_str(&format!(" AND data->>'trade_id' = ${}", bindings.len() + 1));
            bindings.push(trade_id.to_string());
        }
        if let Some(from_ledger) = query.from_ledger {
            sql.push_str(&format!(" AND ledger >= ${}", bindings.len() + 1));
            bindings.push(from_ledger.to_string());
        }
        if let Some(to_ledger) = query.to_ledger {
            sql.push_str(&format!(" AND ledger <= ${}", bindings.len() + 1));
            bindings.push(to_ledger.to_string());
        }

        let mut q = sqlx::query(&sql);
        for b in &bindings { q = q.bind(b); }
        let row = q.fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>(0))
    }

    pub async fn get_events_in_range(&self, from_ledger: i64, to_ledger: i64, contract_id: &str) -> Result<Vec<Event>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at
            FROM events
            WHERE contract_id = $1 AND ledger >= $2 AND ledger <= $3
            ORDER BY ledger ASC, timestamp ASC
            "#,
        )
        .bind(contract_id)
        .bind(from_ledger)
        .bind(to_ledger)
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                event_type: row.get("event_type"),
                contract_id: row.get("contract_id"),
                ledger: row.get("ledger"),
                transaction_hash: row.get("transaction_hash"),
                timestamp: row.get("timestamp"),
                data: row.get("data"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }
}

    /// Total number of indexed events, optionally filtered by contract.
    pub async fn get_event_count(&self, contract_id: Option<&str>) -> Result<i64, AppError> {
        let count: i64 = if let Some(cid) = contract_id {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_id = $1")
                .bind(cid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM events")
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count)
    }

    /// Event counts grouped by event_type — used for stats/dashboard.
    pub async fn get_event_type_counts(&self) -> Result<Vec<(String, i64)>, AppError> {
        let rows = sqlx::query("SELECT event_type, COUNT(*) as cnt FROM events GROUP BY event_type ORDER BY cnt DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("event_type"), r.get::<i64, _>("cnt")))
            .collect())
    }

    /// Latest indexed ledger and its timestamp across all contracts.
    pub async fn get_latest_ledger_global(&self) -> Result<Option<(i64, DateTime<Utc>)>, AppError> {
        let row = sqlx::query(
            "SELECT ledger, timestamp FROM events ORDER BY ledger DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| (r.get::<i64, _>("ledger"), r.get::<DateTime<Utc>, _>("timestamp"))))
    }
}

    pub async fn record_search(&self, query_text: &str, search_type: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO search_history (query_text, search_type)
            VALUES ($1, $2)
            "#,
        )
        .bind(query_text)
        .bind(search_type)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn search_trades(&self, query: &TradeSearchQuery) -> Result<Vec<TradeSearchResult>, AppError> {
        let limit = query.limit.unwrap_or(25).clamp(1, 100);
        let offset = query.offset.unwrap_or(0).max(0);
        let q = query.q.clone().unwrap_or_default();
        let q_pattern = format!("%{}%", q);

        let rows = sqlx::query_as::<_, TradeSearchResult>(
            r#"
            WITH latest_trade_events AS (
                SELECT DISTINCT ON ((data->>'trade_id'))
                    (data->>'trade_id')::BIGINT AS trade_id,
                    event_type
                FROM events
                WHERE data->>'trade_id' IS NOT NULL
                ORDER BY (data->>'trade_id'), ledger DESC, timestamp DESC
            ),
            trade_base AS (
                SELECT
                    (e.data->>'trade_id')::BIGINT AS trade_id,
                    e.data->>'seller' AS seller,
                    e.data->>'buyer' AS buyer,
                    (e.data->>'amount')::BIGINT AS amount,
                    e.timestamp AS created_at
                FROM events e
                WHERE e.event_type = 'trade_created'
            )
            SELECT
                tb.trade_id,
                tb.seller,
                tb.buyer,
                tb.amount,
                lte.event_type AS status,
                tb.created_at
            FROM trade_base tb
            JOIN latest_trade_events lte ON lte.trade_id = tb.trade_id
            WHERE
                ($1 = '' OR tb.trade_id::TEXT ILIKE $2 OR tb.seller ILIKE $2 OR tb.buyer ILIKE $2)
                AND ($3::TEXT IS NULL OR lte.event_type = $3)
                AND ($4::TEXT IS NULL OR tb.seller = $4)
                AND ($5::TEXT IS NULL OR tb.buyer = $5)
                AND ($6::BIGINT IS NULL OR tb.amount >= $6)
                AND ($7::BIGINT IS NULL OR tb.amount <= $7)
            ORDER BY tb.created_at DESC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(q.as_str())
        .bind(q_pattern.as_str())
        .bind(query.status.as_deref())
        .bind(query.seller.as_deref())
        .bind(query.buyer.as_deref())
        .bind(query.min_amount.map(|v| v as i64))
        .bind(query.max_amount.map(|v| v as i64))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn discover_entities(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveryResult>, AppError> {
        let limit = query.limit.unwrap_or(25).clamp(1, 100);
        let q = query.q.clone().unwrap_or_default();
        let q_pattern = format!("%{}%", q);

        let rows = sqlx::query_as::<_, DiscoveryResult>(
            r#"
            WITH entities AS (
                SELECT data->>'seller' AS address, 'user' AS role, timestamp
                FROM events
                WHERE event_type = 'trade_created' AND data->>'seller' IS NOT NULL
                UNION ALL
                SELECT data->>'buyer' AS address, 'user' AS role, timestamp
                FROM events
                WHERE event_type = 'trade_created' AND data->>'buyer' IS NOT NULL
                UNION ALL
                SELECT data->>'arbitrator' AS address, 'arbitrator' AS role, timestamp
                FROM events
                WHERE event_type = 'arb_reg' AND data->>'arbitrator' IS NOT NULL
            )
            SELECT
                address,
                role,
                COUNT(*)::BIGINT AS seen_count,
                MAX(timestamp) AS last_seen
            FROM entities
            WHERE
                ($1 = '' OR address ILIKE $2)
                AND ($3::TEXT IS NULL OR role = $3)
            GROUP BY address, role
            ORDER BY seen_count DESC, last_seen DESC
            LIMIT $4
            "#,
        )
        .bind(q.as_str())
        .bind(q_pattern.as_str())
        .bind(query.role.as_deref())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_search_suggestions(
        &self,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchSuggestion>, AppError> {
        let q_pattern = format!("{}%", prefix);
        let rows = sqlx::query_as::<_, SearchSuggestion>(
            r#"
            SELECT
                query_text AS term,
                COUNT(*)::BIGINT AS hits
            FROM search_history
            WHERE query_text ILIKE $1
            GROUP BY query_text
            ORDER BY hits DESC, term ASC
            LIMIT $2
            "#,
        )
        .bind(q_pattern)
        .bind(limit.clamp(1, 20))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_search_history(&self, limit: i64) -> Result<Vec<SearchHistoryEntry>, AppError> {
        let rows = sqlx::query_as::<_, SearchHistoryEntry>(
            r#"
            SELECT id, query_text, search_type, created_at
            FROM search_history
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // =========================================================================
    // Audit Log Operations
    // =========================================================================

    /// Insert a new audit log entry.
    pub async fn insert_audit_log(&self, entry: &NewAuditLog) -> Result<AuditLog, AppError> {
        let row = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs
                (actor, category, action, resource_type, resource_id,
                 outcome, ledger, tx_hash, metadata, severity)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&entry.actor)
        .bind(entry.category.as_str())
        .bind(&entry.action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(entry.outcome.as_str())
        .bind(entry.ledger)
        .bind(&entry.tx_hash)
        .bind(&entry.metadata)
        .bind(entry.severity.as_str())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    /// Query audit logs with optional filters and pagination.
    pub async fn query_audit_logs(&self, q: &AuditQuery) -> Result<Vec<AuditLog>, AppError> {
        let limit = q.limit.unwrap_or(50).clamp(1, 500);
        let offset = q.offset.unwrap_or(0).max(0);

        // Build dynamic WHERE clauses
        let mut conditions = vec!["1=1".to_string()];
        let mut idx = 1usize;

        macro_rules! push_cond {
            ($field:expr, $val:expr) => {
                if $val.is_some() {
                    conditions.push(format!("{} = ${}", $field, idx));
                    idx += 1;
                }
            };
        }

        push_cond!("actor",         q.actor);
        push_cond!("category",      q.category);
        push_cond!("action",        q.action);
        push_cond!("resource_type", q.resource_type);
        push_cond!("resource_id",   q.resource_id);
        push_cond!("outcome",       q.outcome);
        push_cond!("severity",      q.severity);

        if q.from.is_some() {
            conditions.push(format!("created_at >= ${}", idx));
            idx += 1;
        }
        if q.to.is_some() {
            conditions.push(format!("created_at <= ${}", idx));
            idx += 1;
        }

        let sql = format!(
            "SELECT * FROM audit_logs WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            conditions.join(" AND "),
            limit,
            offset
        );

        let mut qb = sqlx::query_as::<_, AuditLog>(&sql);
        if let Some(v) = &q.actor         { qb = qb.bind(v); }
        if let Some(v) = &q.category      { qb = qb.bind(v); }
        if let Some(v) = &q.action        { qb = qb.bind(v); }
        if let Some(v) = &q.resource_type { qb = qb.bind(v); }
        if let Some(v) = &q.resource_id   { qb = qb.bind(v); }
        if let Some(v) = &q.outcome       { qb = qb.bind(v); }
        if let Some(v) = &q.severity      { qb = qb.bind(v); }
        if let Some(v) = q.from           { qb = qb.bind(v); }
        if let Some(v) = q.to             { qb = qb.bind(v); }

        Ok(qb.fetch_all(&self.pool).await?)
    }

    /// Count audit logs matching the same filters (for pagination).
    pub async fn count_audit_logs(&self, q: &AuditQuery) -> Result<i64, AppError> {
        let mut conditions = vec!["1=1".to_string()];
        let mut idx = 1usize;

        macro_rules! push_cond {
            ($field:expr, $val:expr) => {
                if $val.is_some() {
                    conditions.push(format!("{} = ${}", $field, idx));
                    idx += 1;
                }
            };
        }

        push_cond!("actor",         q.actor);
        push_cond!("category",      q.category);
        push_cond!("action",        q.action);
        push_cond!("resource_type", q.resource_type);
        push_cond!("resource_id",   q.resource_id);
        push_cond!("outcome",       q.outcome);
        push_cond!("severity",      q.severity);

        if q.from.is_some() {
            conditions.push(format!("created_at >= ${}", idx));
            idx += 1;
        }
        if q.to.is_some() {
            conditions.push(format!("created_at <= ${}", idx));
            idx += 1;
        }

        let sql = format!(
            "SELECT COUNT(*) FROM audit_logs WHERE {}",
            conditions.join(" AND ")
        );

        let mut qb = sqlx::query(&sql);
        if let Some(v) = &q.actor         { qb = qb.bind(v); }
        if let Some(v) = &q.category      { qb = qb.bind(v); }
        if let Some(v) = &q.action        { qb = qb.bind(v); }
        if let Some(v) = &q.resource_type { qb = qb.bind(v); }
        if let Some(v) = &q.resource_id   { qb = qb.bind(v); }
        if let Some(v) = &q.outcome       { qb = qb.bind(v); }
        if let Some(v) = &q.severity      { qb = qb.bind(v); }
        if let Some(v) = q.from           { qb = qb.bind(v); }
        if let Some(v) = q.to             { qb = qb.bind(v); }

        let row = qb.fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Aggregate statistics for the analysis dashboard.
    pub async fn audit_stats(&self) -> Result<AuditStats, AppError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await?;

        let by_category = sqlx::query_as::<_, AuditBucket>(
            "SELECT category AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY category ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let by_outcome = sqlx::query_as::<_, AuditBucket>(
            "SELECT outcome AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY outcome ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let by_severity = sqlx::query_as::<_, AuditBucket>(
            "SELECT severity AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY severity ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let top_actors = sqlx::query_as::<_, AuditBucket>(
            "SELECT actor AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY actor ORDER BY count DESC LIMIT 20",
    pub async fn insert_fraud_alert(&self, report: &FraudReport) -> Result<(), AppError> {
        let rules_json = serde_json::to_value(&report.rules_triggered).unwrap_or(serde_json::Value::Null);
        let status = if report.risk_score >= 80 { "pending" } else { "approved" };

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO fraud_alerts (trade_id, risk_score, rules_triggered, ml_score)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(report.trade_id as i64)
        .bind(report.risk_score)
        .bind(&rules_json)
        .bind(report.ml_result.score as f64)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO fraud_reviews (trade_id, status)
            VALUES ($1, $2)
            ON CONFLICT (trade_id) DO NOTHING
            "#
        )
        .bind(report.trade_id as i64)
        .bind(status)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_fraud_alerts(&self) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT a.id, a.trade_id, a.risk_score, a.rules_triggered, a.ml_score, a.created_at,
                   r.status, r.reviewer, r.review_notes, r.updated_at
            FROM fraud_alerts a
            LEFT JOIN fraud_reviews r ON a.trade_id = r.trade_id
            ORDER BY a.risk_score DESC, a.created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let top_actions = sqlx::query_as::<_, AuditBucket>(
            "SELECT action AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY action ORDER BY count DESC LIMIT 20",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AuditStats { total, by_category, by_outcome, by_severity, top_actors, top_actions })
    }

    /// Delete audit logs older than `days` days. Returns the number of rows deleted.
    pub async fn purge_old_audit_logs(&self, days: i64) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM audit_logs WHERE created_at < NOW() - ($1 || ' days')::INTERVAL",
        )
        .bind(days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(serde_json::json!({
                "id": row.try_get::<uuid::Uuid, _>("id").ok(),
                "trade_id": row.get::<i64, _>("trade_id"),
                "risk_score": row.get::<i32, _>("risk_score"),
                "rules_triggered": row.get::<serde_json::Value, _>("rules_triggered"),
                "ml_score": row.try_get::<f64, _>("ml_score").ok(),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "status": row.try_get::<String, _>("status").unwrap_or_else(|_| "pending".to_string()),
                "reviewer": row.try_get::<String, _>("reviewer").ok(),
                "review_notes": row.try_get::<String, _>("review_notes").ok(),
                "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok(),
            }));
        }

        Ok(alerts)
    }

    pub async fn update_fraud_review(
        &self,
        trade_id: u64,
        status: &str,
        reviewer: &str,
        notes: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO fraud_reviews (trade_id, status, reviewer, review_notes)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (trade_id) DO UPDATE SET
                status = EXCLUDED.status,
                reviewer = EXCLUDED.reviewer,
                review_notes = EXCLUDED.review_notes,
                updated_at = NOW()
            "#
        )
        .bind(trade_id as i64)
        .bind(status)
        .bind(reviewer)
        .bind(notes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
