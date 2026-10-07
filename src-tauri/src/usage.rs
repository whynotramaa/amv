use super::Store;
use anyhow::{bail, Result};
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub as_of: i64,
    pub today_start: i64,
    pub week_start: i64,
    pub groups: Vec<UsageGroup>,
    pub groups_truncated: bool,
    pub attempts: Vec<UsageAttempt>,
    pub next: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageGroup {
    pub provider: String,
    pub account_id: Option<String>,
    pub credential_id: Option<String>,
    pub today: UsageTotals,
    pub week: UsageTotals,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    pub attempts: i64,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cached_tokens: Option<i64>,
    pub unknown_token_attempts: i64,
    pub limit_hits: i64,
    pub in_flight: i64,
    pub estimated_cost_micros: Option<i64>,
    pub unknown_cost_attempts: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageAttempt {
    pub id: i64,
    pub request_id: i64,
    pub meeting_id: i64,
    pub provider: String,
    pub account_id: Option<String>,
    pub credential_id: Option<String>,
    pub model: String,
    pub kind: String,
    pub status: String,
    pub created_at: i64,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cached_tokens: Option<i64>,
    pub first_token_ms: Option<i64>,
    pub total_ms: Option<i64>,
    pub error_code: Option<String>,
    pub limit_hit: bool,
    pub cost_micros: Option<i64>,
}

fn known_error_code(code: &str) -> bool {
    crate::inference::safe_code(code).is_some()
}

impl Store {
    pub fn begin_usage(&self, context_id: i64, kind: &str) -> Result<i64> {
        if context_id <= 0 || !matches!(kind, "question" | "transcript") {
            bail!("invalid usage context or kind");
        }
        let tx = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let changed = tx.execute(
            "INSERT INTO usage_events (id, context_id, kind)
             SELECT (SELECT MAX(COALESCE((SELECT MAX(id) FROM usage_events),0),COALESCE((SELECT MAX(usage_id) FROM api_budget_receipts),0))+1), c.id, ?2 FROM request_context c JOIN message_requests r ON r.id=c.request_id
             WHERE c.id=?1 AND r.status='inflight' AND r.kind=?2",
            params![context_id, kind],
        )?;
        if changed != 1 {
            bail!("usage requires an in-flight request of the same kind");
        }
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finish_usage(
        &self,
        id: i64,
        status: &str,
        input: Option<i64>,
        output: Option<i64>,
        cached: Option<i64>,
        first_ms: Option<i64>,
        total_ms: i64,
        error_code: Option<&str>,
        limit_hit: bool,
    ) -> Result<()> {
        if id <= 0
            || !matches!(
                status,
                "ok" | "limited" | "error" | "fell_back" | "partial" | "cancelled" | "interrupted"
            )
            || [input, output, cached]
                .iter()
                .flatten()
                .any(|n| !(0..=1_000_000_000_000).contains(n))
            || !(0..=100_000_000_000_000).contains(&total_ms)
            || first_ms.is_some_and(|n| !(0..=total_ms).contains(&n))
            || matches!((cached, input), (Some(c), Some(i)) if c > i)
            || error_code.is_some_and(|code| !known_error_code(code))
        {
            bail!("invalid usage result");
        }
        let tx = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let changed = tx.execute(
            "UPDATE usage_events SET status=?2,input_tokens=?3,output_tokens=?4,cached_tokens=?5,
             first_token_ms=?6,total_ms=?7,error_code=?8,limit_hit=?9 WHERE id=?1 AND status='in_flight'",
            params![id, status, input, output, cached, first_ms, total_ms, error_code, limit_hit],
        )?;
        if changed != 1 {
            bail!("usage attempt is missing or already terminal");
        }
        let cost = self.finish_api_budget(id, input, output, cached)?;
        tx.execute(
            "UPDATE usage_events SET cost_micros=?2 WHERE id=?1",
            params![id, cost],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn recover_usage(&self) -> Result<usize> {
        Ok(self.conn.execute(
            "UPDATE usage_events SET status='interrupted' WHERE status='in_flight'",
            [],
        )?)
    }

    pub fn usage_report(&self, before_id: Option<i64>) -> Result<UsageReport> {
        if before_id.is_some_and(|id| id <= 0) {
            bail!("invalid usage cursor");
        }
        let (as_of, today_start, week_start): (i64, i64, i64) = self.conn.query_row(
            "SELECT unixepoch(), unixepoch('now','localtime','start of day','utc'),
             unixepoch('now','localtime','-6 days','start of day','utc')",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let mut statement = self.conn.prepare(
            "SELECT u.id,c.request_id,r.meeting_id,c.provider,c.account_id,c.model,u.kind,u.status,
             u.created_at,u.input_tokens,u.output_tokens,u.cached_tokens,u.first_token_ms,u.total_ms,u.error_code,u.limit_hit,c.credential_id,u.cost_micros
             FROM usage_events u JOIN request_context c ON c.id=u.context_id
             JOIN message_requests r ON r.id=c.request_id
             WHERE (?1 IS NULL OR u.id < ?1) ORDER BY u.id DESC LIMIT 21",
        )?;
        let mut attempts = statement
            .query_and_then([before_id], |r| -> Result<UsageAttempt> {
                super::bounded_row_fields(
                    r,
                    &[
                        (3, 10),
                        (4, 43),
                        (5, 200),
                        (6, 10),
                        (7, 16),
                        (14, 100),
                        (16, 43),
                    ],
                )?;
                Ok(UsageAttempt {
                    id: r.get(0)?,
                    request_id: r.get(1)?,
                    meeting_id: r.get(2)?,
                    provider: r.get(3)?,
                    account_id: r.get(4)?,
                    model: r.get(5)?,
                    kind: r.get(6)?,
                    status: r.get(7)?,
                    created_at: r.get(8)?,
                    input_tokens: r.get(9)?,
                    output_tokens: r.get(10)?,
                    cached_tokens: r.get(11)?,
                    first_token_ms: r.get(12)?,
                    total_ms: r.get(13)?,
                    error_code: r.get(14)?,
                    limit_hit: r.get(15)?,
                    credential_id: r.get(16)?,
                    cost_micros: r.get(17)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        for attempt in &attempts {
            if attempt
                .error_code
                .as_deref()
                .is_some_and(|code| !known_error_code(code))
            {
                bail!("invalid stored usage code");
            }
            if let Some(key) = &attempt.credential_id {
                super::validate_account_id(key)?;
            }
            if let Some(account) = &attempt.account_id {
                super::validate_account_id(account)?;
            }
        }
        let more = attempts.len() > 20;
        attempts.truncate(20);
        let next = more.then(|| attempts.last().expect("full usage page").id);
        let mut statement = self.conn.prepare(
            "SELECT c.provider,c.account_id,
             SUM(CASE WHEN u.created_at >= ?1 THEN 1 ELSE 0 END),
             SUM(CASE WHEN u.created_at >= ?1 THEN u.input_tokens END),
             SUM(CASE WHEN u.created_at >= ?1 THEN u.output_tokens END),
             SUM(CASE WHEN u.created_at >= ?1 THEN u.cached_tokens END),
             SUM(CASE WHEN u.created_at >= ?1 AND (u.input_tokens IS NULL OR u.output_tokens IS NULL) THEN 1 ELSE 0 END),
             SUM(CASE WHEN u.created_at >= ?1 THEN u.limit_hit ELSE 0 END),
             SUM(CASE WHEN u.created_at >= ?1 AND u.status='in_flight' THEN 1 ELSE 0 END),
             COUNT(*),SUM(u.input_tokens),SUM(u.output_tokens),SUM(u.cached_tokens),
             SUM(u.input_tokens IS NULL OR u.output_tokens IS NULL),SUM(u.limit_hit),SUM(u.status='in_flight'),
             c.credential_id, SUM(CASE WHEN u.created_at >= ?1 THEN u.cost_micros END),
             SUM(CASE WHEN u.created_at >= ?1 AND c.provider!='chatgpt' AND u.cost_micros IS NULL THEN 1 ELSE 0 END),
             SUM(u.cost_micros),SUM(c.provider!='chatgpt' AND u.cost_micros IS NULL)
             FROM usage_events u JOIN request_context c ON c.id=u.context_id
             WHERE u.created_at >= ?2 AND u.created_at <= ?3
             GROUP BY c.provider,c.account_id,c.credential_id ORDER BY c.provider,c.account_id,c.credential_id LIMIT 101",
        )?;
        let mut groups = statement
            .query_and_then(
                params![today_start, week_start, as_of],
                |r| -> Result<UsageGroup> {
                    super::bounded_row_fields(r, &[(0, 10), (1, 43), (16, 43)])?;
                    let totals = |i| -> rusqlite::Result<UsageTotals> {
                        Ok(UsageTotals {
                            attempts: r.get(i)?,
                            input_tokens: r.get(i + 1)?,
                            output_tokens: r.get(i + 2)?,
                            cached_tokens: r.get(i + 3)?,
                            unknown_token_attempts: r.get(i + 4)?,
                            limit_hits: r.get(i + 5)?,
                            in_flight: r.get(i + 6)?,
                            estimated_cost_micros: r.get(if i == 2 { 17 } else { 19 })?,
                            unknown_cost_attempts: r.get(if i == 2 { 18 } else { 20 })?,
                        })
                    };
                    Ok(UsageGroup {
                        provider: r.get(0)?,
                        account_id: r.get(1)?,
                        credential_id: r.get(16)?,
                        today: totals(2)?,
                        week: totals(9)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>>>()?;
        for group in &groups {
            if let Some(key) = &group.credential_id {
                super::validate_account_id(key)?;
            }
            if let Some(account) = &group.account_id {
                super::validate_account_id(account)?;
            }
        }
        let groups_truncated = groups.len() > 100;
        groups.truncate(100);
        Ok(UsageReport {
            as_of,
            today_start,
            week_start,
            groups,
            groups_truncated,
            attempts,
            next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Settings;

    fn attempt(store: &mut Store, meeting: i64, provider: &str) -> Result<i64> {
        let request = store.prepare_question_request(meeting, "question")?;
        store.begin_request(request.id)?;
        let context = store.save_request_context(
            request.id,
            provider,
            (provider == "chatgpt").then_some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            "model",
            "{}",
            "{}",
            false,
        )?;
        let id = store.begin_usage(context.id, "question")?;
        store.complete_request(request.id, "answer", provider, "model", None)?;
        Ok(id)
    }

    #[test]
    fn usage_records_nullable_totals_paging_recovery_and_cascade() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("usage", 0, &Settings::default())?;
        let known = attempt(&mut store, meeting.id, "chatgpt")?;
        assert!(store
            .finish_usage(
                known,
                "ok",
                Some(10),
                Some(2),
                Some(11),
                None,
                1,
                None,
                false
            )
            .is_err());
        assert!(store
            .finish_usage(known, "ok", None, None, None, Some(2), 1, None, false)
            .is_err());
        assert!(store
            .finish_usage(
                known,
                "ok",
                None,
                None,
                None,
                None,
                1,
                Some("body with secrets"),
                false
            )
            .is_err());
        store.finish_usage(
            known,
            "ok",
            Some(10),
            Some(2),
            Some(3),
            Some(4),
            5,
            None,
            false,
        )?;
        assert!(store
            .finish_usage(known, "error", None, None, None, None, 6, None, false)
            .is_err());
        let unknown = attempt(&mut store, meeting.id, "chatgpt")?;
        store.finish_usage(
            unknown,
            "limited",
            None,
            None,
            None,
            None,
            7,
            Some("rate_limit_exceeded"),
            true,
        )?;
        for _ in 0..20 {
            attempt(&mut store, meeting.id, "gemini")?;
        }
        let report = store.usage_report(None)?;
        assert_eq!(report.attempts.len(), 20);
        let next = report.next.unwrap();
        let old = store.usage_report(Some(next))?;
        assert_eq!(old.attempts.len(), 2);
        assert!(old.next.is_none());
        assert!(old.attempts.iter().all(|a| a.id < next));
        let group = report
            .groups
            .iter()
            .find(|g| g.provider == "chatgpt")
            .unwrap();
        assert_eq!(group.today.attempts, 2);
        assert_eq!(group.today.input_tokens, Some(10));
        assert_eq!(group.today.cached_tokens, Some(3));
        assert_eq!(group.today.unknown_token_attempts, 1);
        assert_eq!(group.today.limit_hits, 1);
        let unknown_group = report
            .groups
            .iter()
            .find(|g| g.provider == "gemini")
            .unwrap();
        assert_eq!(unknown_group.week.input_tokens, None);
        assert_eq!(unknown_group.week.in_flight, 20);
        assert_eq!(store.recover_usage()?, 20);
        assert_eq!(store.recover_usage()?, 0);
        assert!(store.usage_report(Some(0)).is_err());
        store
            .conn
            .execute("UPDATE request_context SET model=?1", ["x".repeat(201)])?;
        assert!(store.usage_report(None).is_err());
        store
            .conn
            .execute("UPDATE request_context SET model='model'", [])?;
        store.conn.execute(
            "UPDATE usage_events SET error_code='unknown_code' WHERE id=?1",
            [known],
        )?;
        assert!(store.usage_report(Some(next)).is_err());
        store.conn.execute(
            "UPDATE usage_events SET error_code=NULL WHERE id=?1",
            [known],
        )?;
        store
            .conn
            .execute("DELETE FROM meetings WHERE id=?1", [meeting.id])?;
        assert!(store.usage_report(None)?.attempts.is_empty());
        Ok(())
    }

    #[test]
    fn usage_upgrade_preserves_old_context_and_reopens_records() -> Result<()> {
        let path =
            std::env::temp_dir().join(format!("harness-usage-{}.sqlite", uuid::Uuid::new_v4()));
        let result = (|| -> Result<()> {
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("upgrade", 0, &Settings::default())?;
            let request = store.prepare_question_request(meeting.id, "question")?;
            store.begin_request(request.id)?;
            let context = store
                .save_request_context(request.id, "chatgpt", None, "old", "{}", "{}", false)?;
            store.conn.execute_batch("DROP TRIGGER api_usage_legacy; DROP TABLE api_legacy_usage; DROP TABLE api_budget_receipts; DROP TABLE api_key_caps; DROP TABLE api_prices; ALTER TABLE request_context DROP COLUMN credential_id; DROP TABLE usage_events; ALTER TABLE request_context DROP COLUMN account_id; ALTER TABLE message_requests DROP COLUMN image_url; PRAGMA user_version=9;")?;
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(
                store.request_context(request.id, None)?.unwrap().id,
                context.id
            );
            assert!(store.usage_report(None)?.attempts.is_empty());
            assert!(store.begin_usage(context.id, "transcript").is_err());
            let usage = store.begin_usage(context.id, "question")?;
            assert!(store.begin_usage(context.id, "question").is_err());
            store.finish_usage(usage, "ok", Some(1), Some(0), Some(0), None, 1, None, false)?;
            drop(store);
            let store = Store::open(&path)?;
            let report = store.usage_report(None)?;
            assert_eq!(report.attempts[0].id, usage);
            assert_eq!(report.attempts[0].account_id, None);
            assert_eq!(report.attempts[0].status, "ok");
            Ok(())
        })();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }
    #[test]
    fn usage_groups_are_bounded_and_calendar_window_excludes_old_and_future_events() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("groups", 0, &Settings::default())?;
        for number in 0..101 {
            let request = store.prepare_question_request(meeting.id, "question")?;
            store.begin_request(request.id)?;
            let context = store.save_request_context(
                request.id,
                "chatgpt",
                Some(&format!("{number:043}")),
                "model",
                "{}",
                "{}",
                false,
            )?;
            store.begin_usage(context.id, "question")?;
            store.complete_request(request.id, "answer", "chatgpt", "model", None)?;
        }
        let report = store.usage_report(None)?;
        assert!(report.groups_truncated);
        assert_eq!(report.groups.len(), 100);
        let oldest = store
            .conn
            .query_row("SELECT MIN(id) FROM usage_events", [], |r| {
                r.get::<_, i64>(0)
            })?;
        store.conn.execute(
            "UPDATE usage_events SET created_at=?1 WHERE id=?2",
            params![report.week_start - 1, oldest],
        )?;
        let report = store.usage_report(None)?;
        assert!(!report.groups_truncated);
        assert_eq!(report.groups.len(), 100);
        store.conn.execute(
            "UPDATE usage_events SET created_at=?1 WHERE id=?2",
            params![report.as_of + 3600, oldest],
        )?;
        assert_eq!(store.usage_report(None)?.groups.len(), 100);
        store.conn.execute(
            "UPDATE usage_events SET created_at=?1 WHERE id=?2",
            params![report.week_start, oldest],
        )?;
        let report = store.usage_report(None)?;
        let oldest_group = report
            .groups
            .iter()
            .find(|g| {
                g.account_id.as_deref() == Some("0000000000000000000000000000000000000000000")
            })
            .unwrap();
        assert_eq!(oldest_group.week.attempts, 1);
        assert_eq!(oldest_group.today.attempts, 0);
        assert_eq!(oldest_group.today.input_tokens, None);
        Ok(())
    }
}
