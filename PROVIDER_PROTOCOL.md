# Provider protocol (research, 2026-10-07)

This is the implementation contract for the provider boundary in `PLAN.md`
sections 14 and 38.3–38.5. It records only facts verified against current
primary documentation. Anything marked **unknown** must not be implemented by
guessing.

## Current source gap

Newer source uses a fixed public OAuth client ID rather than the dynamic flow described below. It is preserved pending integration verification. That source change does not fulfill the user's dynamic-registration decision or establish third-party eligibility. The following research remains the intended contract, not a claim of current implementation.

## Decision that gates the ChatGPT path

OpenAI's current docs describe an OSS/local dynamic-registration flow, but the
current request-client-ID page says Sign in with ChatGPT is offered to a
select group of commercial partners and directs applicants to an interest
form. The docs do not prove that this private personal app is eligible.
Partnership/interest-form submission does not itself prove approval, nor does
it provide a client ID. Do not promise ChatGPT-plan inference until OpenAI
confirms eligibility or a real end-to-end test succeeds.

Primary sources: [OSS overview](https://developers.openai.com/siwc/token-sharing-open-source),
[request a client ID](https://developers.openai.com/siwc/request-client-id),
[interest form](https://openai.com/form/sign-in-with-chatgpt-interest/).

## ChatGPT: dynamic public-client flow

### Stable identities and accounts

* Generate and persist one opaque `ext_agent_host_id` before first sign-in.
  Preferred format is an RFC 9278 JWK-thumbprint URI; supported alternative is
  one persisted `urn:uuid:<UUIDv4>`. It is an identifier, not a credential.
  A client ID may be reused across hosts for the same user/workspace, but each
  host has its own host ID. **Unknown:** whether OpenAI will approve this app’s
  requested use case and what workspace choices it will expose.
* First registration uses literal `client_id=dynamic_agent_client` and the
  consistent app name in `agent_name_hint`. The callback must return the
  issued `client_id` (for example `oaiapp_...`); save that issued ID, never the
  dynamic sentinel, and reuse it for this account/workspace.
* One user can have multiple separate registrations. Keep each issued client
  ID, validated OIDC subject, email/label, ID token, access token, refresh
  token, scopes, and expiry separate. A same email is not sufficient identity.
  Add account = new dynamic registration; switch = explicit user choice; never
  auto-rotate ChatGPT accounts to bypass a limit.

### OAuth/OIDC endpoints and authorization request

The live discovery document at
`https://auth.openai.com/.well-known/openid-configuration` currently reports:

| Item | Value |
|---|---|
| issuer | `https://auth.openai.com` |
| authorize | `https://auth.openai.com/api/accounts/authorize` |
| token | `https://auth.openai.com/api/accounts/oauth/token` |
| JWKS | `https://auth.openai.com/.well-known/jwks.json` |
| revoke | `https://auth.openai.com/api/accounts/oauth/revoke` |
| response type | `code` |
| PKCE method | `S256` |

Start a loopback listener first. Use `http://127.0.0.1:<port>/auth/callback`,
not `localhost`; keep scheme, host, path, and selected port identical in both
the authorize request and token exchange. Generate fresh cryptographic `state`,
OIDC `nonce`, and PKCE verifier for every attempt. Send this URL-encoded query
set:

```text
client_id=dynamic_agent_client                 # first registration
agent_name_hint=<actual stable app name>       # first registration only
ext_agent_host_id=<persisted host ID>
response_type=code
redirect_uri=http://127.0.0.1:<port>/auth/callback
scope=openid profile email offline_access resource.invoke chatgpt.tokens.use.direct
resource=https://api.openai.com/v1
state=<fresh random state>
nonce=<fresh random nonce>
code_challenge_method=S256
code_challenge=<base64url(SHA256(verifier)), no padding>
```

For an existing registration, replace the client ID with the saved issued ID,
omit `agent_name_hint`, and optionally send the retained ID token as
`id_token_hint` and matching saved email as `login_hint`. The hint may be
expired and only selects the account; it is not authentication. Keep each
attempt’s pending account/client/redirect tuple until completion.

### Callback, token exchange, and validation

Accept only a loopback callback for the pending attempt. On
`error=access_denied`, validate `state`, stop, and do not exchange a code. A
successful new registration returns `code`, `state`, `client_id` and possibly
`scope`; returning sign-in may omit `client_id`, in which case use the pending
issued ID. Reject a supplied client ID that differs from the pending one.

Exchange with `application/x-www-form-urlencoded`:

```text
POST https://auth.openai.com/api/accounts/oauth/token
grant_type=authorization_code
client_id=<issued client ID>
code=<callback code>
code_verifier=<PKCE verifier>
redirect_uri=<exact callback URI>
resource=https://api.openai.com/v1
```

No client secret or partner API key is used in this public-client flow. On
HTTP 200, expect `access_token`, `refresh_token`, `id_token`, `token_type`,
`expires_in`, `scope`, and `earliest_refresh_at`. Current docs state access
tokens are one hour and refresh tokens are 30 days; refresh rotates the
refresh token.

Validate the ID token before activation: signature against the discovered
JWKS, `iss=https://auth.openai.com`, `aud=<issued client ID>`, `exp`, and the
fresh nonce. Use validated `sub` as account identity. Require returned scope
`chatgpt.tokens.use.direct` before plan inference; a valid ID token alone is
not enough. Verify returning sign-in subject matches the selected account.

Refresh just before expiry, serializing refresh per credential set:

```text
POST https://auth.openai.com/api/accounts/oauth/token
grant_type=refresh_token
client_id=<issued client ID>
refresh_token=<current refresh token>
resource=https://api.openai.com/v1
```

Omit `scope` to retain the grant. Replace access token, expiry, granted scope,
and rotated refresh token atomically. On `invalid_grant`,
`invalid_refresh_token`, `token_expired`, `refresh_token_expired`,
`refresh_token_invalidated`, or `refresh_token_reused`, mark signed out and
start OAuth again with the saved issued client ID. Do not loop refreshes.

Revoke on sign-out using the discovered revocation endpoint, form-encoded:
`token=<refresh_token>&token_type_hint=refresh_token&client_id=<issued ID>`.
HTTP 200 is success, including an already-invalid token. Clear local tokens
afterward; a network/5xx failure may retry with bounded backoff but must not
claim remote revocation was confirmed.

Primary sources: [registration/sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in),
[accounts/sessions](https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions),
[token reference](https://developers.openai.com/siwc/token-sharing-open-source/token-reference),
[OIDC discovery](https://auth.openai.com/.well-known/openid-configuration).

## ChatGPT inference

### Model discovery

```text
GET https://api.openai.com/v1/models
Authorization: Bearer <access token>
```

The current plan-usage example returns a `models` array. Show entries with
`visibility == "list"`, display `display_name`, and send selected `slug` as
`model`. Refresh this catalog after switching accounts. Do not hardcode a
model name.

### Request and permitted body

```text
POST https://api.openai.com/v1/responses
Authorization: Bearer <access token>
Content-Type: application/json
Accept: text/event-stream
{
  "model": "<catalog slug>",
  "input": [<complete bounded history>],
  "instructions": "<optional developer instructions>",
  "store": false,
  "stream": true
}
```

For this route, send the context every time in `input`; do not send
`previous_response_id`, `conversation`, or `truncation`. Use `instructions` or
developer messages; an explicit `{"type":"message","role":"system"}` is
rejected. Omit `background`, `max_output_tokens`, `max_tool_calls`, `metadata`,
`moderation`, `multi_agent`, `prompt`, `prompt_cache_retention`,
`safety_identifier`, `temperature`, `top_logprobs`, `top_p`, and `user`.
Bound history locally to the product token budget, retaining the current
request and only the selected notes/transcript context. This avoids relying
on server-side conversation storage.

Text, images, and files are supported only where the selected model accepts
them. This flow does not support audio/video input, Files upload API,
transcription API, image generation, file search, Code Interpreter, native
computer use, hosted MCP/connectors, Responses `tool_search`, or top-level
`programmatic_tool_calling`. **Unknown:** per-model modality limits must be
read from the model catalog/actual response; do not infer them from provider
brand.

### SSE parser and terminal behavior

Parse SSE events, not arbitrary newline-delimited JSON. Append text only from
`response.output_text.delta` (`delta`). Treat these as terminal and distinct:

| Event | Handling |
|---|---|
| `response.completed` | success; read final response and `usage`; close normally |
| `response.failed` | terminal failure; read `response.error.code/message/param`; never fail over silently after any stream began |
| `response.incomplete` | terminal incomplete result; preserve partial text and reason |
| connection close before a terminal event | interrupted/transport error; preserve partial text |

Success requires `response.completed`, not merely one text delta. Usage is
provider response data; persist `input_tokens`, `output_tokens`, and cached
input tokens only when present. Direct admission can fail before streaming
with 401/403/503 and a nonstandard `{"detail":"..."}` body. Structured
codes include `subscription_sharing_user_not_eligible` (403),
`subscription_sharing_usage_limit_exceeded` (429),
`subscription_sharing_usage_unavailable` (503),
`subscription_sharing_unsupported_capability` (400),
`subscription_sharing_route_not_supported` (403),
`subscription_sharing_invalid_user` (401), and
`chatpass_v2_scope_not_authorized`/`chatpass_v2_invalid_authorization_context`
(403). Preserve HTTP status, request ID, body shape, code, and param without
logging credentials. Only pre-first-token failure may enter API-key fallback.

Primary sources: [models/inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference),
[preview limits](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations),
[errors/recovery](https://developers.openai.com/siwc/token-sharing-open-source/errors-and-recovery).

## DeepSeek and Gemini OpenAI-compatible providers

Both are API-key providers behind one `OpenAICompatProvider`. Normalize a
user-supplied base URL by requiring HTTPS, stripping query/fragment, trimming
trailing `/`, and appending the protocol path exactly once. Keep the
preconfigured defaults below; user edits must not silently change provider
identity or send a key to a different origin. Validate the final origin and
show it before saving.

| Provider | Default base URL | model discovery | inference |
|---|---|---|---|
| DeepSeek | `https://api.deepseek.com` | `GET /models` | `POST /chat/completions` |
| Gemini | `https://generativelanguage.googleapis.com/v1beta/openai/` | `GET /models` | `POST /chat/completions` |

Use only `Authorization: Bearer <API key>` and `Content-Type: application/json`;
do not put keys in URLs. Discovery is authenticated and should preserve each
provider’s returned model metadata instead of hardcoding names. The model is a
selected returned ID. **Unknown:** neither provider’s compatibility docs make
the full intersection of request fields identical; use a narrow common body.

Common request:

```json
{
  "model": "<selected model id>",
  "messages": [
    {"role":"system","content":"<optional instruction>"},
    {"role":"user","content":"<bounded compiled context>"}
  ],
  "stream": true,
  "stream_options": {"include_usage": true}
}
```

The documented streamed Chat Completions wire format is `Content-Type:
text/event-stream`, `data: {"id","choices":[{"delta":{"content"},
"finish_reason"}],"model",...}` chunks, a final usage-bearing chunk where
supported, then `data: [DONE]`. Normalize text from `choices[0].delta.content`.
Terminal success requires a final `finish_reason` and `[DONE]`; a disconnect
before either is an error. Usage is provider-specific; DeepSeek documents
`prompt_tokens`, `completion_tokens`, `total_tokens`, cached-token details,
and reasoning tokens. Gemini supports `stream_options.include_usage` in its
compatibility examples, but **unknown:** exact usage fields/cadence should be
handled as optional until observed.

### Provider-specific input

* DeepSeek’s current vision guide documents user-message `image_url` content
  parts: HTTPS URLs or base64 data URLs; JPEG/PNG/GIF/WebP; 48 MiB request
  body, 32 MiB per inline/external image, 64 MiB via Files API, 600 images per
  request. Image blocks in system/assistant messages return 400. This product
  should send screenshots as user content; if the selected discovered model
  rejects images, send OCR text only. DeepSeek’s compatibility docs also show
  `deepseek-flash`; it is not a hardcoded product default.
* Gemini’s official compatibility guide documents the same `image_url` shape
  with base64 examples, streaming, function calling, and model listing at
  `GET https://generativelanguage.googleapis.com/v1beta/openai/models`.
  Capability is model-dependent; filter/offer image input only after catalog
  metadata or a user-approved test confirms it. Do not send Gemini-only
  `extra_body.google` thinking controls to DeepSeek. Gemini’s compatibility
  layer says unsupported parameters may be silently ignored, so only send
  fields the common contract needs.

Primary sources: [DeepSeek Chat Completions](https://api-docs.deepseek.com/api/create-chat-completion),
[DeepSeek models](https://api-docs.deepseek.com/api/list-models),
[DeepSeek vision](https://api-docs.deepseek.com/guides/vision),
[DeepSeek errors](https://api-docs.deepseek.com/quick_start/error_codes),
[Gemini OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai).

### Errors and failover

Classify HTTP 401/403 as auth/permission, 408/timeout as transport,
429 as rate/capacity, and 5xx as provider/server. DeepSeek documents 400
invalid format, 422 invalid parameters, 429 rate limit, 500 server error,
and 503 overloaded. Gemini’s compatibility layer does not promise DeepSeek’s
error schema; preserve status and provider body as diagnostic text.

Fail over only when the active request fails before the first output token and
the next provider is explicitly enabled and consented: usage/limit, 401/403,
429, 5xx, or no first token within 10 seconds. Once any token has been shown,
never switch silently: preserve partial output and offer “Retry with …”.
Never retry a malformed request, unsupported capability, or consent failure
without changing the request or asking for consent.

## Permissions, consent, usage, and storage

OAuth scopes are not permission to upload arbitrary product data. Before each
remote request, the app’s explicit settings/meeting permission must authorize
the selected provider and payload:

| Payload | Default rule |
|---|---|
| sales notes/transcript/context | explicit meeting/user permission; show provider before sending |
| screenshot/image | separate explicit screenshot action; send only to an image-capable selected model |
| grounded statistics | allow only compiled source-backed context; show provenance; no fabricated quota |
| API-key fallback | separate provider consent, because data leaves for that company |
| ChatGPT plan usage | explicit OAuth grant `chatgpt.tokens.use.direct`; no inference without it |

Credential storage is Rust-only. On Windows use Credential Manager/DPAPI-backed
storage (for example the existing `keyring` direction), with one record per
ChatGPT issued client/account and one API-key record per provider/base URL.
SQLite may store only `credential_ref`, provider, label, email, status,
selected model, base URL, and redacted usage metadata; never plaintext API
keys, OAuth access/refresh/ID tokens, authorization codes, PKCE verifiers,
or `id_token_hint`. React receives status/label/email as appropriate, never
secrets. Structured logs must redact `Authorization`, query strings containing
tokens/hints, request bodies, and provider key material; record only safe
status/code/request ID/timings. Serialize refresh and credential replacement
atomically.

## Remaining unknowns / implementation gates

1. **OpenAI eligibility:** current public docs do not prove this private app
   can dynamically register or use ChatGPT plan inference; do not claim a
   successful product path until approval or a real test.
2. **OpenAI registration policy:** docs prove one issued client can serve
   multiple hosts for the same user/workspace and separate registrations can
   represent multiple accounts; they do not promise arbitrary workspace
   switching or unlimited registrations.
3. **Model capabilities and limits:** discover per account/model and handle
   rejection; do not infer context, image, or tool support from model names.
4. **Gemini/DeepSeek compatibility edges:** exact error-body schemas and all
   ignored/unsupported OpenAI fields are not a shared contract. Keep the
   adapter narrow and make capability tests/diagnostics visible.

This file is research only. No client was registered, no login was performed,
no API key was requested, and no provider code was added.

Implementation note, 2026-10-07: token validation now requires a replacement refresh token for both code and refresh responses. A missing replacement is rejected; the old token is not substituted. The official token reference and accounts/sessions pages were checked again. Refresh serialization and session UI are still pending integration.
