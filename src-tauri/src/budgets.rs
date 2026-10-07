use super::Store;
use anyhow::{bail, Result};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApiPrice {
    pub input_micros_per_million: i64,
    pub output_micros_per_million: i64,
    pub cached_micros_per_million: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiBudgetInput {
    pub provider: String,
    pub key_id: String,
    pub model: Option<String>,
    pub token_cap: Option<i64>,
    pub cost_cap_micros: Option<i64>,
    pub price: Option<ApiPrice>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiBudgetStatus {
    pub token_cap: Option<i64>,
    pub cost_cap_micros: Option<i64>,
    pub price: Option<ApiPrice>,
    pub today_tokens: i64,
    pub today_cost_micros: Option<i64>,
    pub attempts: i64,
    pub unknown_tokens: i64,
    pub unknown_costs: i64,
    pub pending: i64,
    pub reason: Option<String>,
    pub reset_at: i64,
}

#[derive(Debug)]
pub enum ApiAdmission {
    Allowed(i64),
    Blocked(ApiBudgetStatus),
}

fn validate_identity(provider: &str, key_id: &str, model: Option<&str>) -> Result<()> {
    if !matches!(provider, "gemini" | "deepseek")
        || key_id.len() != 43
        || !key_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        || model.is_some_and(|m| m.is_empty() || m.len() > 200 || m.chars().any(char::is_control))
    {
        bail!("invalid API budget identity");
    }
    Ok(())
}

fn prune(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM api_legacy_usage WHERE day_start < unixepoch('now','localtime','-7 days','start of day','utc')", [])?;
    conn.execute("DELETE FROM api_budget_receipts WHERE status != 'pending' AND day_start < unixepoch('now','localtime','-7 days','start of day','utc')", [])?;
    Ok(())
}

fn budget(
    conn: &Connection,
    provider: &str,
    key_id: &str,
    model: Option<&str>,
) -> Result<ApiBudgetStatus> {
    validate_identity(provider, key_id, model)?;
    let (day, reset_at): (i64, i64) = conn.query_row(
        "SELECT unixepoch('now','localtime','start of day','utc'),unixepoch('now','localtime','start of day','+1 day','utc')", [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (token_cap, cost_cap_micros) = conn
        .query_row(
            "SELECT token_cap,cost_cap_micros FROM api_key_caps WHERE key_id=?1 AND provider=?2",
            params![key_id, provider],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));
    let price = conn.query_row(
        "SELECT input_micros,output_micros,cached_micros FROM api_prices WHERE provider=?1 AND model=?2", params![provider, model],
        |r| Ok(ApiPrice { input_micros_per_million:r.get(0)?, output_micros_per_million:r.get(1)?, cached_micros_per_million:r.get(2)? }),
    ).optional()?;
    let (attempts, today_tokens, cost, unknown_tokens, unknown_costs, pending): (i64,i64,Option<i64>,i64,i64,i64) = conn.query_row(
        "SELECT COUNT(*),COALESCE(SUM(COALESCE(input_tokens,0)+COALESCE(output_tokens,0)),0),SUM(cost_micros),
         COALESCE(SUM(status!='pending' AND (input_tokens IS NULL OR output_tokens IS NULL)),0),
         COALESCE(SUM(status!='pending' AND cost_micros IS NULL),0),COALESCE(SUM(status='pending'),0)
         FROM api_budget_receipts WHERE key_id=?1 AND provider=?2 AND day_start=?3", params![key_id,provider,day],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)),
    )?;
    let legacy: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM api_legacy_usage WHERE provider=?1 AND day_start=?2) OR EXISTS(SELECT 1 FROM usage_events u JOIN request_context c ON c.id=u.context_id WHERE c.provider=?1 AND c.credential_id IS NULL AND u.created_at>=?2 AND u.created_at<?3)",
        params![provider,day,reset_at], |r| r.get(0),
    )?;
    let enabled = token_cap.is_some() || cost_cap_micros.is_some();
    let reason = if enabled && legacy {
        Some("legacy_usage")
    } else if enabled && pending > 0 {
        Some("pending")
    } else if token_cap.is_some() && unknown_tokens > 0 {
        Some("unknown_tokens")
    } else if cost_cap_micros.is_some() && unknown_costs > 0 {
        Some("unknown_costs")
    } else if cost_cap_micros.is_some() && price.is_none() {
        Some("missing_price")
    } else if token_cap.is_some_and(|cap| today_tokens >= cap) {
        Some("token_cap")
    } else if cost_cap_micros.is_some_and(|cap| cost.unwrap_or(0) >= cap) {
        Some("cost_cap")
    } else {
        None
    }
    .map(str::to_owned);
    Ok(ApiBudgetStatus {
        token_cap,
        cost_cap_micros,
        price,
        today_tokens,
        today_cost_micros: if unknown_costs > 0 || pending > 0 {
            None
        } else {
            Some(cost.unwrap_or(0))
        },
        attempts,
        unknown_tokens,
        unknown_costs,
        pending,
        reason,
        reset_at,
    })
}

impl Store {
    pub fn api_budget(
        &self,
        provider: &str,
        key_id: &str,
        model: Option<&str>,
    ) -> Result<ApiBudgetStatus> {
        prune(&self.conn)?;
        budget(&self.conn, provider, key_id, model)
    }

    pub fn save_api_budget(&self, input: &ApiBudgetInput) -> Result<()> {
        validate_identity(&input.provider, &input.key_id, input.model.as_deref())?;
        if input
            .token_cap
            .is_some_and(|n| !(0..=1_000_000_000_000).contains(&n))
            || input
                .cost_cap_micros
                .is_some_and(|n| !(0..=1_000_000_000_000_000).contains(&n))
            || (input.model.is_none() && input.price.is_some())
            || input.price.as_ref().is_some_and(|p| {
                !(0..=1_000_000_000).contains(&p.input_micros_per_million)
                    || !(0..=1_000_000_000).contains(&p.output_micros_per_million)
                    || p.cached_micros_per_million
                        .is_some_and(|n| !(0..=p.input_micros_per_million).contains(&n))
            })
        {
            bail!("invalid API budget policy");
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        prune(&tx)?;
        tx.execute("INSERT INTO api_key_caps(key_id,provider,token_cap,cost_cap_micros) VALUES(?1,?2,?3,?4) ON CONFLICT(key_id) DO UPDATE SET provider=excluded.provider,token_cap=excluded.token_cap,cost_cap_micros=excluded.cost_cap_micros", params![input.key_id,input.provider,input.token_cap,input.cost_cap_micros])?;
        if let Some(model) = &input.model {
            if let Some(price) = &input.price {
                tx.execute("INSERT INTO api_prices(provider,model,input_micros,output_micros,cached_micros) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(provider,model) DO UPDATE SET input_micros=excluded.input_micros,output_micros=excluded.output_micros,cached_micros=excluded.cached_micros",params![input.provider,model,price.input_micros_per_million,price.output_micros_per_million,price.cached_micros_per_million])?;
            } else {
                tx.execute(
                    "DELETE FROM api_prices WHERE provider=?1 AND model=?2",
                    params![input.provider, model],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    // ponytail: admission checks local receipts; enforce hard billing limits at the provider.
    pub fn begin_api_usage(
        &self,
        context_id: i64,
        kind: &str,
        key_id: &str,
    ) -> Result<ApiAdmission> {
        if context_id <= 0 || !matches!(kind, "question" | "transcript") {
            bail!("invalid API usage attempt");
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        prune(&tx)?;
        let (provider,model,credential): (String,String,Option<String>) = tx.query_row(
            "SELECT c.provider,c.model,c.credential_id FROM request_context c JOIN message_requests r ON r.id=c.request_id WHERE c.id=?1 AND r.status='inflight' AND r.kind=?2",params![context_id,kind],|r| {
                super::bounded_row_fields(r, &[(0,10),(1,200),(2,43)]).map_err(|_| rusqlite::Error::InvalidQuery)?;
                Ok((r.get(0)?,r.get(1)?,r.get(2)?))
            },
        )?;
        validate_identity(&provider, key_id, Some(&model))?;
        if credential.as_deref() != Some(key_id) {
            bail!("API usage credential mismatch");
        }
        let state = budget(&tx, &provider, key_id, Some(&model))?;
        if state.reason.is_some() {
            tx.commit()?;
            return Ok(ApiAdmission::Blocked(state));
        }
        tx.execute("INSERT INTO usage_events(id,context_id,kind) SELECT 1+MAX(COALESCE((SELECT MAX(id) FROM usage_events),0),COALESCE((SELECT MAX(usage_id) FROM api_budget_receipts),0)),?1,?2",params![context_id,kind])?;
        let id = tx.last_insert_rowid();
        let day: i64 = tx.query_row(
            "SELECT unixepoch('now','localtime','start of day','utc')",
            [],
            |r| r.get(0),
        )?;
        tx.execute("INSERT INTO api_budget_receipts(usage_id,key_id,provider,model,day_start,status,rate_input,rate_output,rate_cached) VALUES(?1,?2,?3,?4,?5,'pending',?6,?7,?8)",params![id,key_id,provider,model,day,state.price.as_ref().map(|p|p.input_micros_per_million),state.price.as_ref().map(|p|p.output_micros_per_million),state.price.as_ref().and_then(|p|p.cached_micros_per_million)])?;
        tx.commit()?;
        Ok(ApiAdmission::Allowed(id))
    }

    // Called within finish_usage's transaction; terminal receipts cannot be overwritten.
    pub fn finish_api_budget(
        &self,
        usage_id: i64,
        input: Option<i64>,
        output: Option<i64>,
        cached: Option<i64>,
    ) -> Result<Option<i64>> {
        if usage_id <= 0
            || [input, output, cached]
                .iter()
                .flatten()
                .any(|n| !(0..=1_000_000_000_000).contains(n))
            || matches!((cached,input),(Some(c),Some(i)) if c>i)
        {
            bail!("invalid API receipt result");
        }
        let rates = self.conn.query_row("SELECT rate_input,rate_output,rate_cached,status FROM api_budget_receipts WHERE usage_id=?1",[usage_id],|r|Ok((r.get::<_, Option<i64>>(0)?,r.get::<_, Option<i64>>(1)?,r.get::<_, Option<i64>>(2)?,r.get::<_, String>(3)?))).optional()?;
        let Some((rate_input, rate_output, rate_cached, status)) = rates else {
            return Ok(None);
        };
        if status != "pending" {
            bail!("API receipt is already terminal");
        }
        let cost = match (input, output, rate_input, rate_output) {
            (Some(i), Some(o), Some(ri), Some(ro)) if ri >= 0 && ro >= 0 => {
                let c = cached.unwrap_or(0) as u128;
                let rc = rate_cached.unwrap_or(ri);
                if rc < 0 {
                    bail!("invalid stored price");
                }
                let numerator = (i as u128 - c)
                    .checked_mul(ri as u128)
                    .and_then(|n| c.checked_mul(rc as u128).and_then(|v| n.checked_add(v)))
                    .and_then(|n| {
                        (o as u128)
                            .checked_mul(ro as u128)
                            .and_then(|v| n.checked_add(v))
                    })
                    .ok_or_else(|| anyhow::anyhow!("API cost overflow"))?;
                Some(i64::try_from(numerator.div_ceil(1_000_000))?)
            }
            _ => None,
        };
        let changed = self.conn.execute("UPDATE api_budget_receipts SET status=?2,input_tokens=?3,output_tokens=?4,cached_tokens=?5,cost_micros=?6 WHERE usage_id=?1 AND status='pending'",params![usage_id,if input.is_some()&&output.is_some(){"done"}else{"unknown"},input,output,cached,cost])?;
        if changed != 1 {
            bail!("API receipt is already terminal");
        }
        Ok(cost)
    }

    pub fn recover_api_budgets(&self) -> Result<usize> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE api_budget_receipts SET status='unknown' WHERE status='pending'",
            [],
        )?;
        prune(&tx)?;
        tx.commit()?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Settings;
    const KEY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn policy(token_cap: Option<i64>, cost_cap: Option<i64>) -> ApiBudgetInput {
        ApiBudgetInput {
            provider: "deepseek".into(),
            key_id: KEY.into(),
            model: Some("model".into()),
            token_cap,
            cost_cap_micros: cost_cap,
            price: Some(ApiPrice {
                input_micros_per_million: 1_000_000,
                output_micros_per_million: 2_000_000,
                cached_micros_per_million: Some(500_000),
            }),
        }
    }
    fn context(store: &mut Store, meeting: i64) -> Result<i64> {
        let request = store.prepare_question_request(meeting, "question")?;
        store.begin_request(request.id)?;
        let context =
            store.save_request_context(request.id, "deepseek", None, "model", "{}", "{}", false)?;
        store.conn.execute(
            "UPDATE request_context SET credential_id=?1 WHERE id=?2",
            params![KEY, context.id],
        )?;
        Ok(context.id)
    }
    fn another_context(store: &mut Store) -> Result<i64> {
        let meeting = store.create_meeting("another", 0, &Settings::default())?;
        context(store, meeting.id)
    }
    fn allowed(admission: ApiAdmission) -> i64 {
        match admission {
            ApiAdmission::Allowed(id) => id,
            ApiAdmission::Blocked(state) => panic!("unexpected block {:?}", state.reason),
        }
    }

    #[test]
    fn captured_prices_atomic_pending_and_deletion_preserve_caps() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("budgets", 0, &Settings::default())?;
        store.save_api_budget(&policy(Some(11), Some(100)))?;
        let first = context(&mut store, meeting.id)?;
        let usage = allowed(store.begin_api_usage(first, "question", KEY)?);
        let second = another_context(&mut store)?;
        assert!(
            matches!(store.begin_api_usage(second,"question",KEY)?,ApiAdmission::Blocked(s) if s.reason.as_deref()==Some("pending"))
        );
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM usage_events", [], |r| r
                    .get::<_, i64>(0))?,
            1
        );
        let mut changed = policy(Some(11), Some(100));
        changed.price.as_mut().unwrap().input_micros_per_million = 10;
        changed.price.as_mut().unwrap().cached_micros_per_million = Some(5);
        store.save_api_budget(&changed)?;
        store.finish_usage(
            usage,
            "ok",
            Some(10),
            Some(1),
            Some(3),
            None,
            1,
            None,
            false,
        )?;
        let report = store.usage_report(None)?;
        assert_eq!(report.attempts[0].credential_id.as_deref(), Some(KEY));
        assert_eq!(report.attempts[0].cost_micros, Some(11));
        assert_eq!(report.groups[0].credential_id.as_deref(), Some(KEY));
        assert_eq!(report.groups[0].today.estimated_cost_micros, Some(11));
        assert_eq!(report.groups[0].today.unknown_cost_attempts, 0);
        assert!(store
            .finish_api_budget(usage, Some(0), Some(0), None)
            .is_err());
        let state = store.api_budget("deepseek", KEY, Some("model"))?;
        assert_eq!(state.today_tokens, 11);
        assert_eq!(state.today_cost_micros, Some(11));
        assert_eq!(state.reason.as_deref(), Some("token_cap"));
        let now: i64 = store
            .conn
            .query_row("SELECT unixepoch()", [], |r| r.get(0))?;
        assert!((1..=90_000).contains(&(state.reset_at - now)));
        store
            .conn
            .execute("DELETE FROM meetings WHERE id=?1", [meeting.id])?;
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .today_tokens,
            11
        );
        store.save_api_budget(&policy(None, None))?;
        let other = store.create_meeting("next", 0, &Settings::default())?;
        let third = context(&mut store, other.id)?;
        assert!(allowed(store.begin_api_usage(third, "question", KEY)?) > usage);
        Ok(())
    }

    #[test]
    fn unknown_usage_legacy_and_missing_prices_fail_closed_until_reset() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("unknown", 0, &Settings::default())?;
        let first = context(&mut store, meeting.id)?;
        let id = allowed(store.begin_api_usage(first, "question", KEY)?);
        assert_eq!(store.finish_api_budget(id, None, None, None)?, None);
        store.save_api_budget(&policy(Some(100), None))?;
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .reason
                .as_deref(),
            Some("unknown_tokens")
        );
        store.conn.execute("UPDATE api_budget_receipts SET day_start=unixepoch('now','localtime','-1 day','start of day','utc')",[])?;
        assert!(store
            .api_budget("deepseek", KEY, Some("model"))?
            .reason
            .is_none());
        let mut missing = policy(None, Some(100));
        missing.price = None;
        store.save_api_budget(&missing)?;
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .reason
                .as_deref(),
            Some("missing_price")
        );
        let second = another_context(&mut store)?;
        store.conn.execute(
            "UPDATE request_context SET credential_id=NULL WHERE id=?1",
            [second],
        )?;
        store.begin_usage(second, "question")?;
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .reason
                .as_deref(),
            Some("legacy_usage")
        );
        assert!(store.begin_api_usage(second, "question", KEY).is_err());
        store.conn.execute("DELETE FROM meetings", [])?;
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .reason
                .as_deref(),
            Some("legacy_usage")
        );
        Ok(())
    }

    #[test]
    fn recovery_retains_eight_days_and_rounds_conservatively() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("rounding", 0, &Settings::default())?;
        let mut rates = policy(None, None);
        rates.price.as_mut().unwrap().input_micros_per_million = 1;
        rates.price.as_mut().unwrap().output_micros_per_million = 1;
        rates.price.as_mut().unwrap().cached_micros_per_million = None;
        store.save_api_budget(&rates)?;
        let first = context(&mut store, meeting.id)?;
        let id = allowed(store.begin_api_usage(first, "question", KEY)?);
        assert_eq!(
            store.finish_api_budget(id, Some(1), Some(0), None)?,
            Some(1)
        );
        let second = another_context(&mut store)?;
        allowed(store.begin_api_usage(second, "question", KEY)?);
        assert_eq!(store.recover_api_budgets()?, 1);
        assert_eq!(store.recover_api_budgets()?, 0);
        assert_eq!(
            store
                .api_budget("deepseek", KEY, Some("model"))?
                .unknown_tokens,
            1
        );
        store.conn.execute("UPDATE api_budget_receipts SET day_start=unixepoch('now','localtime','-7 days','start of day','utc')",[])?;
        store.recover_api_budgets()?;
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM api_budget_receipts", [], |r| r
                    .get::<_, i64>(0))?,
            2
        );
        store.conn.execute("UPDATE api_budget_receipts SET day_start=unixepoch('now','localtime','-8 days','start of day','utc')",[])?;
        store.recover_api_budgets()?;
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM api_budget_receipts", [], |r| r
                    .get::<_, i64>(0))?,
            0
        );
        assert!(store
            .save_api_budget(&ApiBudgetInput {
                token_cap: Some(-1),
                ..rates
            })
            .is_err());
        Ok(())
    }

    #[test]
    fn ordinary_reads_prune_old_receipts_but_preserve_live_pending() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("retention", 0, &Settings::default())?;
        let first = context(&mut store, meeting.id)?;
        let done = allowed(store.begin_api_usage(first, "question", KEY)?);
        store.finish_api_budget(done, Some(1), Some(1), None)?;
        let second = another_context(&mut store)?;
        let pending = allowed(store.begin_api_usage(second, "question", KEY)?);
        store.conn.execute("UPDATE api_budget_receipts SET day_start=unixepoch('now','localtime','-8 days','start of day','utc')", [])?;
        store.api_budget("deepseek", KEY, Some("model"))?;
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM api_budget_receipts", [], |r| r
                    .get::<_, i64>(0))?,
            1
        );
        assert_eq!(
            store
                .conn
                .query_row("SELECT usage_id FROM api_budget_receipts", [], |r| r
                    .get::<_, i64>(0))?,
            pending
        );
        store.finish_api_budget(pending, Some(1), Some(0), None)?;
        store.api_budget("deepseek", KEY, Some("model"))?;
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM api_budget_receipts", [], |r| r
                    .get::<_, i64>(0))?,
            0
        );
        Ok(())
    }

    #[test]
    fn schema_ten_upgrade_preserves_old_usage() -> Result<()> {
        let path =
            std::env::temp_dir().join(format!("harness-budgets-{}.sqlite", uuid::Uuid::new_v4()));
        let result = (|| -> Result<()> {
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("upgrade", 0, &Settings::default())?;
            let context = context(&mut store, meeting.id)?;
            let old = store.begin_usage(context, "question")?;
            store.conn.execute_batch("DROP TABLE api_legacy_usage; DROP TRIGGER api_usage_legacy; DROP TABLE api_budget_receipts; DROP TABLE api_key_caps; DROP TABLE api_prices; ALTER TABLE usage_events DROP COLUMN cost_micros; ALTER TABLE request_context DROP COLUMN credential_id; PRAGMA user_version=10;")?;
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(store.usage_report(None)?.attempts[0].id, old);
            store.conn.execute("DELETE FROM meetings", [])?;
            store.save_api_budget(&policy(Some(100), None))?;
            assert_eq!(
                store
                    .api_budget("deepseek", KEY, Some("model"))?
                    .reason
                    .as_deref(),
                Some("legacy_usage")
            );
            Ok(())
        })();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }
}

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    use crate::store::Settings;
    use std::sync::{Arc, Barrier};
    #[test]
    fn concurrent_connections_admit_one_pending_attempt() -> Result<()> {
        let path = std::env::temp_dir().join(format!(
            "harness-budget-race-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let result = (|| -> Result<()> {
            let mut store = Store::open(&path)?;
            let key = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
            store.save_api_budget(&ApiBudgetInput {
                provider: "gemini".into(),
                key_id: key.into(),
                model: None,
                token_cap: Some(100),
                cost_cap_micros: None,
                price: None,
            })?;
            let mut contexts = Vec::new();
            for _ in 0..2 {
                let meeting = store.create_meeting("race", 0, &Settings::default())?;
                let request = store.prepare_question_request(meeting.id, "question")?;
                store.begin_request(request.id)?;
                let context = store
                    .save_request_context(request.id, "gemini", None, "model", "{}", "{}", false)?;
                store.conn.execute(
                    "UPDATE request_context SET credential_id=?1 WHERE id=?2",
                    params![key, context.id],
                )?;
                contexts.push(context.id);
            }
            let gate = Arc::new(Barrier::new(3));
            let threads: Vec<_> = contexts
                .into_iter()
                .map(|id| {
                    let connection = Store::open(&path).expect("open test connection");
                    let gate = gate.clone();
                    std::thread::spawn(move || -> Result<ApiAdmission> {
                        gate.wait();
                        connection.begin_api_usage(id, "question", key)
                    })
                })
                .collect();
            gate.wait();
            let mut admitted = 0;
            for thread in threads {
                match thread.join().expect("budget thread panicked")? {
                    ApiAdmission::Allowed(_) => admitted += 1,
                    ApiAdmission::Blocked(s) => assert_eq!(s.reason.as_deref(), Some("pending")),
                }
            }
            assert_eq!(admitted, 1);
            assert_eq!(store.api_budget("gemini", key, Some("model"))?.pending, 1);
            Ok(())
        })();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }
}
