ALTER TABLE request_context ADD COLUMN credential_id TEXT;
ALTER TABLE usage_events ADD COLUMN cost_micros INTEGER;
CREATE TABLE api_prices (
 provider TEXT NOT NULL CHECK(provider IN ('gemini','deepseek')),
 model TEXT NOT NULL CHECK(length(model) BETWEEN 1 AND 200),
 input_micros INTEGER NOT NULL CHECK(input_micros BETWEEN 0 AND 1000000000),
 output_micros INTEGER NOT NULL CHECK(output_micros BETWEEN 0 AND 1000000000),
 cached_micros INTEGER CHECK(cached_micros BETWEEN 0 AND input_micros),
 PRIMARY KEY(provider,model)
);
CREATE TABLE api_key_caps (
 key_id TEXT PRIMARY KEY CHECK(length(key_id)=43),
 provider TEXT NOT NULL CHECK(provider IN ('gemini','deepseek')),
 token_cap INTEGER CHECK(token_cap BETWEEN 0 AND 1000000000000),
 cost_cap_micros INTEGER CHECK(cost_cap_micros BETWEEN 0 AND 1000000000000000)
);
-- Anonymous accounting survives meeting deletion so deletion cannot reset a cap.
CREATE TABLE api_budget_receipts (
 usage_id INTEGER PRIMARY KEY,
 key_id TEXT NOT NULL CHECK(length(key_id)=43),
 provider TEXT NOT NULL CHECK(provider IN ('gemini','deepseek')),
 model TEXT NOT NULL,
 day_start INTEGER NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('pending','done','unknown')),
 input_tokens INTEGER CHECK(input_tokens BETWEEN 0 AND 1000000000000),
 output_tokens INTEGER CHECK(output_tokens BETWEEN 0 AND 1000000000000),
 cached_tokens INTEGER CHECK(cached_tokens BETWEEN 0 AND input_tokens),
 cost_micros INTEGER CHECK(cost_micros >= 0),
 rate_input INTEGER CHECK(rate_input BETWEEN 0 AND 1000000000),
 rate_output INTEGER CHECK(rate_output BETWEEN 0 AND 1000000000),
 rate_cached INTEGER CHECK(rate_cached BETWEEN 0 AND rate_input)
);
CREATE INDEX api_budget_receipts_day ON api_budget_receipts(key_id,day_start);
CREATE INDEX api_budget_receipts_expiry ON api_budget_receipts(day_start) WHERE status != 'pending';
-- Unattributed legacy usage must remain conservative after meeting deletion.
CREATE TABLE api_legacy_usage (
 provider TEXT NOT NULL CHECK(provider IN ('gemini','deepseek')),
 day_start INTEGER NOT NULL,
 PRIMARY KEY(provider,day_start)
);
INSERT OR IGNORE INTO api_legacy_usage(provider,day_start)
 SELECT c.provider,unixepoch(u.created_at,'unixepoch','localtime','start of day','utc')
 FROM usage_events u JOIN request_context c ON c.id=u.context_id
 WHERE c.provider IN ('gemini','deepseek') AND c.credential_id IS NULL;
CREATE TRIGGER api_usage_legacy AFTER INSERT ON usage_events BEGIN
 INSERT OR IGNORE INTO api_legacy_usage(provider,day_start)
 SELECT c.provider,unixepoch(new.created_at,'unixepoch','localtime','start of day','utc')
 FROM request_context c WHERE c.id=new.context_id
 AND c.provider IN ('gemini','deepseek') AND c.credential_id IS NULL;
END;
