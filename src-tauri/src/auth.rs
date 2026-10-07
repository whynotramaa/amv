//! Official Sign in with ChatGPT dynamic public-client flow.
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use futures_util::future::{select, Either};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;
use zeroize::{Zeroize, Zeroizing};

const ISSUER: &str = "https://auth.openai.com";
const AUTHORIZE: &str = "https://auth.openai.com/api/accounts/authorize";
const TOKEN: &str = "https://auth.openai.com/api/accounts/oauth/token";
const JWKS: &str = "https://auth.openai.com/.well-known/jwks.json";
const REVOKE: &str = "https://auth.openai.com/api/accounts/oauth/revoke";
const DYNAMIC_CLIENT: &str = "dynamic_agent_client";
const RESOURCE: &str = "https://api.openai.com/v1";
const SCOPES: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
const MAX_HEADERS: usize = 8 * 1024;
const MAX_BODY: usize = 1024 * 1024;

#[derive(Clone)]
pub struct AuthAttempt {
    listener: std::sync::Arc<TcpListener>,
    redirect_uri: String,
    authorization: String,
    client_id: String,
    state: Zeroizing<String>,
    verifier: Zeroizing<String>,
    nonce: Zeroizing<String>,
    expected_subject: Option<String>,
}

pub struct AuthGrant {
    pub account_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub issued_client_id: String,
    pub subscopes: Vec<String>,
    subject: String,
    tokens: OpaqueTokenBundle,
}

/// Token material deliberately has no `Debug` or `Serialize` implementation.
pub struct OpaqueTokenBundle {
    access_token: Zeroizing<String>,
    refresh_token: Zeroizing<String>,
    id_token: Zeroizing<String>,
    expires_at: u64,
}

impl OpaqueTokenBundle {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    pub fn id_token(&self) -> &str {
        &self.id_token
    }
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
}

impl AuthAttempt {
    pub async fn prepare(host_id: &str) -> Result<Self> {
        if host_id.trim().is_empty() || host_id.len() > 256 || host_id.chars().any(char::is_control)
        {
            bail!("Invalid host identifier")
        }
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .context("Unable to start the local sign-in callback")?;
        let redirect_uri = format!(
            "http://127.0.0.1:{}/auth/callback",
            listener.local_addr()?.port()
        );
        let state = random_url_secret(32)?;
        let nonce = random_url_secret(32)?;
        let verifier = random_url_secret(32)?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let client_id = DYNAMIC_CLIENT.to_owned();
        let mut query = vec![
            ("response_type", "code".to_owned()),
            ("client_id", client_id.clone()),
            ("agent_name_hint", "Harness".into()),
            ("ext_agent_host_id", host_id.into()),
            ("redirect_uri", redirect_uri.clone()),
            ("scope", SCOPES.into()),
            ("resource", RESOURCE.into()),
            ("code_challenge", challenge),
            ("code_challenge_method", "S256".into()),
            ("state", state.to_string()),
            ("nonce", nonce.to_string()),
        ];
        let authorization = Url::parse_with_params(AUTHORIZE, query.drain(..))?.to_string();
        Ok(Self {
            listener: std::sync::Arc::new(listener),
            redirect_uri,
            authorization,
            client_id,
            state,
            verifier,
            nonce,
            expected_subject: None,
        })
    }

    pub fn authorization_url(&self) -> &str {
        &self.authorization
    }

    pub async fn prepare_for_client(
        host_id: &str,
        issued_client_id: &str,
        id_token_hint: Option<&str>,
        login_hint: Option<&str>,
    ) -> Result<Self> {
        validate_client_id(issued_client_id)?;
        if id_token_hint.is_some_and(|hint| {
            hint.is_empty() || hint.len() > 16 * 1024 || hint.chars().any(char::is_control)
        }) || login_hint.is_some_and(|hint| {
            hint.is_empty() || hint.len() > 512 || hint.chars().any(char::is_control)
        }) {
            bail!("Invalid account sign-in hint")
        }
        let mut attempt = Self::prepare(host_id).await?;
        attempt.client_id = issued_client_id.to_owned();
        let mut url = Url::parse(&attempt.authorization)?;
        let mut query: Vec<(String, String)> = url
            .query_pairs()
            .into_owned()
            .filter(|(key, _)| key != "client_id" && key != "agent_name_hint")
            .collect();
        query.insert(0, ("client_id".into(), issued_client_id.into()));
        if let Some(hint) = id_token_hint {
            query.push(("id_token_hint".into(), hint.into()));
        }
        if let Some(hint) = login_hint {
            query.push(("login_hint".into(), hint.into()));
        }
        url.set_query(None);
        url.query_pairs_mut().extend_pairs(query);
        attempt.authorization = url.to_string();
        Ok(attempt)
    }

    pub async fn prepare_for_account(
        host_id: &str,
        issued_client_id: &str,
        subject: &str,
        id_token_hint: Option<&str>,
        login_hint: Option<&str>,
    ) -> Result<Self> {
        if subject.is_empty() || subject.len() > 512 || subject.chars().any(char::is_control) {
            bail!("Invalid account subject")
        }
        let mut attempt =
            Self::prepare_for_client(host_id, issued_client_id, id_token_hint, login_hint).await?;
        attempt.expected_subject = Some(subject.to_owned());
        Ok(attempt)
    }

    pub async fn finish(
        self,
        client: &Client,
        gate: &tokio::sync::Mutex<()>,
        cancel: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<AuthGrant> {
        let (callback, mut cancel) = match select(
            Box::pin(tokio::time::timeout(
                Duration::from_secs(120),
                self.callback(),
            )),
            Box::pin(cancel),
        )
        .await
        {
            Either::Left((callback, cancel)) => (
                callback.map_err(|_| anyhow::anyhow!("OAuth callback timed out"))??,
                cancel,
            ),
            Either::Right((_, _)) => bail!("Sign-in cancelled"),
        };
        let _gate = gate.lock().await;
        if !matches!(
            cancel.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ) {
            bail!("Sign-in cancelled")
        }
        let issued_client_id = callback
            .client_id
            .clone()
            .unwrap_or_else(|| self.client_id.clone());
        validate_client_id(&issued_client_id)?;
        let exchange: Result<TokenResponse> = async {
            let response = checked_response(
                client.post(TOKEN).form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", issued_client_id.as_str()),
                    ("code", callback.code.as_str()),
                    ("code_verifier", self.verifier.as_str()),
                    ("redirect_uri", self.redirect_uri.as_str()),
                    ("resource", RESOURCE),
                ]),
                TOKEN,
            )
            .await?;
            let body = bounded_body(response).await?;
            if !body.status.is_success() {
                bail!("OAuth token exchange failed ({})", body.status.as_u16())
            }
            let token: TokenResponse =
                serde_json::from_slice(&body.bytes).context("Invalid OAuth token response")?;
            Ok(token)
        }
        .await;
        let token = exchange.map_err(|error| {
            if self.client_id == DYNAMIC_CLIENT {
                error.context(RegistrationExchangeFailed {
                    client_id: issued_client_id.clone(),
                })
            } else {
                error
            }
        })?;
        if let Err(error) = token.validate_required() {
            revoke_rejected_token(client, &issued_client_id, &token).await;
            return Err(error);
        }
        let claims = validate_id_token(
            client,
            token.id_token.as_deref().unwrap_or_default(),
            &issued_client_id,
            Some(&self.nonce),
            self.expected_subject.as_deref(),
        )
        .await;
        let claims = match claims {
            Ok(claims) => claims,
            Err(error) => {
                revoke_rejected_token(client, &issued_client_id, &token).await;
                return Err(error);
            }
        };
        let subject = claims.sub;
        let account_id = account_key(&issued_client_id, &subject);
        Ok(AuthGrant {
            account_id,
            display_name: safe_metadata(claims.name),
            email: safe_metadata(claims.email),
            issued_client_id,
            subscopes: token
                .scope
                .as_deref()
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
            subject,
            tokens: token.bundle()?,
        })
    }

    async fn callback(&self) -> Result<Callback> {
        loop {
            let (mut stream, _) = self.listener.accept().await?;
            let request = tokio::time::timeout(Duration::from_secs(5), read_headers(&mut stream))
                .await
                .map_err(|_| anyhow::anyhow!("Callback read timed out"))
                .and_then(|r| r);
            let result = request.and_then(|request| {
                parse_callback(&request, &self.redirect_uri, &self.state, &self.client_id)
            });
            let success = result.is_ok();
            let cancelled = result
                .as_ref()
                .is_err_and(|error| error.is::<SignInCancelled>());
            let _ = stream.write_all(if success { b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nSign-in received. Return to Harness to check its status." } else { b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\nSign-in request rejected." }).await;
            if success || cancelled {
                return result;
            }
        }
    }
}

impl AuthGrant {
    pub fn plan_usage_enabled(&self) -> bool {
        self.subscopes
            .iter()
            .any(|scope| scope == "chatgpt.tokens.use.direct")
    }

    pub fn tokens(&self) -> &OpaqueTokenBundle {
        &self.tokens
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Only pass this zeroized buffer directly into the encrypted credential vault.
    pub fn to_vault_bytes(&self) -> Result<Zeroizing<Vec<u8>>> {
        validate_grant(self)?;
        let stored = StoredGrant {
            account_id: self.account_id.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            issued_client_id: self.issued_client_id.clone(),
            subscopes: self.subscopes.clone(),
            subject: self.subject.clone(),
            access_token: self.tokens.access_token.to_string(),
            refresh_token: self.tokens.refresh_token.to_string(),
            id_token: self.tokens.id_token.to_string(),
            expires_at: self.tokens.expires_at,
        };
        Ok(Zeroizing::new(serde_json::to_vec(&stored)?))
    }

    pub fn from_vault_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 64 * 1024 {
            bail!("Credential record too large")
        }
        let mut stored: StoredGrant =
            serde_json::from_slice(bytes).context("Invalid stored OAuth grant")?;
        let grant = Self {
            account_id: std::mem::take(&mut stored.account_id),
            display_name: stored.display_name.take(),
            email: stored.email.take(),
            issued_client_id: std::mem::take(&mut stored.issued_client_id),
            subscopes: std::mem::take(&mut stored.subscopes),
            subject: std::mem::take(&mut stored.subject),
            tokens: OpaqueTokenBundle {
                access_token: Zeroizing::new(std::mem::take(&mut stored.access_token)),
                refresh_token: Zeroizing::new(std::mem::take(&mut stored.refresh_token)),
                id_token: Zeroizing::new(std::mem::take(&mut stored.id_token)),
                expires_at: stored.expires_at,
            },
        };
        validate_grant(&grant)?;
        Ok(grant)
    }

    pub async fn refresh(&self, client: &Client) -> Result<Self> {
        validate_grant(self)?;
        let response =
            checked_response(client.post(TOKEN).form(&refresh_form(self)), TOKEN).await?;
        let rotated = response.status() == StatusCode::OK;
        let body = bounded_body(response).await.map_err(|error| {
            if rotated {
                error.context(RefreshRequiresSignIn)
            } else {
                error
            }
        })?;
        if body.status != StatusCode::OK {
            return Err(refresh_response_error(body.status, &body.bytes));
        }
        // A successful refresh consumes the old rotating token. Later validation failures are terminal.
        async {
            let token: TokenResponse =
                serde_json::from_slice(&body.bytes).context("Invalid OAuth refresh response")?;
            if let Err(error) = token.validate_required() {
                revoke_rejected_token(client, &self.issued_client_id, &token).await;
                return Err(error);
            }
            let claims = validate_id_token(
                client,
                token.id_token.as_deref().unwrap_or_default(),
                &self.issued_client_id,
                None,
                Some(&self.subject),
            )
            .await;
            let claims = match claims {
                Ok(claims) => claims,
                Err(error) => {
                    revoke_rejected_token(client, &self.issued_client_id, &token).await;
                    return Err(error);
                }
            };
            let subject = claims.sub;
            if subject != self.subject {
                bail!("OAuth account changed during refresh")
            }
            Ok(Self {
                account_id: account_key(&self.issued_client_id, &subject),
                display_name: safe_metadata(claims.name).or_else(|| self.display_name.clone()),
                email: safe_metadata(claims.email).or_else(|| self.email.clone()),
                issued_client_id: self.issued_client_id.clone(),
                subscopes: token
                    .scope
                    .as_deref()
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
                subject,
                tokens: token.bundle()?,
            })
        }
        .await
        .context(RefreshRequiresSignIn)
    }

    pub async fn revoke(&self, client: &Client) -> Result<()> {
        validate_grant(self)?;
        let response = checked_response(
            client.post(REVOKE).form(&[
                ("token", self.tokens.refresh_token()),
                ("token_type_hint", "refresh_token"),
                ("client_id", self.issued_client_id.as_str()),
            ]),
            REVOKE,
        )
        .await?;
        if !response.status().is_success() {
            bail!("OAuth revocation failed ({})", response.status().as_u16())
        }
        Ok(())
    }
}

pub fn account_key(client_id: &str, subject: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(client_id.as_bytes());
    hash.update([0]);
    hash.update(subject.as_bytes());
    URL_SAFE_NO_PAD.encode(hash.finalize())
}

fn validate_grant(grant: &AuthGrant) -> Result<()> {
    validate_client_id(&grant.issued_client_id)?;
    validate_scopes(&grant.subscopes)?;
    if grant.subject.is_empty()
        || grant.account_id != account_key(&grant.issued_client_id, &grant.subject)
        || grant.tokens.refresh_token.is_empty()
    {
        bail!("Invalid OAuth grant")
    }
    if grant.issued_client_id.len() > 256
        || grant.issued_client_id.chars().any(char::is_control)
        || grant.subject.len() > 512
        || grant.subject.chars().any(char::is_control)
        || [
            grant.tokens.access_token(),
            grant.tokens.refresh_token(),
            grant.tokens.id_token(),
        ]
        .iter()
        .any(|value| {
            value.is_empty() || value.len() > 16 * 1024 || value.chars().any(char::is_control)
        })
    {
        bail!("Invalid stored OAuth grant")
    }
    Ok(())
}

fn validate_client_id(client_id: &str) -> Result<()> {
    // Issued IDs are opaque; validate transport syntax without guessing a prefix.
    if client_id.is_empty()
        || client_id == DYNAMIC_CLIENT
        || client_id.len() > 256
        || !client_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        bail!("Invalid issued client identifier")
    }
    Ok(())
}

fn validate_scopes(scopes: &[String]) -> Result<()> {
    if scopes.is_empty()
        || scopes.len() > 64
        || scopes.iter().any(|scope| {
            scope.is_empty()
                || scope.len() > 256
                || scope.chars().any(|c| c.is_control() || c.is_whitespace())
        })
    {
        bail!("Invalid OAuth granted scopes")
    }
    Ok(())
}

fn refresh_form(grant: &AuthGrant) -> [(&str, &str); 4] {
    [
        ("grant_type", "refresh_token"),
        ("client_id", grant.issued_client_id.as_str()),
        ("refresh_token", grant.tokens.refresh_token()),
        ("resource", RESOURCE),
    ]
}

#[derive(Debug)]
struct RegistrationExchangeFailed {
    client_id: String,
}
impl std::fmt::Display for RegistrationExchangeFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Registration code exchange failed. Retry sign-in")
    }
}
impl std::error::Error for RegistrationExchangeFailed {}

/// Public registration identity only. Authorization codes and token material are never retained here.
pub fn registration_retry_client(error: &anyhow::Error) -> Option<&str> {
    error
        .downcast_ref::<RegistrationExchangeFailed>()
        .map(|error| error.client_id.as_str())
}

#[derive(Debug)]
struct RefreshRequiresSignIn;
impl std::fmt::Display for RefreshRequiresSignIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The ChatGPT session cannot be renewed. Sign in again")
    }
}
impl std::error::Error for RefreshRequiresSignIn {}

pub fn refresh_requires_sign_in(error: &anyhow::Error) -> bool {
    error.is::<RefreshRequiresSignIn>()
}

fn refresh_response_error(status: StatusCode, bytes: &[u8]) -> anyhow::Error {
    let error = anyhow::anyhow!("OAuth refresh failed ({})", status.as_u16());
    if !status.is_client_error() || status == StatusCode::TOO_MANY_REQUESTS {
        return error;
    }
    let body: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(body) => body,
        Err(_) => return error,
    };
    let code = body
        .get("error")
        .and_then(|value| value.as_str().or_else(|| value.get("code")?.as_str()));
    if matches!(
        code,
        Some(
            "invalid_grant"
                | "invalid_refresh_token"
                | "token_expired"
                | "refresh_token_expired"
                | "refresh_token_invalidated"
                | "refresh_token_reused"
        )
    ) {
        error.context(RefreshRequiresSignIn)
    } else if code == Some("invalid_client") {
        error.context("The ChatGPT client configuration was rejected")
    } else {
        error
    }
}

#[derive(Serialize, Deserialize)]
struct StoredGrant {
    account_id: String,
    display_name: Option<String>,
    email: Option<String>,
    issued_client_id: String,
    subscopes: Vec<String>,
    subject: String,
    access_token: String,
    refresh_token: String,
    id_token: String,
    expires_at: u64,
}
impl Drop for StoredGrant {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
        self.id_token.zeroize();
    }
}

fn random_url_secret(size: usize) -> Result<Zeroizing<String>> {
    let mut bytes = Zeroizing::new(vec![0; size]);
    getrandom::fill(&mut bytes).map_err(|_| anyhow::anyhow!("Secure random generation failed"))?;
    Ok(Zeroizing::new(URL_SAFE_NO_PAD.encode(&*bytes)))
}

struct Callback {
    code: String,
    client_id: Option<String>,
}
impl Drop for Callback {
    fn drop(&mut self) {
        self.code.zeroize();
    }
}

fn parse_callback(
    request: &[u8],
    redirect: &str,
    expected_state: &str,
    pending_client: &str,
) -> Result<Callback> {
    let line = std::str::from_utf8(request)
        .map_err(|_| anyhow::anyhow!("Invalid callback request"))?
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing callback request"))?;
    let mut parts = line.split_whitespace();
    if parts.next() != Some("GET") {
        bail!("Invalid callback method")
    }
    let target = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing callback target"))?;
    if !target.starts_with('/') || target.starts_with("//") || target.contains('#') {
        bail!("Invalid callback target")
    }
    let expected = Url::parse(redirect)?;
    let url = expected.join(target)?;
    let host = format!(
        "{}:{}",
        expected.host_str().unwrap_or_default(),
        expected
            .port()
            .ok_or_else(|| anyhow::anyhow!("Invalid callback port"))?
    );
    let headers =
        std::str::from_utf8(request).map_err(|_| anyhow::anyhow!("Invalid callback headers"))?;
    let hosts: Vec<_> = headers
        .lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    if hosts != [host.as_str()] {
        bail!("Invalid callback Host")
    }
    if url.path() != expected.path()
        || url.host_str() != expected.host_str()
        || url.port() != expected.port()
    {
        bail!("Invalid callback target")
    }
    let mut values: HashMap<String, String> = HashMap::new();
    for (key, value) in url.query_pairs() {
        if values
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            bail!("Duplicate callback parameter")
        }
    }
    let state = values
        .remove("state")
        .ok_or_else(|| anyhow::anyhow!("Missing callback state"))?;
    if !constant_time_eq(state.as_bytes(), expected_state.as_bytes()) {
        bail!("Callback state mismatch")
    }
    if let Some(error) = values.remove("error") {
        if error == "access_denied" {
            return Err(SignInCancelled.into());
        }
        bail!("Sign-in failed")
    }
    let code = values
        .remove("code")
        .filter(|v| !v.is_empty() && v.len() <= 4096 && !v.chars().any(char::is_control))
        .ok_or_else(|| anyhow::anyhow!("Missing authorization code"))?;
    let client_id = values.remove("client_id");
    if pending_client == DYNAMIC_CLIENT {
        validate_client_id(client_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!("Registration did not return an issued client identifier")
        })?)?;
    } else {
        validate_client_id(pending_client)?;
        if client_id.as_deref().is_some_and(|id| id != pending_client) {
            bail!("OAuth client mismatch")
        }
    }
    Ok(Callback { code, client_id })
}

#[derive(Debug)]
struct SignInCancelled;
impl std::fmt::Display for SignInCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sign-in cancelled")
    }
}
impl std::error::Error for SignInCancelled {}

async fn read_headers(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(1024);
    loop {
        let mut chunk = [0; 512];
        let count = stream.read(&mut chunk).await?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() > MAX_HEADERS {
            bail!("Callback headers too large")
        }
        if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
            return Ok(bytes);
        }
    }
    bail!("Incomplete callback request")
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut result = 0u8;
    for (a, b) in left.iter().zip(right) {
        result |= a ^ b;
    }
    result == 0
}

async fn checked_response(request: reqwest::RequestBuilder, endpoint: &str) -> Result<Response> {
    let response = request
        .send()
        .await
        .context("OAuth network request failed")?;
    if response.url().as_str() != endpoint {
        bail!("OAuth endpoint redirected")
    }
    Ok(response)
}

struct Body {
    status: StatusCode,
    bytes: Zeroizing<Vec<u8>>,
}

async fn bounded_body(response: Response) -> Result<Body> {
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BODY as u64)
    {
        bail!("OAuth response too large")
    }
    let mut bytes = Zeroizing::new(Vec::new());
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("OAuth response read failed")?;
        if bytes.len() + chunk.len() > MAX_BODY {
            bail!("OAuth response too large")
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Body { status, bytes })
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
}

impl Drop for TokenResponse {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
        self.id_token.zeroize();
    }
}

impl TokenResponse {
    fn validate_required(&self) -> Result<()> {
        if self.access_token.as_deref().unwrap_or_default().is_empty()
            || self.refresh_token.as_deref().unwrap_or_default().is_empty()
            || self.id_token.as_deref().unwrap_or_default().is_empty()
        {
            bail!("OAuth token response is incomplete")
        }
        if self
            .token_type
            .as_deref()
            .is_some_and(|kind| !kind.eq_ignore_ascii_case("bearer"))
            || [
                self.access_token.as_deref(),
                self.refresh_token.as_deref(),
                self.id_token.as_deref(),
            ]
            .iter()
            .flatten()
            .any(|token| token.len() > 16 * 1024 || token.chars().any(char::is_control))
            || self
                .scope
                .as_ref()
                .is_some_and(|scope| scope.len() > 4096 || scope.chars().any(char::is_control))
        {
            bail!("Invalid OAuth token response")
        }
        let scopes = self
            .scope
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        validate_scopes(&scopes)?;
        Ok(())
    }
    fn bundle(&self) -> Result<OpaqueTokenBundle> {
        let expires_at = match self.expires_in {
            Some(seconds) => now()?.saturating_add(seconds),
            None => jwt_claims(self.access_token.as_deref().unwrap_or_default())
                .and_then(|claims| claims.get("exp")?.as_u64())
                .unwrap_or(now()?.saturating_add(3600)),
        };
        Ok(OpaqueTokenBundle {
            access_token: Zeroizing::new(
                self.access_token
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing access token"))?,
            ),
            refresh_token: Zeroizing::new(
                self.refresh_token
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing refresh token"))?,
            ),
            id_token: Zeroizing::new(
                self.id_token
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Missing ID token"))?,
            ),
            expires_at,
        })
    }
}

async fn revoke_rejected_token(client: &Client, client_id: &str, token: &TokenResponse) {
    let Some(refresh) = token.refresh_token.as_deref().filter(|value| {
        !value.is_empty() && value.len() <= 16 * 1024 && !value.chars().any(char::is_control)
    }) else {
        return;
    };
    let _ = tokio::time::timeout(
        Duration::from_secs(10),
        checked_response(
            client.post(REVOKE).form(&[
                ("token", refresh),
                ("token_type_hint", "refresh_token"),
                ("client_id", client_id),
            ]),
            REVOKE,
        ),
    )
    .await;
}

fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
fn jwt_claims(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?).ok()
}

fn safe_metadata(value: Option<String>) -> Option<String> {
    value.filter(|value| value.len() <= 512 && !value.chars().any(char::is_control))
}

#[derive(Deserialize)]
struct IdClaims {
    iss: String,
    sub: String,
    exp: usize,
    nonce: Option<String>,
    name: Option<String>,
    email: Option<String>,
}
#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}
#[derive(Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    n: String,
    e: String,
}

async fn validate_id_token(
    client: &Client,
    token: &str,
    audience: &str,
    nonce: Option<&str>,
    subject: Option<&str>,
) -> Result<IdClaims> {
    let header = decode_header(token).context("Invalid ID token")?;
    if header.alg != Algorithm::RS256 {
        bail!("Unsupported ID token signature")
    }
    let response = checked_response(client.get(JWKS), JWKS).await?;
    let body = bounded_body(response).await?;
    if !body.status.is_success() {
        bail!("Unable to validate ID token")
    }
    let keys: Jwks = serde_json::from_slice(&body.bytes).context("Invalid identity key set")?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[audience]);
    let mut claims = None;
    for key in keys
        .keys
        .into_iter()
        .filter(|key| key.kty == "RSA" && (header.kid.is_none() || key.kid == header.kid))
    {
        let decoding =
            DecodingKey::from_rsa_components(&key.n, &key.e).context("Invalid identity key")?;
        if let Ok(value) = decode::<IdClaims>(token, &decoding, &validation) {
            claims = Some(value.claims);
            break;
        }
    }
    let claims = claims.ok_or_else(|| anyhow::anyhow!("ID token validation failed"))?;
    validate_identity(&claims, nonce, subject, now()?)?;
    Ok(claims)
}

fn validate_identity(
    claims: &IdClaims,
    nonce: Option<&str>,
    subject: Option<&str>,
    time: u64,
) -> Result<()> {
    if claims.exp as u64 <= time
        || claims.sub.is_empty()
        || claims.sub.len() > 512
        || claims.sub.chars().any(char::is_control)
        || claims.iss != ISSUER
        || nonce.is_some_and(|expected| claims.nonce.as_deref() != Some(expected))
        || subject.is_some_and(|expected| claims.sub != expected)
    {
        bail!("ID token identity mismatch")
    }
    Ok(())
}

impl Drop for OpaqueTokenBundle {
    fn drop(&mut self) {
        self.expires_at.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_rotation_requires_a_replacement_refresh_token() {
        let complete = br#"{"access_token":"access","refresh_token":"replacement","id_token":"identity","token_type":"Bearer","expires_in":3600,"scope":"chatgpt.tokens.use.direct"}"#;
        let token: TokenResponse = serde_json::from_slice(complete).unwrap();
        assert!(token.validate_required().is_ok());
        let incomplete = br#"{"access_token":"access","id_token":"identity","token_type":"Bearer","expires_in":3600,"scope":"chatgpt.tokens.use.direct"}"#;
        let token: TokenResponse = serde_json::from_slice(incomplete).unwrap();
        assert!(token.validate_required().is_err());
    }

    #[test]
    fn pkce_and_url_are_safe() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let attempt = runtime
            .block_on(AuthAttempt::prepare("urn:uuid:test"))
            .unwrap();
        let url = Url::parse(attempt.authorization_url()).unwrap();
        assert_eq!(url.path(), "/api/accounts/authorize");
        assert_eq!(
            url.query_pairs().find(|(k, _)| k == "client_id").unwrap().1,
            DYNAMIC_CLIENT
        );
        assert!(url
            .query_pairs()
            .any(|(k, v)| k == "code_challenge" && !v.is_empty()));
    }

    #[test]
    fn cancellation_before_callback_never_reaches_token_exchange() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (cancel, receiver) = tokio::sync::oneshot::channel();
        let (attempt, client) = runtime.block_on(async {
            let attempt =
                AuthAttempt::prepare_for_client("urn:uuid:test", "oaiapp_saved", None, None)
                    .await
                    .unwrap();
            (attempt, Client::new())
        });
        cancel.send(()).unwrap();
        let result = runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(1),
                attempt.finish(&client, &tokio::sync::Mutex::new(()), receiver),
            )
            .await
        });
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn callback_guards_state_client_and_errors() {
        let request = |query: &str| {
            format!("GET /auth/callback?{query} HTTP/1.1\r\nHost: 127.0.0.1:9\r\n\r\n").into_bytes()
        };
        assert!(parse_callback(
            &request("state=bad&code=x&client_id=issued"),
            "http://127.0.0.1:9/auth/callback",
            "good",
            DYNAMIC_CLIENT
        )
        .is_err());
        assert!(parse_callback(
            &request("state=good&error=access_denied"),
            "http://127.0.0.1:9/auth/callback",
            "good",
            DYNAMIC_CLIENT
        )
        .is_err());
        assert!(parse_callback(
            &request("state=good&code=x"),
            "http://127.0.0.1:9/auth/callback",
            "good",
            DYNAMIC_CLIENT
        )
        .is_err());
        assert!(parse_callback(
            &request("state=good&code=x&client_id=issued"),
            "http://127.0.0.1:9/auth/callback",
            "good",
            DYNAMIC_CLIENT
        )
        .is_ok());
    }
    #[test]
    fn dynamic_registration_and_returning_accounts_keep_security_bindings() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let host = uuid::Uuid::new_v4().urn().to_string();
        let (initial, returning) = runtime.block_on(async {
            (
                AuthAttempt::prepare(&host).await.unwrap(),
                AuthAttempt::prepare_for_account(
                    &host,
                    "oaiapp_saved",
                    "subject",
                    Some("old.identity.hint"),
                    Some("user@example.com"),
                )
                .await
                .unwrap(),
            )
        });
        let initial_url = Url::parse(initial.authorization_url()).unwrap();
        let initial_values: HashMap<_, _> = initial_url.query_pairs().into_owned().collect();
        assert_eq!(initial_values["client_id"], "dynamic_agent_client");
        assert_eq!(initial_values["agent_name_hint"], "Harness");
        assert_eq!(initial_values["ext_agent_host_id"], host);
        assert_eq!(initial_values.len(), 11);
        for attempt in [&initial, &returning] {
            let url = Url::parse(attempt.authorization_url()).unwrap();
            assert_eq!(url.scheme(), "https");
            assert_eq!(url.host_str(), Some("auth.openai.com"));
            assert_eq!(url.path(), "/api/accounts/authorize");
            let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert_eq!(url.query_pairs().count(), query.len());
            assert_eq!(query["response_type"], "code");
            assert_eq!(query["resource"], "https://api.openai.com/v1");
            assert_eq!(
                query["scope"],
                "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct"
            );
            assert_eq!(query["code_challenge_method"], "S256");
            assert_eq!(query["state"], attempt.state.as_str());
            assert_eq!(query["nonce"], attempt.nonce.as_str());
            assert_eq!(query["redirect_uri"], attempt.redirect_uri);
            assert_eq!(
                query["code_challenge"],
                URL_SAFE_NO_PAD.encode(Sha256::digest(attempt.verifier.as_bytes()))
            );
            assert!(!query.contains_key("originator"));
            assert!(!query.contains_key("codex_cli_simplified_flow"));
            let redirect = Url::parse(&query["redirect_uri"]).unwrap();
            assert_eq!(redirect.scheme(), "http");
            assert_eq!(redirect.host_str(), Some("127.0.0.1"));
            assert_eq!(redirect.path(), "/auth/callback");
            assert_eq!(
                redirect.port(),
                Some(attempt.listener.local_addr().unwrap().port())
            );
        }
        let url = Url::parse(returning.authorization_url()).unwrap();
        let values: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(values["client_id"], "oaiapp_saved");
        assert_eq!(values["ext_agent_host_id"], host);
        assert_eq!(values["resource"], RESOURCE);
        assert_eq!(values["scope"], SCOPES);
        assert_eq!(values["id_token_hint"], "old.identity.hint");
        assert!(!values.contains_key("agent_name_hint"));
        assert!(!values.contains_key("originator"));
        assert!(!values.contains_key("codex_cli_simplified_flow"));
        let redirect = Url::parse(&values["redirect_uri"]).unwrap();
        assert_eq!(redirect.host_str(), Some("127.0.0.1"));
        assert_ne!(initial.redirect_uri, returning.redirect_uri);
        assert_ne!(initial.state.as_str(), returning.state.as_str());
        assert_ne!(initial.nonce.as_str(), returning.nonce.as_str());
        assert_eq!(values["nonce"], returning.nonce.as_str());
        assert_eq!(
            values["code_challenge"],
            URL_SAFE_NO_PAD.encode(Sha256::digest(returning.verifier.as_bytes()))
        );
    }

    #[test]
    fn identity_only_grants_are_stored_without_inference_permission() {
        let claims = IdClaims {
            iss: ISSUER.into(),
            sub: "subject".into(),
            exp: 200,
            nonce: Some("fresh".into()),
            name: None,
            email: None,
        };
        assert!(validate_identity(&claims, Some("fresh"), Some("subject"), 100).is_ok());
        assert!(validate_identity(&claims, Some("different"), None, 100).is_err());
        assert!(validate_identity(&claims, None, Some("different"), 100).is_err());
        assert!(validate_identity(&claims, None, None, 200).is_err());
        for scope in ["", "chatgpt.tokens.use.direct\u{0000}"] {
            let token: TokenResponse = serde_json::from_value(serde_json::json!({"access_token":"access", "refresh_token":"refresh", "id_token":"identity", "scope":scope})).unwrap();
            assert!(token.validate_required().is_err());
        }
        let identity_only: TokenResponse = serde_json::from_value(serde_json::json!({"access_token":"access", "refresh_token":"refresh", "id_token":"identity", "scope":"openid profile email offline_access"})).unwrap();
        assert!(identity_only.validate_required().is_ok());
        let mut grant = AuthGrant {
            account_id: account_key("oaiapp_saved", "subject"),
            display_name: None,
            email: None,
            issued_client_id: "oaiapp_saved".into(),
            subscopes: vec!["chatgpt.tokens.use.direct".into()],
            subject: "subject".into(),
            tokens: OpaqueTokenBundle {
                access_token: Zeroizing::new("access".into()),
                refresh_token: Zeroizing::new("refresh".into()),
                id_token: Zeroizing::new("id".into()),
                expires_at: 200,
            },
        };
        assert!(grant.plan_usage_enabled());
        grant.subscopes = vec!["openid".into(), "profile".into(), "offline_access".into()];
        assert!(validate_grant(&grant).is_ok());
        assert!(!grant.plan_usage_enabled());
        let restored = AuthGrant::from_vault_bytes(&grant.to_vault_bytes().unwrap()).unwrap();
        assert!(!restored.plan_usage_enabled());
        let form: HashMap<_, _> = refresh_form(&grant).into_iter().collect();
        assert_eq!(form["resource"], RESOURCE);
        assert!(!form.contains_key("scope"));
        for invalid in [DYNAMIC_CLIENT, "", "insecure id", "bad\r\nheader"] {
            assert!(validate_client_id(invalid).is_err());
        }
    }

    #[test]
    fn returning_callback_rejects_changed_registration_and_duplicates() {
        let request = |query: &str| {
            format!("GET /auth/callback?{query} HTTP/1.1\r\nHost: 127.0.0.1:9\r\n\r\n").into_bytes()
        };
        let redirect = "http://127.0.0.1:9/auth/callback";
        assert!(parse_callback(
            &request("state=good&code=x"),
            redirect,
            "good",
            "oaiapp_saved"
        )
        .is_ok());
        for query in [
            "state=good&code=x&client_id=other",
            "state=good&code=x&client_id=dynamic_agent_client",
            "state=good&state=good&code=x",
        ] {
            assert!(parse_callback(&request(query), redirect, "good", "oaiapp_saved").is_err());
        }
        assert!(parse_callback(
            &request("state=good&code=x&client_id=dynamic_agent_client"),
            redirect,
            "good",
            DYNAMIC_CLIENT
        )
        .is_err());
    }

    #[test]
    fn refresh_recovery_distinguishes_terminal_tokens_from_temporary_errors() {
        for code in [
            "invalid_grant",
            "invalid_refresh_token",
            "token_expired",
            "refresh_token_expired",
            "refresh_token_invalidated",
            "refresh_token_reused",
        ] {
            for body in [
                serde_json::json!({"error":code}),
                serde_json::json!({"error":{"code":code}}),
            ] {
                let bytes = serde_json::to_vec(&body).unwrap();
                assert!(refresh_requires_sign_in(&refresh_response_error(
                    StatusCode::BAD_REQUEST,
                    &bytes
                )));
                for status in [
                    StatusCode::TOO_MANY_REQUESTS,
                    StatusCode::SERVICE_UNAVAILABLE,
                ] {
                    assert!(!refresh_requires_sign_in(&refresh_response_error(
                        status, &bytes
                    )));
                }
            }
        }
        for body in [
            br#"{"error":"invalid_client"}"#.as_slice(),
            br#"{"detail":"refresh_token_expired"}"#,
            b"invalid_grant",
            br#"{"error":"unknown"}"#,
        ] {
            assert!(!refresh_requires_sign_in(&refresh_response_error(
                StatusCode::BAD_REQUEST,
                body
            )));
        }
        let consumed = anyhow::anyhow!("Invalid rotated response")
            .context(RefreshRequiresSignIn)
            .context("Outer context");
        assert!(refresh_requires_sign_in(&consumed));
    }

    #[test]
    fn exchange_retry_retains_only_the_issued_registration_identity() {
        let error = anyhow::anyhow!("OAuth token exchange failed (400)")
            .context(RegistrationExchangeFailed {
                client_id: "oaiapp_retry".into(),
            })
            .context("Outer request context");
        assert_eq!(registration_retry_client(&error), Some("oaiapp_retry"));
        assert!(!format!("{error:#}").contains("oaiapp_retry"));
        assert_eq!(
            registration_retry_client(&anyhow::anyhow!("Other error")),
            None
        );
    }

    #[test]
    fn account_key_never_uses_email() {
        assert_ne!(
            account_key("client", "sub-a"),
            account_key("client", "email@example.com")
        );
    }
}
