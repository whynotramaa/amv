# Requirements checklist

Review basis: revised `PLAN.md` (3,035 lines in the live worktree), current source excluding
`node_modules/` and `target/`, and `PROGRESS.md`. This file is the review
record, not a completion claim.

Status: `IMPLEMENTED` means source and a runnable check support the claim;
`PARTIAL` means only part is present; `PENDING` means no implementation;
`RUNTIME-UNVERIFIED` means code exists but the required Windows/device/user
proof is missing. Planned code, a schema sketch, a URL, or an example is not
evidence of completion.

Evidence commands used repeatedly below:

- `E1`: `npm run build`
- `E2`: `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --locked`
- `E3`: `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`
- `E4`: `cargo xwin check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --locked -j 2`
- `E5`: `rg -n "audio|STT|ChatGPT|OAuth|memory|meeting|inference|installer|Poppins|Geist|shadow" src src-tauri scripts package.json`
- `E6`: inspect `src-tauri/src/audio.rs`, `src-tauri/examples/audio_probe.rs`, and run the Windows probe with `cargo run --example audio_probe --manifest-path src-tauri/Cargo.toml -- 5` on Windows.
- `E7`: `rg -n "shadow|Segoe|Sparkles|Icon|Poppins|Geist" src src-tauri/tauri.conf.json`

## Agreed decisions and overrides

- [ ] `RUNTIME-UNVERIFIED` PLAN 19-20, 31: private-use Windows x64 product, 8 GB RAM, CPU-only, no dedicated GPU, unload idle models, bound PCM queues, and measure concurrent model memory. Evidence: CPU-only Parakeet TDT model, model unload on stop, bounded capture/speech queues, and build-time model checksum exist in `src-tauri/src/speech.rs`, `src-tauri/src/audio.rs`, and `scripts/prepare-model.mjs`; 8 GB and Windows runtime measurements are not proved.
- [x] `IMPLEMENTED` PLAN 21, 24-25: persisted `On hotkey`/`Automatically`, three response modes, non-empty custom validation, distinct default shortcuts, and next-meeting copy. Evidence: `src/App.tsx:80-97`, `src-tauri/src/store.rs:32-64`, E1/E2. Meeting-start snapshot is implemented; actual send behavior remains pending.
- [ ] `PARTIAL` PLAN 22-23, 27: finalized system-only message queueing, ordering/provenance/retry state, local history/context budgeting, disclosure of incomplete context, automatic batching, hotkey send, and unsent preservation. Evidence: meeting-start settings snapshots, source-tagged durable finalized transcript rows, bounded transcript state, and system-only delivery-cursor store methods exist in `src-tauri/src/meeting.rs` and `src-tauri/src/store.rs`; bounded pure context compiler is now implemented with whole-current-message preservation, explicit microphone opt-in, source provenance and omission metadata; durable request identity, frozen segment batches, no-output retry state, delivered cursors and bounded local history are implemented; schema-6 streamed recovery, sequential new-only SYSTEM dispatch, frozen UTF-8 slices, manual high-water batching, fresh automatic consent and streaming answer UI are implemented; live Windows/provider acceptance remains pending.
- [ ] `PARTIAL` PLAN 26: local onboarding before sign-in, with capture, transcription, persistence, and search available offline. Evidence: consented local meeting setup, Windows capture/STT, schema-6 transcript and request persistence, and transcript FTS exist; saved-meeting list and literal transcript search UI are implemented; Windows runtime proof remains pending.
- [ ] `PARTIAL` PLAN 28: consented sales meeting notes, grounded statistical answers with provenance, explicit capture/remote-send control, and no covert-monitoring/interview-cheating framing. Evidence: sales-oriented copy, explicit per-meeting capture permission, local finalized notes, and no-cheating scope exist in `src/App.tsx`, `src-tauri/src/meeting.rs`, and PLAN 217-222; grounded statistics, answer provenance, and remote-send control are not complete.
- [ ] `PARTIAL` PLAN 29: Poppins and Geist Mono assets/tokens, thin borders, and `shadow: false` now exist. The override still requires the same system across future screens. Current evidence: `src/styles/tokens.css:1-56`, `src/styles/app.css:21-30`, `src-tauri/tauri.conf.json:23-29`, `src/components/primitives.tsx:3-15`; Poppins and Geist Mono are bundled; the Segoe UI fallback is sans-serif, not a serif violation. Inline SVG icons are original custom icons and contain no Sparkles. Current popover verification passed; future Control center and answer views remain pending.
- [ ] `PARTIAL` PLAN 30, 34: editable provider base URLs, secure user API keys, DeepSeek/Gemini preconfiguration and discovery, explicit provider-sharing consent, and verified eligible ChatGPT client configuration. Evidence: provider validation/defaults, Connections UI/native commands, DPAPI vault, model discovery, OAuth, and streaming clients exist in `src/components/Connections.tsx`, `src-tauri/src/providers.rs`, `credentials.rs`, `auth.rs`, and `inference.rs`; consented quota-only pre-output fallback is wired; eligible sign-in and real end-to-end inference remain unverified.
- [ ] `PARTIAL` PLAN 31: final working Windows installer, end-to-end ChatGPT sign-in, separate voice transcription, fallback inference, multi-pass correctness/integration/UI-accessibility/Windows-runtime checks, and measured low-RAM behavior. Evidence: root builds `dist/Harness-Setup-x64.exe`; separate local speech and provider streaming code plus Linux checks exist. Clean Windows install/runtime, eligible sign-in, fallback wiring, and low-RAM proof are absent.
- [ ] `PARTIAL` PLAN 32-36: overlay privacy/control, multiple ChatGPT accounts, DeepSeek/Gemini fallbacks, screenshot input, and streaming Markdown/code. Evidence: account/provider configuration, secure credential storage, model discovery, and streaming parsers exist; explicit account switching, ChatGPT model discovery/selection, serialized refresh, cancellable reauthorization and sign-out are implemented in source; fallback dispatch and safe GFM/Shiki rendering are implemented; overlay controls, screenshot/OCR and usage pages remain pending. Eligible live session behavior remains runtime-unverified.
- [ ] `PARTIAL` PLAN 37: package foundation, then prove simultaneous local STT and authorized streaming inference before embeddings/reranking. Evidence: integrated Windows-only capture plus CPU local STT and a streaming provider client exist; dispatch integration exists; authorized live simultaneous proof remains pending.

## 0. Executive definition

- [ ] `PARTIAL` PLAN 45-65: Windows-native tray agent with separate system/mic near-real-time transcription, evolving meeting state, personal memory, global-shortcut popover, local documents, retained facts, and answers grounded in all listed context types. Evidence: integrated capture/speech/meeting state, tray, and popover exist; manual memories and bounded enabled-only lexical retrieval are integrated; document lexical retrieval exists; full grounded-answer acceptance remains pending.
- [ ] `PARTIAL` PLAN 67-73: official Sign in with ChatGPT eligible-plan inference; local capture, transcription, storage, meeting state, ingestion, retrieval, embeddings, memory, settings, and context assembly; remote requests limited to compiled text plus request. Evidence: OAuth/vault/streaming source and local capture/transcription/storage/meeting state exist; bounded context compilation exists; manual memory and lexical retrieval are implemented; eligible live inference, ingestion, hybrid retrieval and embeddings remain pending.

## 1. Non-negotiable product requirements

### 1.1 Windows application, PLAN 84-110

- [ ] `RUNTIME-UNVERIFIED` Windows 11 x64 launch target and Windows 10 22H2 support where APIs/WebView2 work. ARM64 remains portable but not a launch target. Evidence: E4 is compilation only; no Windows runtime.
- [ ] `RUNTIME-UNVERIFIED` Start-menu launch, optional login startup, tray presence, no full-size main-window dependency, global shortcut, fast show/hide, multi-monitor/DPI behavior, audio-device recovery, sleep/wake recovery, clean shutdown. Evidence: desktop, integrated capture/STT, meeting shutdown, startup stale-meeting recovery, and tray Quit cleanup exist; Windows/device/runtime acceptance remains pending.

### 1.2 Local-first, PLAN 112-140

- [ ] `PARTIAL` Local microphone/system PCM, VAD, resampling, STT, meeting state/cache, documents, indexes, memories, records, retrieval, context selection, settings/logs, and OS-backed credentials. Evidence: PCM capture, VAD, CPU STT, meeting state/cache, transcript FTS, settings/logging, OAuth, and DPAPI vault exist; manual memory, lexical retrieval and bounded context selection exist; documents are indexed locally; hybrid retrieval remains pending.
- [ ] `PARTIAL` Raw audio never uploads; remote receives only compiler-selected text and request. Evidence: capture and inference are separate modules and inference sends JSON text, not audio; the bounded pure compiler exists, the native dispatch boundary sends only compiled text JSON. Live acceptance remains pending.
- [ ] `PARTIAL` Final ChatGPT inference is the only deliberately remote/local-network-dependent final step. Evidence: official OAuth and streaming inference clients exist; the final dispatch path is integrated in source and runtime-unverified.

### 1.3 Installation, PLAN 142-186

- [ ] `PENDING` `Harness-Setup-x64.exe` installs on a clean machine for a non-technical user, including sign-in and microphone permission. Evidence: `scripts/package.ps1:1-14` can package a foundation installer but `PROGRESS.md` says clean-machine acceptance is pending.
- [ ] `RUNTIME-UNVERIFIED` Target needs none of Rust/Cargo/Node/npm/Python/Git/CMake/MSVC/Whisper/FFmpeg/SQLite/server/model service/Docker. Evidence: the installer bundles the model, manifest and license; WebView2 offline installation is configured; clean-machine dependency proof is pending.
- [ ] `PARTIAL` Runtime components may be bundled or installed through controlled first run. NSIS, offline WebView2 installer, model, manifest, license, and migrations are configured; clean-machine behavior is unverified. Evidence: `src-tauri/tauri.conf.json`, `scripts/prepare-model.mjs`, and `PROGRESS.md`.
- [ ] `PARTIAL` Canonical offline NSIS package includes offline WebView2, STT runtime/model, migrations, and assets; a bootstrap package cannot replace it. Evidence: current build includes the model, manifest/license, migrations and assets; offline WebView2 configuration is restored. Windows runtime acceptance remains unverified.

### 1.4 Server boundary, PLAN 188-204

- [x] `IMPLEMENTED` No hosted Harness backend, Postgres, Redis, remote vector DB, remote transcription, hosted memory, Harness account server, or Docker is present. Evidence: source/dependency inventory, `package.json`, `src-tauri/Cargo.toml`, E5. This is absence evidence only; it does not complete the local engines.
- [ ] `PARTIAL` Local desktop process is the application backend and ChatGPT is the external inference dependency. Evidence: Tauri commands own capture, meeting state, store, OAuth, vault, providers, and streaming; conversation dispatch is integrated in source and runtime-unverified.

### 1.5 Privacy and consent, PLAN 206-226

- [ ] `PARTIAL` Documented `WDA_EXCLUDEFROMCAPTURE`, taskbar/Alt+Tab hiding, clear recording/transcription state and controls. Evidence: `contentProtected`, `skipTaskbar`, explicit permission UI, meeting states, and transcript UI exist; documented Windows capture-affinity/Alt+Tab runtime behavior remains unverified.
- [x] `IMPLEMENTED` Proctoring/anti-cheat defeat and secretly retained audio are out of scope. Evidence: scope decision in PLAN 217-222; no such code found, E5.
- [ ] `PARTIAL` UI must surface the user's responsibility for meeting consent/recording law and enforce explicit capture/remote-send control. The UI requires per-meeting permission before local capture and requires a configured provider before enabling the composer and fresh automatic-send consent per meeting. Native consent/runtime acceptance remains pending. Evidence: `src/App.tsx`, `src-tauri/src/meeting.rs`.

## 2. Product mental model, PLAN 227-273

- [ ] `PARTIAL` Five local engines connected by an event bus: Windows audio, local speech, meeting state, memory/retrieval/recent transcript, and context compiler before ChatGPT. The diagram is illustrative, but each named behavior is normative. Evidence: capture, speech, meeting/store, and Tauri events exist; context compiler and final dispatcher exist; manual memory and enabled-only lexical retrieval exist; hybrid retrieval remains pending.

## 3. First launch, PLAN 275-353

- [ ] `PENDING` Short deterministic first-run screens: Harness identity/wordmark, explanation, `Continue with ChatGPT`, no separate Harness account. Evidence: current app is settings/assistant only and reports ChatGPT unavailable, `src/App.tsx:54-74`.
- [ ] `PARTIAL` Official browser Sign in with ChatGPT flow with installation `ext_agent_host_id`, dynamic registration, callback validation, local profile metadata, Windows secure credentials, and no React tokens. Evidence: dynamic OAuth, PKCE/state/nonce/JWKS/callback validation, account metadata, and DPAPI vault exist in `src-tauri/src/auth.rs`, `store.rs`, and `credentials.rs`; eligible live sign-in is unverified.
- [ ] `PARTIAL` Audio screen detects/tests default microphone and system output and visibly shows both streams. Evidence: product capture resolves both sources and the meeting UI labels SYSTEM/MIC; a dedicated device-test screen and Windows runtime proof are pending.
- [ ] `PARTIAL` Local AI setup verifies runtime, model checksum, writable DB, and search index, with no download when bundled. Evidence: pinned model preparation/checksum and local store migrations exist; onboarding/runtime/search verification is incomplete.
- [x] `IMPLEMENTED` Default overlay shortcut `Ctrl+Space`, send shortcut `Ctrl+Shift+Enter`, both configurable and distinct. Evidence: `src-tauri/src/store.rs:19-29,50-62`, `src/App.tsx:91-93`, E1/E2.
- [ ] `PARTIAL` Claimed fallback `Ctrl+Shift+Space` on conflict and send behavior are not implemented. Current conflict handling reports an error and keeps tray access; the send shortcut dispatches only new finalized SYSTEM speech; native shortcut/runtime acceptance remains pending. Evidence: `src-tauri/src/desktop.rs:51-60,231-236`, `src/App.tsx`.
- [ ] `RUNTIME-UNVERIFIED` Finish enters tray idle. The source has a tray and an idle menu item, but Windows runtime behavior is unchecked. Evidence: `src-tauri/src/desktop.rs:244-283`, `PROGRESS.md`.

## 4. Day-to-day interaction, PLAN 354-490

- [ ] `PARTIAL` Tray menu has Open, settings, idle listening label, and Quit; Memory opens a separate management window; Start/Stop meeting, device labels and full listening state remain absent from the tray. Evidence: `src-tauri/src/desktop.rs:244-279`.
- [ ] `RUNTIME-UNVERIFIED` Closing hides rather than terminates; Quit terminates capture/STT/DB/process. Hide, worker joins, final draining, model unload, and tray shutdown are coded; Windows runtime proof remains pending. Evidence: `src-tauri/src/desktop.rs`, `src-tauri/src/meeting.rs`, `src-tauri/src/speech.rs`, `PROGRESS.md`.
- [ ] `RUNTIME-UNVERIFIED` Global shortcut toggles overlay, Esc hides, works over other apps, focuses input only for overlay shortcut, and does not create a taskbar window. Evidence: registration/toggle code `src-tauri/src/desktop.rs:51-60,171-212`; no Windows runtime test, E4/PROGRESS.
- [ ] `PARTIAL` Frameless always-on-top rounded draggable resizable popover, remembered/clamped position, DPI/multi-monitor safety, keyboard/mouse interaction, and fast show/hide. Window sizing/frameless/always-on-top/resizable and position clamp exist; DPI, drag/resize behavior and runtime speed are unverified. Evidence: `tauri.conf.json:13-31`, `src-tauri/src/window.rs:1-34`, `src-tauri/src/desktop.rs:110-145`.
- [ ] `PENDING` Background/text opacity, click-through, keyboard move/resize, capture/taskbar/Alt+Tab exclusion, and override-compliant border-only visual. Evidence: opacity/click-through/move/resize controls are absent; custom icons, Poppins/Geist Mono, thin borders, and no CSS/native shadows are implemented, while Windows exclusion behavior remains unverified.
- [ ] `PARTIAL` Suggested dimensions are configured as 520/420-720 logical width, but state heights and actual answer states do not exist. Evidence: `tauri.conf.json:17-22`, `src/App.tsx`.
- [ ] `PARTIAL` Idle/listening/asking/streaming popover states, live transcript, answer controls, and error states. Current UI has idle/starting/active/stopping/error states, finalized source-labeled transcript, and native preparing/streaming/completed/partial/error answers with cancellation and durable no-output retry; Windows rendering remains unverified. Evidence: `src/App.tsx`, `src-tauri/src/meeting.rs`.

## 5. Meeting lifecycle, PLAN 491-548

- [ ] `PARTIAL` Explicit IDLE/STARTING/ACTIVE/PAUSED/STOPPING/COMPLETED abstraction and reliable manual start/stop. Evidence: native store states and meeting controller cover starting/active/paused/stopping/completed/interrupted; the UI maps idle/error and Windows runtime reliability is unverified.
- [ ] `PARTIAL` Start creates meeting, resolves devices, starts both capture streams/STT/bus/state accumulator, and shows listening. Evidence: `src-tauri/src/meeting.rs` starts integrated `SpeechSession`, persists the meeting snapshot, and emits state; Linux returns explicit WASAPI unsupported, and Windows proof is pending.
- [ ] `PARTIAL` Stop drains/finalizes/persists/releases/marks complete and queues final enrichment without delaying release. Evidence: meeting stop drains final events, persists finals, joins workers, unloads the model, and marks the meeting complete; final enrichment is absent; dispatch is implemented separately.
- [ ] `PARTIAL` Crash preserves produced transcript. Evidence: finalized events persist incrementally and worker panic/recovery paths preserve produced rows; full crash/restart acceptance is unverified.

## 6. Audio, PLAN 550-680

- [ ] `PARTIAL` Separate `system` and `microphone` streams with bounded queues and source distinction. Evidence: integrated `CaptureSession` has separate source workers, fair selection, bounded channels, drop metrics, and source-tagged speech events in `src-tauri/src/audio.rs` and `speech.rs`; Windows runtime proof is pending.
- [ ] `RUNTIME-UNVERIFIED` Windows WASAPI default render loopback and selected microphone enumeration/capture. Evidence: product capture enumerates default loopback and selected microphone in `src-tauri/src/audio.rs`; Linux explicitly reports unsupported and Windows hardware acceptance is pending.
- [ ] `PARTIAL` Device format negotiation, resampling/downmix to PCM float32/16 kHz/mono, endpoint changes, Bluetooth/headset recovery, no raw-audio disk writes, and user-selectable non-default microphone. Evidence: WASAPI conversion and selected microphone path exist; endpoint/Bluetooth recovery, device UI, and Windows proof remain pending.
- [ ] `PARTIAL` Capture callback avoids SQLite/network/React/inference/disk/long mutex waits; bounded backpressure/drop metrics exist in the probe. Evidence: capture workers use bounded queues and metrics and the integrated speech worker keeps blocking decode off callbacks; two-hour proof is pending.
- [ ] `PARTIAL` Local VAD detects onset/end, trims silence, keeps pre-roll, and merges short pauses with configurable internal parameters. Evidence: `webrtc-vad` segmentation, pre-roll, silence, minimum speech, and maximum utterance settings exist in `src-tauri/src/speech.rs`; measured runtime acceptance is pending.
- [ ] `PENDING` Survives unplugged mic, Bluetooth/output/USB changes, sleep/wake, exclusive conflicts, denied permission, and no system audio; mic failure leaves system transcription running with a warning. Evidence: no recovery path or runtime tests.

## 7. Local STT, PLAN 681-772

- [ ] `PARTIAL` Replaceable Rust speech-engine abstraction with partial/final event output and no Whisper coupling. Evidence: `SpeechSession` emits source-tagged partial/final/error events; its current implementation uses Whisper directly, so the replaceable abstraction remains incomplete.
- [ ] `PARTIAL` Redistributable Windows x64 CPU-capable native Whisper-compatible runtime, optional acceleration, partial decoding, and no Python runtime. Evidence: current CPU-only Sherpa ONNX/Parakeet TDT runtime and pinned bundled model replace whisper-rs. A Linux reference WAV probe passes at ~1.05GiB peak RSS; live partial updates, optional acceleration and Windows runtime proof remain pending.
- [ ] `RUNTIME-UNVERIFIED` One default model selected for measured English accuracy/real-time performance; model hidden from onboarding and unload policy measured. Evidence: pinned English `base.en-q5_1` model, checksum, isolated decode probe, and stop-time unload exist; Windows/8 GB measurement is pending.
- [ ] `PARTIAL` Segment fields, mutable partials, immutable finals except explicit correction, stable transcript UI, and measured latency instrumentation. Evidence: speech events carry IDs, source, timestamp, revision, and text; finalized rows persist through schema 4 and the UI labels them. Full partial UI and latency instrumentation remain pending.

## 8. Event bus and concurrency, PLAN 774-828

- [ ] `PARTIAL` Event families for audio, speech, transcript, meetings, state, retrieval, context, inference, device changes, and errors. Evidence: audio/speech/transcript/meeting/state/error events and inference stream events exist; retrieval/context/device-change event families are not implemented.
- [ ] `PARTIAL` Tokio orchestration without putting blocking real-time callbacks on async executor; separate system/mic capture workers, STT workers, persistence/retrieval/context/auth/client workers. Evidence: separate capture/STT workers, native commands, async auth/inference, and durable store exist; context compilation and dispatch are integrated; retrieval workers remain pending.

## 9. Transcript storage, PLAN 830-851

- [x] `IMPLEMENTED` SQLite local store with WAL, foreign keys, busy timeout, transactional initial migration, and FTS5 finalized-text triggers. Evidence: `src-tauri/src/store.rs:71-95`, `src-tauri/migrations/001_initial.sql:1-52`, E2.
- [ ] `PARTIAL` Required indexes, full incremental finalized-segment persistence, and production transcript repositories. Evidence: schema 4 adds event identity/order indexes; meeting code incrementally writes finalized segments and provides bounded chronological paging and system-only delivery cursors. Search/repository scope beyond meetings remains pending.

## 10. Meeting state, PLAN 853-926

- [ ] `PARTIAL` Recent finalized/partial ring buffer preserving source/timestamps and token-adjusted recent context, with retrieval for older meeting moments. Evidence: bounded source/timestamp transcript state and durable chronological paging exist; token budgeting and older-moment retrieval do not.
- [ ] `PARTIAL` Rolling state triggers and per-meeting send policy: no background cloud request in hotkey mode, sequential automatic batches, graceful local degradation. Evidence: settings snapshot preserves send mode and capture continues locally; sequential dispatch, fresh remote consent, manual high-water batching, failure pause and durable retry are wired; live acceptance remains pending.
- [ ] `PARTIAL` End-of-meeting transcript, summary, decisions, action items, entities, memory links, and timestamps. Evidence: finalized transcript and meeting timestamps persist; summaries, decisions, entities, memory links, and enrichment are absent.

## 11. Personal memory, PLAN 928-1050

- [ ] `PARTIAL` Structured, semantic, and episodic memory categories covering projects, people, organizations, preferences, documents, notes, prior meetings, decisions, actions, and changes. Evidence: schema7 manual memories represent nine structured categories with project labels and text; derived episodic memories, imported semantics and separate entity relationships remain pending.
- [ ] `PARTIAL` Provenance-aware candidate/accepted/rejected/superseded lifecycle; no silent permanent memory conversion. Evidence: entries are explicit manual inputs and enabled only by user choice; candidate/rejected/superseded lifecycle and derived artifacts remain pending.
- [ ] `PARTIAL` Provenance answers source, creation time, originating meeting/document/manual entry, and supersession. Evidence: manual memories persist source and creation/update timestamps; originating document/meeting relationships and supersession remain pending.
- [ ] `PARTIAL` Local hybrid retrieval with entity/project detection, FTS/BM25, local vectors, structured lookup, reranking, and budgeted candidates; lexical fallback when embeddings unavailable and automatic model install. Evidence: manual memory FTS5/BM25, title/project weights, enabled-only top8/16KiB lexical retrieval and compiler budgeting exist; entity detection, vectors, structured lookup and reranking remain pending.
- [ ] `PARTIAL` Normal-size memory management view with search, inspect/provenance, add/edit/delete, entities/projects, document import, and meeting-history deletion. Evidence: tray/book icon open a separate 900×700 management window with search, provenance and manual CRUD; draft-safe close destroys the WebView. Separate document management is implemented. Entities and meeting-history deletion remain pending.

## 12. Document ingestion, PLAN 1052-1099

- [ ] `PARTIAL` Local ingestion of `.md`, `.txt`, meaningful `.json`, reliable bundled `.pdf`, and optional later `.docx`; extraction, normalization, chunking, metadata, FTS, embeddings, ready state. Evidence: documents.rs handles bounded UTF-8 TXT/Markdown/JSON extraction, normalization and hashing; schema8 chunks/FTS and dedicated UI/native picker are integrated. PDF/DOCX and embeddings remain pending.
- [x] `IMPLEMENTED` No source upload; store path/hash/modified time/text/chunks/index version and offer re-index when source changes. Evidence: schema8 path/hash/modified time/index version and chunks, explicit changed-source check/reindex, reference/copy policy, default-off sharing and original-safe deletion are implemented. Native acceptance remains unverified.

## 13. Context compiler, PLAN 1101-1180

- [ ] `PARTIAL` Compiler accepts query, recent transcript, meeting state/retrieval, memory, documents, and session history and emits a bounded request. Evidence: context.rs compiles current request, recent transcript, bounded history and provenance records; enabled-only memory/document lexical retrieval is integrated.
- [ ] `PARTIAL` Priority order, explicit model-aware token budgets, response reserve, and disclosure when current content cannot fit. Evidence: conservative UTF-8 byte estimates, response reserve, omission metadata and whole-current-message rejection exist. Discovered model-specific context limits remain pending.
- [ ] `PARTIAL` Provenance labels distinguish recent meeting, personal memory, and other sources; inspectable sent-context view later stores compiled request with answer. Evidence: compiler provenance and omission metadata exist; manual-memory provenance is integrated; schema9 bounded prepared-request storage and on-demand inspection are integrated; provider receipt and runtime acceptance are unverified.
- [ ] `PARTIAL` Imported documents/transcripts are untrusted context and cannot override application/system instructions. Evidence: trusted product instructions distinguish quoted source JSON as untrusted data; document evidence is integrated as untrusted JSON with bounded excerpts and provenance.

## 14. ChatGPT authentication and inference, PLAN 1182-1284

- [ ] `PARTIAL` Official Sign in with ChatGPT open-source/local flow, eligible-plan path, dynamic registration/host ID, no embedded secret/API key requirement, no history access, streamed Responses API, and replaceable `AiProvider` boundary. Evidence: auth.rs implements official dynamic registration, identity-only grants and typed refresh recovery; desktop gates plan usage before and after refresh. Callback validation, streaming and provider boundary exist; eligible third-party sign-in and live inference are unverified.
- [ ] `PARTIAL` Tokens only in Windows Credential Manager/DPAPI-backed Rust storage, never localStorage/React/JSON/SQLite/logs. Evidence: `src-tauri/src/credentials.rs` uses Windows DPAPI-backed per-slot files, auth grants are zeroized before vault storage, and SQLite stores public account metadata only; Windows runtime proof remains pending.
- [ ] `PARTIAL` Streaming flow snapshots context, retrieves in parallel, compiles, dispatches, renders first/remaining deltas without waiting for nonessential enrichment. Evidence: bounded streaming Responses and provider SSE clients exist with quota/auth classification; context compiler, dispatch and UI deltas are integrated; bounded lexical memory retrieval is integrated; parallel/hybrid retrieval remains pending and live acceptance is unverified.
- [ ] `PARTIAL` Offline capture/transcription/persistence/local search/state continue, and inference shows a clear offline state. Evidence: local Windows capture/STT, durable meetings/transcripts, state UI, and Linux unsupported handling exist; local search and explicit inference error states exist; offline/native acceptance remains pending.

## 15. Database design, PLAN 1285-1447

- [ ] `PARTIAL` Migration-managed local SQLite under per-user app data with secrets excluded, WAL/foreign keys/busy timeout. Evidence: `src-tauri/src/store.rs` implements schema 7, WAL, foreign keys, busy timeout, meetings, transcript rows, provider settings, and public account metadata; secure secrets stay outside SQLite.
- [ ] `PARTIAL` Production entities and relationships for meetings, transcript segments with full fields, artifacts, projects, people, memories, documents/chunks, entities, embeddings, non-secret settings, migrations, plus accounts/usage/context profile tables. Evidence: meetings, transcript segments, settings, provider settings, public account metadata and manual memories are migrated; schema8 documents/chunks/FTS are migrated; artifacts, embeddings, usage events and context profiles remain pending.

## 16-18. Architecture, frontend, Tauri/Windows integration, PLAN 1448-1645

- [ ] `PARTIAL` Working repository boundaries for core audio/STT/meeting/memory/retrieval/context/auth/providers plus UI feature boundaries and typed bridge/events. The directory tree is illustrative and omitted as a file-by-file obligation. Evidence: audio, speech, meeting, auth, credentials, providers, inference, store, typed bridge, Connections UI, and meeting UI exist; memory/retrieval/context boundaries do not.
- [x] `PARTIAL` Tauri 2 + Rust + React/TypeScript shell, tray, global shortcut, single-instance behavior, startup setting, window state, and typed bridge exist. Evidence: `src-tauri/src/desktop.rs`, `src/bridge.ts`, `package.json`, E1-E4; Windows runtime remains unverified.
- [ ] `PARTIAL` Windows startup/tray/device/window integrations for the complete product, including single instance preventing duplicate capture. Evidence: startup, tray, window, single-instance, capture lifecycle, and shutdown code exist; Windows/device runtime proof and complete product integrations remain pending.

## 19-22. Performance, errors, diagnostics, security, PLAN 1647-1814

- [ ] `PARTIAL` Idle/active/overlay/retrieval budgets, bounded memory/queues, model unload-on-idle, and measured 8 GB CPU-only operation. Evidence: bounded capture/speech/transcript limits and stop-time model unload exist; manual-memory retrieval is capped at eight rows/16KiB before compiler budgeting; 8 GB measurement remains pending.
- [ ] `PARTIAL` Actionable errors for permissions/devices/STT/database/auth/inference/shortcuts/installer/migrations/credentials, with retries and preserved local data/unsent text. Evidence: meeting permission, capture/STT, auth, provider, streaming, migration, shortcut, and vault errors are surfaced; durable no-output retries, frozen unsent batches and partial-output recovery exist; installer/runtime coverage remains incomplete.
- [ ] `PARTIAL` Structured bounded logging exists, but required timings/device/model/STT/retrieval/context/inference/error diagnostics and redaction coverage are not complete. Evidence: bounded logging plus capture drops/errors, speech errors, auth/inference classifications, and sensitive-header handling exist; retrieval/context and full timing diagnostics are absent.
- [ ] `PENDING` Trust-boundary rules: validate model output, treat transcript/doc text as data, no arbitrary navigation, safe opener/CSP, secure credentials, user-profile database access, real deletion, and no telemetry without consent. Evidence: CSP exists in `tauri.conf.json:33-35`; no provider/data deletion/telemetry implementation.

## 23-27. Packaging, build, clean-machine tests, observability, PLAN 1816-2099

- [ ] `PARTIAL` Tauri NSIS, per-user install, offline WebView2, hash output, and Windows build/package scripts exist. Evidence: current NSIS config uses downloaded WebView2 rather than offline packaging; package scripts produce installer/hash, with clean-machine acceptance pending.
- [ ] `PARTIAL` Bundled STT runtime/model/licenses/checksums, first-run asset verification, native dependency handling, signing configuration, update preservation/migrations, and failure rollback. Evidence: model, license, manifest, checksum preparation, native build handling, and migrations exist; first-run verification, signing, updater, and rollback are absent.
- [ ] `PARTIAL` Build scripts run frontend/Rust checks and release build, but do not yet verify assets, run TypeScript tests beyond type/build, sign, or produce a tested complete installer. Evidence: root currently builds the NSIS installer and asset manifest/hash; signing, clean-install/runtime testing, and broader TypeScript tests remain absent.
- [ ] `PENDING` Clean Windows VM/physical-laptop sequence: offline install, mic/system offline transcription, DB, online sign-in, separate roles, global shortcut, streamed contextual answer, persistence, memory retrieval, reboot/startup, uninstall/data policy. Evidence: no Windows runtime; `PROGRESS.md`.
- [ ] `PARTIAL` Foundation checks pass: `npm run build`, native tests (3), clippy, Linux/headless UI checks, and MSVC cross-check. Evidence: current checks include52 library/39 core tests, Clippy on Linux and MSVC, and repeated mocked UI/source-review passes. Required live Windows integration suites remain pending.
- [ ] `PARTIAL` Required unit/integration tests for audio/resampling/ring/VAD/transcripts/budgets/retrieval/memory/auth/injection/UI/streaming/migrations and mocked end-to-end pipelines. Evidence: source tests cover stores/migrations, capture/speech, context, dispatch, auth and window helpers; IPC fixtures cover meetings, connections, history, responses, memory and documents. Real end-to-end/native acceptance remains pending.
- [ ] `PENDING` Windows tests for shortcut/tray/window/DPI/hotplug/Bluetooth/sleep/permissions/network/privacy/screenshot/failover. Evidence: no Windows runtime test records.
- [ ] `PENDING` Two-hour synthetic meeting verifies bounded memory/queues/WAL, transcript/UI/STT stability, and no callback starvation. Evidence: bounded queues and stop-drain paths exist, but no two-hour run has been recorded.
- [ ] `PENDING` Developer diagnostics screen with queue depth, drops, VAD/STT latencies, storage/retrieval/context/inference timing, RSS/model RSS. Evidence: no diagnostics UI.

## 28. Implementation phases, PLAN 2103-2230

- [ ] `PARTIAL` Phase A: shell/core/overlay/tray/single instance/shortcut/show-hide/startup/settings/SQLite/logging and package foundation exist; opacity/click-through/move/resize/privacy flags and Windows exit acceptance are incomplete or unverified. Evidence: files cited above, `PROGRESS.md`.
- [ ] `PARTIAL` Phase B: integrated dual WASAPI, bounded buffers, resampling, VAD, recovery, diagnostics, and two-hour exit condition. Evidence: integrated dual-source WASAPI, bounded buffers, resampling, VAD, and error reporting exist; device recovery, diagnostics, two-hour run, and Windows proof remain pending.
- [ ] `PARTIAL` Phase C: native STT abstraction/backend/model/package/partial-final/source/persistence/UI and clean-machine exit. Evidence: CPU STT, bundled model, partial/final source-tagged events, persistence, model unload, and transcript UI exist; clean-machine and Windows exit evidence are pending.
- [ ] `PARTIAL` Phase D: meeting lifecycle/recent buffer/search/rolling state/artifacts/history. Evidence: lifecycle, consent, snapshots, durable finals, bounded transcript state, history paging, local search and saved-meeting UI exist; rolling state and artifacts remain pending.
- [ ] `PARTIAL` Phase E: memories/entities/documents/FTS/embeddings/hybrid retrieval/reranking/provenance/UI. Evidence: schema7 manual memory CRUD/FTS, provenance, dedicated management UI and enabled-only lexical retrieval are implemented; schema8 documents and enabled-only lexical retrieval are integrated; entities, embeddings, hybrid retrieval and reranking remain pending.
- [ ] `PARTIAL` Phase F: ChatGPT auth/credentials/account UI/model discovery/streaming/refresh/sign-out/multiple accounts/API-key providers/fallback/usage. Evidence: auth, DPAPI vault, account metadata/UI, provider settings/key handling, model discovery, and streaming exist; serialized refresh, cancellable reauthorization, explicit account/model selection and bounded sign-out are implemented with compile/UI checks; fallback dispatch and actual returned usage are integrated; eligible live proof, live usage/cap acceptance remains pending.
- [ ] `PARTIAL` Phase G: query understanding/parallel retrieval/compiler/budget/fusion/injection boundaries/source chips/streaming Markdown/context profiles/screenshot image+OCR. Evidence: inference stream parsing exists; compiler budgets, untrusted-context boundaries and GFM/Shiki UI exist; query understanding, retrieval, context profiles and screenshot/OCR remain pending.
- [ ] `PARTIAL` Phase H: profiling/keyboard/audio recovery/offline/memory deletion/diagnostics/accessibility/crash recovery/installer/signing/updater/clean-machine test. Evidence: installer build, crash/startup recovery, and local offline capture paths exist; profiling, recovery acceptance, deletion, diagnostics, signing, updater, and clean-machine tests remain pending.

## 29. Definition of done, PLAN 2232-2310

- [ ] `PENDING` Installation: clean Windows setup, no tools, local STT, WebView2, uninstall.
- [ ] `PENDING` Desktop: reliable tray/global shortcut/fast overlay/multi-monitor+DPI/one capture instance.
- [ ] `PENDING` Audio/STT: local system+mic, distinct sources, no raw upload, device recovery.
- [ ] `PENDING` Meetings: start/stop, incremental persistence, immediate recent context, local history search, artifacts.
- [ ] `PENDING` Memory: add/import, local storage, lexical+semantic+structured retrieval, provenance, deletion.
- [ ] `PENDING` AI: Continue with ChatGPT, no intended-path API key, secure tokens, streaming, live+personal context, offline local operation.
- [ ] `PENDING` Overlay/privacy/screen input: opacity/click-through/move/resize, share/taskbar exclusion, both screenshot paths, streaming Markdown/code.
- [ ] `PENDING` Accounts/providers: multiple ChatGPT accounts with user switching, DeepSeek/Gemini Credential Manager fallbacks, usage page.
- [ ] `PENDING` Distribution: reproducible scripts/CI, clean VM test, signing configuration, no embedded secrets.

## 30. Explicit non-goals, PLAN 2312-2334

- [x] `IMPLEMENTED AS SCOPE` macOS/Linux/mobile/browser extension, cloud sync, hosted accounts/team workspaces/remote vectors/web dashboard/CRM/video capture, automatic screen capture, autonomous third-party control, proctoring defeat, remote transcription, and default raw-audio storage are excluded. Evidence: no such implementation, E5. Linux is a build host, not a launch target.

## 31-34. Engineering principles, suggestions, fixed decisions, milestone, PLAN 2338-2490

- [ ] `PARTIAL` Local-before-remote, nonblocking audio, incremental persistence, retrieval instead of dumping, local state authority, measured latency, graceful partial failure, and durable production-shaped storage/audio/auth/package/update paths are behavioral constraints. Evidence: nonblocking bounded capture, incremental durable finals, local state authority, graceful stop/error paths, OAuth/vault, and package build exist; retrieval, measured latency, and update paths remain incomplete.
- [ ] `PARTIAL` Suggested stack decisions: Tauri/Rust/React/Vite/WASAPI/Tokio/SQLite WAL/FTS5/local embeddings/native Whisper-compatible STT/ChatGPT streaming/Credential Manager/global shortcut/NSIS/offline WebView2/signed x64 setup. Evidence: current source uses Sherpa ONNX/Parakeet with offline WebView2 configured; local embeddings and signing remain pending.
- [x] `IMPLEMENTED AS SCOPE` Fixed no-go decisions remain respected: not Electron/browser/cloud transcription/raw upload/mixed audio/Python target/Postgres/Redis/backend-first/full-screen chat/auto ChatGPT account rotation/wholesale memory upload/disposable V0/silent document upload/model-as-state. Evidence: source/dependency inventory and current UI, E5.
- [ ] `PENDING` First-build milestone: real Tauri/Rust/React/tray/shortcut/single-instance/SQLite/settings/logging/Windows packaging/NSIS/scripts/CI/README. Foundation covers most items; CI definition and full acceptance remain incomplete. Evidence: files above, `PROGRESS.md`.

## 35. Final product scenario, PLAN 2492-2555

- [ ] `PENDING` End-to-end proof: install only setup executable, ChatGPT authorization, mic selection, start meeting, separate system/mic transcript, shortcut, live+project-memory retrieval, compiled authorized streaming answer, hide overlay while transcription continues, completed local artifacts, restart persistence. Evidence: local source paths exist, but no complete Windows/user run has been proved. Context compiler, dispatcher and answer UI exist; manual memory retrieval exists; full Windows acceptance remains pending.

## 36-37. References and build mandate, PLAN 2557-2623

- [ ] `PENDING` External references are research pointers, not evidence. Every protocol/framework claim must be checked against current docs during implementation. Evidence: auth/provider source exists, but live eligibility and runtime validation are not proof from URLs or plan text.
- [ ] `PARTIAL` Completion order is correctness, privacy, capture/transcription reliability, latency, resource use, visual polish; final repo must contain coherent native audio, local inference, event concurrency, memory, retrieval, context, secure OAuth, streaming, desktop UX, and production packaging. Evidence: native audio/STT, event concurrency, OAuth, streaming, desktop UX, local memory/documents, lexical retrieval and package build exist; hybrid retrieval, runtime proof and production acceptance remain pending.

## 38. Added overlay, privacy, accounts, providers, screen input, PLAN 2625-3019

### 38.1 Overlay controls, PLAN 2632-2697

- [ ] `PENDING` Settings/defaults: background opacity 15-100% default 85, text opacity 50-100% default 100, click-through off, move 20 logical px, resize 40 logical px; persist position/size/monitor/opacity.
- [ ] `PENDING` Configurable global shortcuts for overlay/send/image/OCR/account switch/move/resize/opacity/click-through/reset with correct registration scope, conflict preservation, collision warnings, logical-unit clamp, debounced writes, focus/no-focus behavior. Evidence: only two basic shortcuts exist; no 38.1 handlers, E5.

### 38.2 Privacy, PLAN 2699-2745

- [ ] `PARTIAL` Privacy settings default capture exclusion/taskbar/Alt+Tab/meeting notification suppression on and tray icon visible; use only documented flags, handle old Windows behavior, state limitations, protect overlay and management windows, and document unprotected tray/dialog/process/camera/hardware surfaces. Overlay `contentProtected: true` and `skipTaskbar: true` are configured, but Alt+Tab, management-window coverage, settings, old-build behavior, and runtime proof are absent. Evidence: `src-tauri/tauri.conf.json:23-29`, E5.

### 38.3 Accounts, PLAN 2747-2783

- [ ] `PENDING` Multiple ChatGPT accounts, labels/email/status/limits, repeat sign-in, per-account Credential Manager targets, one active account, meeting snapshot, user-only switching, limited-account choices, no automatic ChatGPT rotation, host-ID decision checked against current docs. Evidence: no accounts table or auth.

### 38.4 Providers, PLAN 2785-2825

- [ ] `PARTIAL` DeepSeek/Gemini API keys entered securely outside SQLite/React, editable base URLs, preconfigured endpoints, `/models` discovery and selection, reorderable fallback, provider consent/context policy, image capability handling, provider chip, provider-neutral compiler, pre-first-token failover only, partial-answer retry after tokens. Evidence: editable provider settings, DPAPI key storage, DeepSeek/Gemini defaults, and bounded model discovery exist; consented pre-output fallback, bounded compiler, actual returned usage and durable retries exist; image routing and live usage/cap acceptance remain pending.

### 38.5 Usage, PLAN 2827-2864

- [ ] `PARTIAL` `usage_events`, per-provider/account seven-day usage page, editable API price table, daily token/cost caps, honest ChatGPT quota display, and no fabricated quota bars. Evidence: schema10 usage_events stores per-attempt outcomes, returned counts and timings with public ChatGPT account attribution; the bounded local Usage page covers today/seven local days, unknown counts, limit hits and recovery. Schema11 adds editable per-model USD pricing, private URL-bound per-key identity, atomic local daily token/estimated-cost admission caps, captured-price receipts and anonymous counters that survive meeting deletion. Unknown usage and legacy attribution block enabled caps conservatively; live acceptance remains pending.

### 38.6 Context center, PLAN 2866-2916

- [ ] `PARTIAL` Named context profiles with custom instruction, pinned items/budget, retrieval scope, allowed providers, pre-meeting snapshot, overflow explanation, inspectable “What was sent” with provenance/tokens/provider/account and deletion linkage. Evidence: bounded compiler and per-meeting send/instruction snapshots exist; bounded prepared-request inspection with provenance/provider/model/estimates and meeting deletion linkage exists; new contexts include public account attribution; named profiles and broader deletion remain pending.

### 38.7 Screenshot/OCR, PLAN 2918-2983

- [ ] `PENDING` User-only `Ctrl+Shift+O` image and `Ctrl+Shift+X` local OCR messages, optional typed instruction/default, active-meeting independence, cursor-monitor xcap capture, overlay exclusion/hide fallback, 2048px PNG/image-provider routing, Windows OCR/BGRA/max-dimension/code indentation behavior, OCR text visibility, cache deletion/keep setting, and no image/OCR logs. Evidence: no xcap/OCR/screenshot code or shortcuts, E5.

### 38.8 Answer rendering, PLAN 2985-3008

- [ ] `PARTIAL` Provider prompt for GFM/language fences; streaming-safe Markdown; Shiki code highlighting/theme; labels/copy/horizontal scroll; tables/task lists/strike; raw HTML disabled; safe external links; no remote images. Evidence: trusted GFM/language-fence instruction, react-markdown/remark-gfm and bounded lazy Shiki are integrated. HTML is disabled; links use a validated native opener; images never fetch; code copy and tables have fixture checks. Production assets pass the native-CSP browser fixture; Windows rendering and streaming performance remain unverified.

### 38.9 Control center, PLAN 3010-3018

- [ ] `PARTIAL` Management pages Accounts, Providers, Usage, Context, Shortcuts, Overlay, Privacy, Audio, Memory, Meetings; correct next-meeting snapshot exception rules; press-to-record/conflict/reset shortcut UI. Evidence: meeting preferences, Connections, saved meetings and dedicated manual memory management exist; local Usage is also integrated; full ten-page control center and shortcut recorder remain pending.

### 38.10 Shipped-product examples, PLAN 3023-3035

- [ ] `INFORMATIONAL` Example products and mechanism mappings are references, not independent runtime obligations. Their corresponding Harness rows remain tracked above; none are marked done from the examples.

The literal inventory in [PLAN_BULLETS.md](PLAN_BULLETS.md) records each bullet separately. Refresh it with `python scripts/plan-bullets.py`; checked records retain their evidence. This grouped report does not replace individual acceptance.

## Actual current blockers and ranked next steps

1. **Blocker: Windows capture/STT proof.** Run `E6` on real hardware, test device changes and recovery, then run the two-hour bounded meeting and low-RAM checks.
2. **Pending dispatch acceptance.** Exercise the implemented compiler, sequential batches, cancellation, retries and fresh remote consent on Windows with real providers.
3. **Blocker: official ChatGPT proof.** Verify the restored official dynamic-registration integration on Windows. Test with an actually eligible account/client configuration. Private-plan eligibility remains unproven.
4. **Pending provider acceptance.** Exercise consented pre-output fallback and partial recovery; verify implemented editable pricing/per-key caps and add model-specific budgets.
5. **Pending memory/context work.** Manual memory and bounded lexical retrieval are implemented. Add PDF/DOCX ingestion, derived artifacts, hybrid retrieval, entity relationships, named context profiles and broader deletion controls.
6. **Blocker: Windows product acceptance.** Finish overlay/privacy/control center/screenshot/OCR, clean VM/physical-laptop tests, accessibility/UI pass, integration pass, and runtime/low-RAM pass.

These are pending requirements and acceptance gaps. Source implementation and synthetic UI fixtures do not prove Windows hardware behavior, eligible sign-in, memory retrieval or clean-machine acceptance.
