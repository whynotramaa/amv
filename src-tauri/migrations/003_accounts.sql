CREATE TABLE oauth_host (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    host_id TEXT NOT NULL
);
-- Public identity only. Tokens are encrypted in the Windows credential vault.
CREATE TABLE chatgpt_accounts (
    id TEXT PRIMARY KEY,
    metadata TEXT NOT NULL
);
CREATE TABLE active_chatgpt_account (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    account_id TEXT NOT NULL REFERENCES chatgpt_accounts(id) ON DELETE CASCADE
);
