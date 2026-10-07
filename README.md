# Harness

Private Windows meeting assistant built with Tauri 2, Rust, React, and SQLite.

This development build includes the desktop foundation, persistent meeting preferences, a Connections page, official ChatGPT dynamic OAuth handlers, encrypted Windows credentials, explicit account/model selection, cancellable reauthorization, serialized refresh and sign-out, and Gemini/DeepSeek provider configuration and model discovery. WASAPI and local CPU speech modules compile, and the bundled model passes a real WAV decode probe. Permissioned meeting capture is wired to local transcription, finalized transcript storage, and the live meeting window. Saved meetings can be reopened and searched locally with separate MIC/SYSTEM provenance. Permissioned automatic sending, hotkey batches, meeting questions, durable retries and streamed Markdown answers are wired in source. Saved meetings can reopen in the assistant after restart. Live sign-in, simultaneous Windows voice/inference, and the remaining plan features still require acceptance checks.

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

The script runs checks and creates `dist/Harness-Setup-x64.exe` and its SHA-256 hash. NSIS is configured for per-user installation with an offline WebView2 installer. Build preparation verifies and bundles the English base.en-q5_1 speech model and its license. The installed engine uses a caller-supplied local model path and never downloads models or runs Python. Packaging is not equivalent to the final clean-machine product acceptance test.

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
cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --example transcribe_probe -- src-tauri/models/ggml-base.en-q5_1.bin path/to/sample.wav
```

An 11-second upstream reference clip decoded correctly in 7.28 seconds with approximately 242 MiB peak resident memory on the current Linux build host. This measures the decoder alone; it does not certify full-application performance or the Windows 8 GB target. The English default, live partial updates, device recovery and long-meeting soak checks remain pending.

## Local memory

Open Memory from the tray or the book icon in the assistant. It opens a separate 900×700 management window. Add a manual project, person, organization, preference, note, decision, experience, education or term. Each entry retains its manual source and creation/update times. Search, edits and deletion stay local.

New entries default to local only. Enable an entry explicitly to allow relevant excerpts in authorized answers, including consented fallback providers. Local FTS5/BM25 retrieves at most eight enabled entries and 16 KiB of body text before the context compiler applies its budget. Title/project terms receive extra lexical weight. This is lexical retrieval; document import, embeddings, reranking, related-entity records and inspectable sent context remain unfinished.

The window confirms before discarding unsaved changes, remains open during saves, and destroys its WebView when closed. Database and UI pages replace prior results and stay bounded. Run `npm run check:memory` with the same browser setup as the response fixture; it mocks IPC and does not prove Windows window behavior.

## UI regression

With `npm run dev` running, use `npm run check:ui`. Install Playwright Chromium with `npx playwright install chromium`, or point `HARNESS_BROWSER_EXECUTABLE` at an existing Chromium/Edge executable. The check mocks native IPC and credentials. It verifies response attempts, stale events, consent, safe Markdown/code/links/images, copying, narrow layouts, and reopening a saved meeting. It does not prove Windows audio, OAuth eligibility, or native clipboard behavior.

After `npm run build`, use `HARNESS_PRODUCTION_FIXTURE=1 npm run check:ui` to run the same IPC fixture against production assets with the native content security policy. This verifies CSS-based syntax colors without permitting inline styles.

The compiler currently uses a conservative application cap and byte-based estimate. Discovered model-specific limits, exact tokenizers, retrieval, inspectable compiled context, usage pages, and the remaining PLAN features are still pending. See REQUIREMENTS.md and PROGRESS.md.
