# Harness

Private Windows meeting assistant built with Tauri 2, Rust, React, and SQLite.

This development build includes the desktop foundation, persistent meeting preferences, a Connections page, browser-based ChatGPT OAuth handlers, encrypted Windows credentials, explicit account/model selection, cancellable reauthorization, serialized refresh and sign-out, and Gemini/DeepSeek provider configuration and model discovery. WASAPI and local CPU speech modules compile, with Parakeet TDT model resources; a Linux reference-WAV decode probe passes, while Windows live decode remains unverified. Permissioned meeting capture is wired to local transcription, finalized transcript storage, and the live meeting window. Saved meetings can be reopened and searched locally with separate MIC/SYSTEM provenance. Permissioned automatic sending, hotkey batches, meeting questions, durable retries and streamed Markdown answers are wired in source. Saved meetings can reopen in the assistant after restart. Live sign-in, simultaneous Windows voice/inference, and the remaining plan features still require acceptance checks.

## Develop

Install Node 24 or later, Rust as pinned in `rust-toolchain.toml`, and the [Tauri development prerequisites](https://v2.tauri.app/start/prerequisites/) for your build OS. Windows builds require the Visual Studio C++ build tools, CMake, and WebView2. Users of the final installer will not need these development tools.

```sh
npm ci
npm run prepare:model
npm run tauri dev
```

For the frontend-only browser preview:

```sh
npm run dev
```

Browser preview preferences use isolated browser local storage. The desktop app stores preferences in SQLite under Tauri's per-user local application-data directory, in `data/harness.db`. OAuth tokens and API keys are stored only in per-user DPAPI-encrypted files under `data/credentials`. React and SQLite receive public metadata. A key is bound to its saved base URL; changing that URL requires saving a key again.

## Verify

```sh
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
```

The core tests run without a WebView or audio device. Full desktop compilation still requires OS development libraries.

## Package for Windows

On a Windows x64 build machine:

```powershell
.\scripts\package.ps1
```

The script runs checks and creates `dist/Harness-Setup-x64.exe` and its SHA-256 hash. NSIS is configured for per-user installation with a downloaded WebView2 bootstrapper. Installation may need network access if WebView2 is absent. Build preparation verifies and bundles the English Parakeet TDT int8 ONNX model and its license. The installed engine uses a caller-supplied local model path and never downloads models or runs Python. Packaging is not equivalent to the final clean-machine product acceptance test.

Closing the window hides Harness while capture continues. The tray's Quit command stops inference, preserves partial output, flushes finalized speech, unloads the model, and waits for sign-in credential cleanup before exiting. `Ctrl+Space` toggles the assistant; `Ctrl+Shift+Enter` sends unsent finalized system speech available at the press, in bounded batches. Automatic mode requires fresh remote-send consent at each meeting and pauses after failure or cancellation. Preferences allow changing both. An unavailable shortcut produces an actionable error rather than preventing tray access.

Read [PLAN.md](PLAN.md) for the product contract and [PROGRESS.md](PROGRESS.md) for completed checks, remaining work, and resume instructions. [PRODUCT.md](PRODUCT.md) and [DESIGN.md](DESIGN.md) preserve product and visual decisions.

## Windows foundation acceptance

The Phase A exit check remains pending until these pass on Windows:

1. Install the generated installer under a normal user account and open Harness from Start. A second launch must focus the existing process.
2. Close or press Escape to hide the window. Open it again from the tray and with the configured overlay shortcut. Quit must exit the process.
3. Save both sending modes and every response mode; restart and confirm persistence. Empty custom instructions and duplicate shortcuts must be rejected. A conflicting registered shortcut must leave tray access working and preserve the prior saved preferences.
4. Enable startup, sign out/in, and confirm Harness starts in the tray. Disable startup and confirm it no longer launches. Move the window between monitors, disconnect a monitor, and reopen it inside the remaining work area.
5. Verify hotkey sending, fresh automatic-send consent, only-new-system text, explicit MIC context, streamed answers, cancellation, no-output retry, partial output, fallback limits, and saved-meeting recovery after restart. In Connections, complete sign-in in the system browser, select the intended account, cancel a pending attempt, and verify that credentials survive restart. Discover and select a ChatGPT model. Reauthorize through Account options, then sign out and distinguish local cleanup from remote revocation confirmation. Check provider key storage, URL binding, removal and model discovery with an authorized test key.

Windows runtime tests, clean installation, and the 8 GB memory budget cannot be certified by a Linux compilation check.

## Local speech probe

The probe accepts an explicit model path and a mono PCM16 WAV at 16 kHz, up to 15 seconds. It prints transcript text only when you run the developer command:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --example transcribe_probe -- src-tauri/models/parakeet path/to/sample.wav
```

An earlier Whisper decoder probe is historical evidence only. It does not measure the current Parakeet backend. On this Linux host, the current Parakeet probe decoded an 11-second reference clip in 0.92 seconds, with about 1.05 GiB peak resident memory (model load plus decode). This is a single decoder probe, not full-application or Windows performance. The English default, live partial updates, device recovery and long-meeting soak checks remain pending.

## Local memory

Open Memory from the tray or the book icon in the assistant. It opens a separate 900×700 management window. Add a manual project, person, organization, preference, note, decision, experience, education or term. Each entry retains its manual source and creation/update times. Search, edits and deletion stay local.

New entries default to local only. Enable an entry explicitly to allow relevant excerpts in authorized answers, including consented fallback providers. Local FTS5/BM25 retrieves at most eight enabled entries and 16 KiB of body text before the context compiler applies its budget. Title/project terms receive extra lexical weight. This is lexical retrieval; embeddings, reranking, related-entity records and inspectable sent context remain unfinished.

The window confirms before discarding unsaved changes, remains open during saves, and destroys its WebView when closed. Database and UI pages replace prior results and stay bounded. Run `npm run check:memory` with the same browser setup as the response fixture; it mocks IPC and does not prove Windows window behavior.

## UI regression

With `npm run dev` running, use `npm run check:ui`. Install Playwright Chromium with `npx playwright install chromium`, or point `HARNESS_BROWSER_EXECUTABLE` at an existing Chromium/Edge executable. The check mocks native IPC and credentials. It verifies response attempts, stale events, consent, safe Markdown/code/links/images, copying, narrow layouts, and reopening a saved meeting. It does not prove Windows audio, OAuth eligibility, or native clipboard behavior.

After `npm run build`, use `HARNESS_PRODUCTION_FIXTURE=1 npm run check:ui` to run the same IPC fixture against production assets with the native content security policy. This verifies CSS-based syntax colors without permitting inline styles.

The compiler currently uses a conservative application cap and byte-based estimate. Discovered model-specific limits, exact tokenizers, retrieval, inspectable compiled context, usage pages, and the remaining PLAN features are still pending. See REQUIREMENTS.md and PROGRESS.md.

## Local documents

Open Documents from the tray or document icon. Import UTF-8 `.txt`, `.md` or `.json` files using the native Windows file picker. Choose an original-file reference or managed local copy. Limits are 1 MiB per source and 256 KiB extracted text; PDF and Word support remain unfinished. JSON indexes meaningful scalar leaves with their paths. No source is uploaded for indexing.

New imports are local only. Enable a document explicitly for relevant authorized answers. Metadata retains original path, raw-file hash, modification time and index version. Selection checks for changes; reindexing is explicit and replaces chunks only after success. Local search includes disabled documents. Deletion removes indexed text and schedules managed-copy cleanup while preserving the original file. Retrieval uses enabled-only FTS5/BM25, at most eight chunks and 16 KiB of text. Absolute source paths are omitted from remote evidence.

Run `npm run check:documents` against the development preview for the mocked interface checks. Native dialogs, copies, close behavior and actual inference still need Windows acceptance.

Current sign-in source uses a fixed public OAuth client ID rather than the agreed dynamic registration. Third-party eligibility and error-free sign-in remain unverified; this is an open integration gap.
