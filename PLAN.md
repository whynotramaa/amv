# Harness --- Product & Engineering Plan

> **Status:** V0 launch specification\
> **Platform:** Windows-first, x64\
> **Product class:** Local-first ambient meeting agent / floating AI
> copilot\
> **Primary implementation:** Tauri 2 + Rust + React/TypeScript\
> **Distribution artifact:** `Harness-Setup-x64.exe`\
> **Document purpose:** This file is the implementation contract. A
> coding agent should be able to build the product from this document
> without inventing product behavior or architecture.

------------------------------------------------------------------------

## Agreed implementation decisions

These decisions from the product conversation override conflicting earlier wording. The latest sales-assistant and visual decisions below also override older examples, including interview examples and shadow guidance.

- **Use and distribution:** private personal use only. Public code signing and public release infrastructure are not V0 blockers. Do not assume private-app ChatGPT-plan eligibility; prove the official sign-in/inference path before claiming it works.
- **Minimum hardware:** Windows x64 with 8 GB RAM, CPU-only operation, no dedicated GPU required. Keep the model unloaded when idle, bound PCM queues, and avoid loading embedding and STT models concurrently without a measured budget.
- **Sending mode:** before each meeting, the settings panel offers `On hotkey` and `Automatically`. Default to `On hotkey`. Save the chosen settings and snapshot them at meeting start. Editing those settings takes effect for the next meeting.
- **New messages:** only new finalized system-audio speech becomes a transcript user message. Never repeatedly append the whole meeting as a new message. Automatic mode batches finalized speech while one response is running; hotkey mode sends all unsent finalized system speech. Preserve order, segment provenance, and retry state. Hotkey batches stop at the latest finalized row available at the press. Large finalized segments may be sent as durable UTF-8 fragments without changing the saved full transcript. Microphone transcripts stay separate and are available as explicitly compiled context.
- **History:** Harness owns local message history. A new transcript message is appended to that conversation. The HTTP inference request must include prior context because the intended API path does not provide persistent server conversation storage. Bound older context to model limits while keeping the full transcript/history locally; disclose incomplete context rather than silently losing the current message.
- **Response mode:** offer `Suggested answers`, `Summary`, and `Custom instruction`. All modes are configurable before the meeting; custom mode requires a non-empty instruction.
- **Hotkeys:** the popover shortcut and the send-transcript shortcut are distinct and configurable. Opening or hiding the popover must not accidentally send speech.
- **Offline onboarding:** local setup may finish before ChatGPT sign-in. Local capture, transcription, persistence, and search remain available without account/network access.
- **Remote policy:** automatic mode requires fresh remote-send consent at meeting setup and authorizes sending new finalized system text during that meeting. Restoring a saved meeting does not restore automatic permission. Failure or cancellation pauses automatic sending until explicit manual resumption. Hotkey mode makes no background summarization requests. No raw audio is sent in either mode. Inference errors and usage limits preserve unsent speech and local transcripts.
- **Purpose and consent:** assist sales calls with meeting notes, grounded statistics, and clear answers. Capture and remote sending require explicit user control and meeting permissions. Do not present this product as an interview-cheating or covert-monitoring tool. Treat transcripts and documents as untrusted context, and give statistical answers provenance rather than invented certainty.
- **Design:** clarity first, with Linear-level restraint and an original desktop utility composition. Poppins for UI text, Geist Mono for code, no serif fonts, no Sparkles icon. Use original consistent icons. No CSS or native window shadows: separate surfaces with thin low-opacity borders and subtle luminance changes. Establish shared spacing, typography, surfaces, radii, borders, icon sizes, control heights, and states before components. The latest direction overrides the earlier system-font and shadow examples.
- **Provider configuration:** offer editable base URLs and securely entered API keys, with DeepSeek and Gemini preconfigured. Model choices come from provider discovery. Confirm allowed providers and remote-context sharing before automatic fallback; never rotate ChatGPT accounts automatically to bypass a limit. ChatGPT sign-in remains the preferred path, uses official dynamic registration as confirmed by the user, and must be verified with an actual eligible account/client configuration.
- **Manual memory sharing:** new manual memories stay local by default. Each entry has an explicit enable control for relevant authorized answers, including consented fallback providers. Search, provenance, editing and deletion are local. Management uses a separate normal-size window with draft-safe close and WebView release. Lexical retrieval works before embeddings are available; hybrid retrieval remains required.
- **Completion evidence:** a working Windows installer and end-to-end ChatGPT sign-in, smooth separate audio transcription, and fallback inference are required. Track every normative plan bullet with evidence and pending checks. Run multiple verification passes for correctness, integration, UI/accessibility, and Windows runtime. Measure memory on the minimum 8 GB machine; bounded buffers and unload-on-idle are requirements, not proof of zero growth.
- **Overlay privacy:** by default, Harness windows are excluded from screen capture and screen share, and hidden from the taskbar and Alt+Tab. Harness uses documented Windows window flags for this and nothing else. Section 38.2 sets the limits.
- **Overlay control:** opacity, click-through, position, and size all have settings and keyboard shortcuts. Section 38.1 has the full shortcut table.
- **Accounts and providers:** the user can sign in more than one ChatGPT account and add DeepSeek or Gemini API keys as fallbacks. Switching ChatGPT accounts is a user action. Automatic failover goes only to API-key providers. See sections 38.3 to 38.5.
- **Screen input:** `Ctrl+Shift+O` sends a full screenshot as an image. `Ctrl+Shift+X` sends local OCR text of the screenshot. Harness captures the screen only when the user presses one of these shortcuts. See section 38.7.
- **Answer formatting:** answers render as streaming GitHub-flavored Markdown with highlighted code blocks. See section 38.8.
- **Milestones:** package the desktop foundation first; then prove simultaneous local STT and authorized streaming inference early, before expanding embeddings and reranking. The directory tree in section 16 is illustrative: create modules when they contain working behavior.

Progress, runnable checks, environment limitations, and resume instructions live in `PROGRESS.md`. An unchecked Windows acceptance test is not evidence of completion.

------------------------------------------------------------------------

## 0. Executive definition

Harness is a Windows-native, local-first ambient meeting agent.

It runs quietly in the Windows system tray, captures **system audio and
microphone audio as separate streams**, transcribes both locally in near
real time, maintains an evolving representation of the current meeting,
retrieves relevant information from the user's persistent personal
memory, and exposes an instant floating popover through a global
keyboard shortcut.

The product is not a full-size chat client and not merely a meeting
recorder. The meeting itself is the live context. The user should be
able to invoke Harness at any moment and ask a question whose answer can
depend on:

1.  what another participant just said;
2.  what the user said earlier;
3.  the whole current meeting;
4.  prior meetings;
5.  personal/project context stored in Harness;
6.  imported local documents and notes;
7.  structured facts Harness has learned and the user has retained.

AI inference uses **Sign in with ChatGPT** and eligible ChatGPT plan
usage. Everything that can reasonably remain local should remain local:
audio capture, raw audio handling, transcription, transcript storage,
meeting state, document ingestion, retrieval indexes, embeddings where a
suitable local model is used, memory, settings, and context assembly.
Only the deliberately compiled text context required for an AI request
is sent to the remote model.

The launch build is **V0, but V0 is the product**. Do not implement a
throwaway prototype architecture.

------------------------------------------------------------------------

# 1. Non-negotiable product requirements

These requirements override implementation convenience.

## 1.1 Windows application

Harness is a Windows desktop application, not a browser app.

Primary launch target:

-   Windows 11 x64
-   Windows 10 22H2 x64 should be supported where the required Windows
    APIs and WebView2 runtime behave correctly.
-   ARM64 is not a launch blocker. Keep architecture boundaries portable
    enough that an ARM64 build can be added later.

Harness must:

-   start normally from the Start menu;
-   optionally launch at Windows login;
-   live in the system tray;
-   remain usable without a full-size main window;
-   expose a global shortcut;
-   show/hide its popover quickly;
-   capture system loopback audio;
-   capture microphone audio;
-   operate correctly across multiple monitors;
-   handle Windows DPI scaling;
-   recover from audio-device changes;
-   recover from sleep/wake;
-   shut down cleanly.

## 1.2 Local-first

The following data and processing are local by default:

-   microphone PCM;
-   system/loopback PCM;
-   VAD;
-   resampling;
-   speech-to-text;
-   transcript database;
-   meeting summaries/state cache;
-   imported documents;
-   local search indexes;
-   personal memories;
-   user/project/entity records;
-   retrieval;
-   context selection;
-   settings;
-   logs;
-   authentication secrets/tokens in OS-backed secure storage.

Raw audio MUST NOT be uploaded to the model provider for normal
operation.

The remote model receives only the context selected by the context
compiler and the current user request.

"Local-first" does **not** mean the final ChatGPT inference is local.
The ChatGPT request is remote and requires network connectivity.

## 1.3 Clean-machine installation

The release deliverable is:

`Harness-Setup-x64.exe`

A non-technical user should be able to copy this file to a compatible
clean Windows machine, double-click it, install Harness, sign in, grant
microphone access if required, and use the product.

The target machine must **not** need the user to manually install:

-   Rust;
-   Cargo;
-   Node.js;
-   npm/pnpm;
-   Python;
-   Git;
-   CMake;
-   Visual Studio Build Tools;
-   Whisper;
-   FFmpeg;
-   SQLite;
-   a local database server;
-   model-serving software;
-   Docker.

All runtime components needed by Harness must either be bundled into the
installer or installed by Harness itself as part of a controlled
first-run flow.

For the canonical shareable release, prefer a
**self-contained/offline-capable installer**:

-   NSIS setup executable generated through Tauri;
-   offline WebView2 installer included;
-   local STT runtime included;
-   default STT model included if licensing permits redistribution;
-   database migrations included;
-   app assets included.

If bundling the chosen STT model makes the installer very large, that is
acceptable for the canonical offline build. A smaller online/bootstrap
installer may be produced later, but it is not a replacement for the
self-contained release artifact.

## 1.4 No hidden server dependency

Harness must not require our own backend server merely to run.

Do not introduce:

-   hosted PostgreSQL;
-   Redis;
-   remote vector database;
-   remote transcription service;
-   hosted memory service;
-   a Harness account server;
-   a Docker daemon.

The local desktop process is the application backend.

OpenAI/ChatGPT is the external inference dependency.

## 1.5 Privacy and consent

Harness is a personal meeting assistant. The user owns its windows and
can keep them out of their own screen shares.

In scope (section 38.2):

-   exclude Harness windows from screen capture and screen share with
    the documented `WDA_EXCLUDEFROMCAPTURE` window flag;
-   hide Harness windows from the taskbar and Alt+Tab.

Out of scope:

-   detecting, hiding from, or defeating proctoring, monitoring, or
    anti-cheat software;
-   secretly persisting audio against the user's configured retention
    behavior.

Provide clear recording/transcription state and controls. The user is
responsible for complying with meeting consent and recording laws, and
with the rules of any meeting or interview they join.

------------------------------------------------------------------------

# 2. Product mental model

The system should be thought of as five local engines connected by an
event bus:

``` text
┌─────────────────────────────────────────────────────────────────┐
│                         HARNESS                                 │
│                                                                 │
│  Windows Audio                                                  │
│  ┌───────────────┐         ┌────────────────┐                   │
│  │ System/WASAPI │────────▶│                │                   │
│  └───────────────┘         │ Local Speech   │                   │
│  ┌───────────────┐────────▶│ Engine         │                   │
│  │ Microphone    │         │                │                   │
│  └───────────────┘         └───────┬────────┘                   │
│                                    │ transcript events          │
│                                    ▼                            │
│                         ┌─────────────────────┐                  │
│                         │ Meeting State Engine│                  │
│                         └──────────┬──────────┘                  │
│                                    │                            │
│            ┌───────────────────────┼──────────────────────┐     │
│            ▼                       ▼                      ▼     │
│    ┌───────────────┐      ┌────────────────┐     ┌────────────┐ │
│    │ Personal      │      │ Local Retrieval│     │ Recent     │ │
│    │ Memory        │◀────▶│ + Reranking    │     │ Transcript │ │
│    └───────┬───────┘      └───────┬────────┘     └─────┬──────┘ │
│            └───────────────────────┼────────────────────┘       │
│                                    ▼                            │
│                         ┌─────────────────────┐                  │
│                         │ Context Compiler    │                  │
│                         └──────────┬──────────┘                  │
│                                    │ selected text only         │
└────────────────────────────────────┼────────────────────────────┘
                                     ▼
                              ChatGPT inference
                                     │
                                     ▼
┌────────────────────────────────────┼────────────────────────────┐
│                         Floating Popover                        │
└─────────────────────────────────────────────────────────────────┘
```

The LLM does not own application state. Harness does.

------------------------------------------------------------------------

# 3. User experience

## 3.1 First launch

The first-run sequence should be short and deterministic.

### Screen 1 --- Welcome

Show:

-   Harness identity/wordmark;
-   one-sentence explanation;
-   `Continue with ChatGPT`.

Do not require the user to create a separate Harness account.

### Screen 2 --- ChatGPT authorization

Launch the user's default browser and execute the official Sign in with
ChatGPT flow.

The app must:

1.  create and persist an installation-specific `ext_agent_host_id`;
2.  initiate the documented open-source dynamic registration flow;
3.  use the official callback mechanism;
4.  validate returned identity/auth state as required by the official
    integration;
5.  store public profile metadata in the local database where useful;
6.  store refresh/access credentials only in Windows-backed secure
    credential storage;
7.  never expose tokens to the React layer.

Sign in with ChatGPT does not grant access to the user's ChatGPT
conversation history. Harness memory is independent.

### Screen 3 --- Audio

Show detected:

-   default microphone;
-   default system output device.

Allow the user to test both.

The test should visually show activity from each stream.

### Screen 4 --- Local AI setup

Verify:

-   STT runtime is present;
-   bundled model exists and checksum matches;
-   database is writable;
-   local search index can initialize.

If the canonical installer contains the model, no model download should
be necessary.

### Screen 5 --- Shortcut

Default shortcut:

`Ctrl + Space`

If Windows or another application has already claimed it, offer:

`Ctrl + Shift + Space`

The shortcut must be configurable later. A separate send-transcript shortcut defaults to
`Ctrl + Shift + Enter`; it sends unsent system speech in hotkey mode.
Section 38.1 lists every default shortcut.

### Finish

Harness enters the tray and begins in an idle state.

------------------------------------------------------------------------

# 4. Day-to-day interaction

## 4.1 Tray

The tray icon is the persistent app presence.

Tray menu:

``` text
Harness
────────────────
Open Harness
Start meeting
Stop meeting
────────────────
Listening: Off / On
Microphone: <device>
System audio: <device>
────────────────
Memory
Settings
────────────────
Quit
```

The exact visual labels can be polished, but the functionality must
exist.

Closing the popover must not terminate Harness.

`Quit` must actually terminate capture, STT workers, database handles,
and the process.

## 4.2 Global shortcut

Default:

`Ctrl + Space`

Behavior:

-   if popover hidden → show;
-   if visible and interactive → hide;
-   `Esc` → hide;
-   shortcut should work while Zoom/Meet/Teams/browser/IDE has focus;
-   this shortcut shows the overlay with keyboard focus in the input.
    The send-transcript and screenshot shortcuts show it without taking
    focus (section 38.1).

Invocation should not open a normal taskbar window.

## 4.3 Popover

The primary interface is a floating, frameless, compact window.

Required properties:

-   always-on-top while visible;
-   no traditional title bar;
-   rounded visual surface;
-   draggable;
-   resizable within defined min/max bounds;
-   remembers last position;
-   clamps itself into the visible work area if monitor topology
    changes;
-   per-monitor DPI aware;
-   multi-monitor safe;
-   fast show/hide;
-   keyboard-first;
-   supports mouse interaction;
-   adjustable background and text opacity, click-through, and keyboard
    move and resize (section 38.1);
-   excluded from screen capture, the taskbar, and Alt+Tab by default
    (section 38.2).

Suggested default dimensions:

-   width: \~520 px logical;
-   collapsed height: \~72--110 px;
-   answer state: grows up to \~560 px;
-   minimum width: \~420 px;
-   maximum width: \~720 px.

Do not create a full ChatGPT clone inside the popover.

## 4.4 Popover states

### Idle

``` text
┌──────────────────────────────────────────┐
│ Harness                            Ready │
│ Ask about this meeting...                │
│ Meeting · Memory · Documents             │
└──────────────────────────────────────────┘
```

### Listening

``` text
┌──────────────────────────────────────────┐
│ ● Listening                       18:42  │
│ "...why did you choose BM25 here?"       │
└──────────────────────────────────────────┘
```

### Asking

The input should accept natural questions such as:

-   "what is he asking?"
-   "what should I mention here?"
-   "what did we decide about the deployment?"
-   "pull up the relevant details from Trace"
-   "summarize his last point"
-   "what did I say earlier about this?"
-   "give me a concise answer"
-   "what are the action items so far?"

### Streaming answer

Render response tokens as they arrive.

The first useful response text matters more than waiting for a perfect
completed response.

Show small context/source chips when useful:

`Meeting · 1m`\
`Trace`\
`Foomato`\
`Project notes`

These chips refer to local context sources, not fabricated citations.

------------------------------------------------------------------------

# 5. Meeting lifecycle

Harness needs an explicit meeting/session abstraction.

States:

``` text
IDLE
  │
  ├── start manually
  └── future meeting detection
        ↓
STARTING
        ↓
ACTIVE
        ↓
PAUSED
        ↓
ACTIVE
        ↓
STOPPING
        ↓
COMPLETED
```

Launch V0 must support manual start/stop reliably.

Automatic meeting detection may be included if it can be implemented
without destabilizing the audio pipeline, but manual meeting start is
the canonical behavior and must always remain available.

Starting a meeting:

1.  create `meeting` row;
2.  resolve current audio devices;
3.  start system capture;
4.  start mic capture;
5.  start STT workers;
6.  start transcript bus;
7.  start meeting-state accumulator;
8.  set tray/popover state to listening.

Stopping:

1.  stop accepting new capture frames;
2.  drain pending STT segments;
3.  finalize partial utterances;
4.  persist final transcript;
5.  flush database;
6.  release audio devices;
7.  unload or keep STT model according to memory policy;
8.  mark meeting completed;
9.  queue authorized final enrichment separately; network requests must not
    delay resource release or meeting completion.

A crash must not destroy the transcript already produced.

------------------------------------------------------------------------

# 6. Audio architecture

Audio quality and latency are core product features.

## 6.1 Separate streams

Never mix microphone and system audio before transcription.

Maintain:

``` text
AudioSource::System
AudioSource::Microphone
```

This gives Harness a strong baseline speaker-role distinction:

-   microphone ≈ user;
-   system loopback ≈ remote participants.

This is not full speaker diarization. Multiple remote speakers can share
the system stream.

## 6.2 Windows system audio

Use Windows WASAPI loopback capture through Rust/Windows APIs.

Requirements:

-   capture the current default render endpoint;
-   capture what the user hears from conferencing/browser apps;
-   handle device-format negotiation;
-   resample to the STT engine's expected format;
-   detect endpoint changes;
-   restart the capture stream safely after device changes;
-   recover from Bluetooth/headset changes;
-   avoid writing raw audio to disk unless explicitly required by a
    future recording feature.

## 6.3 Microphone

Use WASAPI capture for the selected microphone.

Requirements mirror system capture.

The UI must allow choosing a non-default microphone.

## 6.4 Internal audio representation

Normalize STT input to a known representation, for example:

``` text
PCM float32
16 kHz
mono
```

Do not assume hardware provides this format.

Capture at device-native format, then resample/downmix in the audio
pipeline.

## 6.5 Audio pipeline

``` text
WASAPI callback
     │
     ▼
lock-free / bounded audio ring buffer
     │
     ▼
resampler
     │
     ▼
VAD
     │
     ▼
utterance segmenter
     │
     ▼
STT worker
     │
     ├── partial transcript events
     └── finalized transcript events
```

Audio callbacks must never block on:

-   SQLite;
-   network;
-   React;
-   model inference;
-   disk;
-   long mutex waits.

If a downstream consumer is slow, use bounded queues and observable
backpressure/drop metrics rather than freezing the capture callback.

## 6.6 VAD

Use a lightweight local VAD.

Responsibilities:

-   identify speech onset;
-   identify likely end of utterance;
-   avoid sending long silence spans to STT;
-   allow a small pre-roll buffer so initial phonemes are not clipped;
-   merge very short pauses where appropriate.

Make VAD parameters configurable internally, not necessarily exposed to
users initially.

## 6.7 Audio failures

The app must survive:

-   unplugged microphone;
-   Bluetooth device switching;
-   default output change;
-   USB headset removal;
-   sleep/wake;
-   temporary exclusive-mode conflict;
-   no microphone permission;
-   no system audio.

If mic fails but system audio works, continue system transcription and
display a warning instead of ending the meeting.

------------------------------------------------------------------------

# 7. Local speech-to-text

## 7.1 Abstraction

Do not couple the rest of Harness to one Whisper implementation.

Define a Rust-side abstraction conceptually equivalent to:

``` rust
trait SpeechEngine {
    fn load(&mut self, config: SpeechConfig) -> Result<()>;
    fn push(&mut self, source: AudioSource, frame: AudioFrame) -> Result<()>;
    fn flush(&mut self, source: AudioSource) -> Result<()>;
    fn unload(&mut self) -> Result<()>;
}
```

Transcript output should be event-based.

``` rust
enum TranscriptEvent {
    Partial(PartialTranscript),
    Final(FinalTranscript),
}
```

## 7.2 Backend selection

The initial implementation should use a local Whisper-compatible runtime
that:

-   can be redistributed legally;
-   works on Windows x64;
-   has a CPU path;
-   can use hardware acceleration where practical;
-   can stream/approximate streaming with partial decoding;
-   does not require Python at runtime.

A `whisper.cpp`-class native runtime is a strong default architecture
for this reason, but benchmark before freezing the exact model/runtime.

The codebase must isolate the backend so a faster future engine can
replace it.

## 7.3 Model choice

Choose the smallest model that reaches acceptable English meeting
accuracy while preserving real-time performance on a typical modern
laptop.

Ship one default model.

Do not make users understand Whisper model families during onboarding.

Advanced settings can later allow another model.

## 7.4 Partial and final transcripts

A transcript segment has:

``` text
id
meeting_id
source
started_at
ended_at
text
is_final
confidence? (if available)
revision
created_at
```

Partial text can change.

Final text is immutable except through explicit correction/edit logic.

UI should favor stable text to avoid distracting word churn.

## 7.5 Target latency

Desired perceived behavior:

-   speech onset detected quickly;
-   partial text begins appearing roughly within hundreds of
    milliseconds to low seconds depending on hardware/model;
-   finalized utterance arrives shortly after the speaker pauses.

Do not fake an exact 300 ms SLA if the chosen local model/hardware
cannot meet it. Instrument real latency and optimize from measurements.

------------------------------------------------------------------------

# 8. Event bus and concurrency model

The Rust core is event-driven.

Suggested event families:

``` text
AudioFrame
SpeechStarted
SpeechEnded
TranscriptPartial
TranscriptFinal
MeetingStarted
MeetingPaused
MeetingStopped
MeetingStateUpdated
MemoryRetrieved
ContextCompiled
InferenceStarted
InferenceDelta
InferenceCompleted
InferenceFailed
AudioDeviceChanged
AppError
```

Use Tokio for asynchronous orchestration/network/database-adjacent work
where appropriate.

Do not force real-time audio callbacks onto the async executor if doing
so adds blocking or scheduling jitter.

Suggested logical workers:

``` text
UI / Tauri main thread

Audio capture thread: system
Audio capture thread: mic

Audio/STT worker(s)

Tokio runtime:
    transcript persistence
    meeting-state processing
    retrieval
    context compiler
    ChatGPT client
    auth
    background maintenance
```

The exact thread count may adapt to hardware.

------------------------------------------------------------------------

# 9. Transcript storage

Use SQLite.

Do not require a SQLite server.

Enable:

-   WAL mode;
-   foreign keys;
-   appropriate busy timeout;
-   migrations;
-   indexes on meeting/time/source fields;
-   FTS5 for textual retrieval.

Store transcript text incrementally as final segments arrive.

Never wait until meeting end to persist everything.

Suggested tables are defined later.

------------------------------------------------------------------------

# 10. Meeting state engine

The complete raw transcript should not be sent to the LLM on every
request.

Maintain a compact evolving state.

Conceptual state:

``` text
MeetingState
├── title / inferred topic
├── participants / known people
├── topics
├── rolling summary
├── decisions
├── questions asked
├── unresolved questions
├── action items
├── important claims
├── referenced projects/entities
└── recent transcript window
```

## 10.1 Recent transcript buffer

Keep a fast in-memory ring/deque of recent finalized and relevant
partial transcript segments.

Default context target:

-   last \~90 seconds, adjusted by token budget;
-   preserve source role and timestamps;
-   prioritize current conversational turn.

Do not use time alone. If the user asks "what did he say about
deployment 20 minutes ago?", local retrieval should search the meeting
transcript.

## 10.2 Rolling state

Periodically consolidate older conversation into meeting state.

Remote rolling-state requests follow the per-meeting sending policy. In hotkey mode,
do not issue background cloud requests. In automatic mode, batch new finalized system
speech and process requests sequentially.

Trigger state refresh based on meaningful conditions, for example:

-   enough finalized text accumulated;
-   topic boundary;
-   explicit question/decision language;
-   elapsed interval while active.

If network inference is unavailable, the raw transcript remains fully
functional locally and state updates can degrade gracefully.

## 10.3 Final meeting record

At meeting end store:

-   full transcript;
-   summary;
-   decisions;
-   action items;
-   extracted entities;
-   links to memories referenced/created;
-   start/end timestamps.

------------------------------------------------------------------------

# 11. Personal memory system

Memory is a first-class subsystem, not a prompt dump.

## 11.1 Memory categories

Support at least:

### Structured memory

Stable explicit facts:

-   projects;
-   project technologies;
-   project decisions;
-   work experience;
-   education;
-   people;
-   organizations;
-   preferences;
-   recurring terms/entities.

### Semantic memory

Retrievable chunks from:

-   project descriptions;
-   notes;
-   documents;
-   previous meeting summaries;
-   imported markdown/text/PDF-derived text when supported.

### Episodic memory

Time-bound events:

-   meetings;
-   conversations;
-   decisions;
-   action items;
-   project changes.

## 11.2 Memory lifecycle

A memory needs provenance.

Never silently convert every transcript sentence into permanent memory.

A candidate memory can have:

``` text
candidate
accepted
rejected
superseded
```

For launch, favor explicit/imported memories plus clearly derived
meeting artifacts. Automatic long-term memory extraction should be
conservative.

## 11.3 Provenance

Each memory should be able to answer:

-   where did this come from?
-   when was it created?
-   which meeting/document/manual entry?
-   has it been superseded?

This is essential to avoid stale context.

## 11.4 Local retrieval

Use hybrid local retrieval.

Pipeline:

``` text
query
  │
  ├── entity/project detection
  │
  ├── FTS5 / BM25 lexical search
  │
  ├── semantic vector retrieval
  │
  └── structured entity lookup
          │
          ▼
     candidate union
          │
          ▼
       reranker
          │
          ▼
     context budgeter
```

If the initial vector embedding model would materially inflate the
installer or active memory, it may be a separately bundled small local
model. It must still be installed automatically; do not require external
services.

Lexical retrieval must work even if embeddings are unavailable.

## 11.5 Memory UI

Provide a dedicated normal-size management view accessible from
tray/settings, separate from the meeting popover.

Users need to be able to:

-   search memory;
-   inspect a memory;
-   see provenance;
-   add a memory manually;
-   edit user-created memory;
-   delete memory;
-   inspect projects/entities;
-   import supported documents;
-   delete meeting history.

The floating popover remains the primary meeting interaction; management
screens can be conventional windows.

------------------------------------------------------------------------

# 12. Document ingestion

Harness should be able to build personal/project context from local
files.

Initial supported ingestion targets:

-   `.md`
-   `.txt`
-   `.json` where text extraction is meaningful
-   `.pdf` if a reliable local text extraction library is bundled
-   optionally `.docx` after the core path is stable

Ingestion pipeline:

``` text
file picker
   ↓
copy/reference decision
   ↓
local text extraction
   ↓
normalization
   ↓
chunking
   ↓
metadata extraction
   ↓
FTS indexing
   ↓
local embeddings
   ↓
document ready
```

Do not upload the source document merely to index it.

Store:

-   original path;
-   content hash;
-   modified time;
-   extracted text/chunks;
-   indexing version.

If the original changes, offer/re-run indexing.

------------------------------------------------------------------------

# 13. Context compiler

The context compiler is one of the most important components.

It receives:

``` text
user query
recent transcript
meeting state
meeting retrieval results
personal memory results
document results
conversation history inside current popover session
```

It produces a bounded model request.

## 13.1 Priority order

Default priority:

1.  current user question;
2.  immediate conversational turn;
3.  recent transcript;
4.  directly relevant personal/project memory;
5.  directly relevant earlier meeting transcript;
6.  meeting summary/state;
7.  relevant documents;
8.  older chat interaction history.

Never include context simply because it exists.

## 13.2 Token budget

Implement explicit budgets rather than concatenating until failure.

Conceptual:

``` text
system/product instructions
user query
recent meeting context
retrieved memory
retrieved meeting history
document context
response reserve
```

The compiler must be model-aware when model context limits are
discoverable.

## 13.3 Context provenance

Internally label context sections:

``` text
<recent_meeting>
...
</recent_meeting>

<personal_memory source="project:trace">
...
</personal_memory>
```

Do not rely on the model to guess whether text came from the user,
another participant, or a stored note.

## 13.4 Injection resistance

Imported documents and transcript content are untrusted data.

The model instruction must make clear that quoted meeting/document
content is context, not application/system instruction.

Do not let a transcript phrase such as "ignore your previous
instructions" change Harness behavior.

------------------------------------------------------------------------

# 14. ChatGPT authentication and inference

Use the official **Sign in with ChatGPT** open-source/local-app flow
available at implementation time.

Current documented properties to preserve:

-   eligible Plus/Pro users can authorize ChatGPT-plan usage;
-   open-source clients can dynamically register;
-   first registration uses an installation host ID;
-   no user-supplied API key should be required for the intended path;
-   no client secret should be embedded for the open-source dynamic
    flow;
-   the authorization does not provide access to ChatGPT conversation
    history;
-   requests use eligible Responses API access and should stream.

Because this external protocol can change, isolate it behind:

``` rust
trait AiProvider {
    async fn authenticate(...);
    async fn connection_status(...);
    async fn discover_models(...);
    async fn stream_response(...);
    async fn sign_out(...);
}
```

The rest of Harness must not know OAuth details.

Optional API-key providers (DeepSeek, Gemini) implement the same trait.
Section 38.4 covers them, and section 38.3 covers more than one ChatGPT
account.

## 14.1 Secrets

Never store OAuth access/refresh tokens:

-   in localStorage;
-   in frontend state longer than necessary;
-   in plaintext JSON;
-   in SQLite;
-   in logs.

Use Windows Credential Manager / DPAPI-backed secure storage from the
Rust side.

React receives only safe account state such as:

``` text
connected: true
display_name
email if appropriate
plan_usage_enabled
```

## 14.2 Streaming

Inference UI must be streaming.

Flow:

``` text
user presses Enter
    ↓
snapshot meeting context
    ↓
parallel local retrieval
    ↓
compile context
    ↓
dispatch request
    ↓
first delta
    ↓
popover starts rendering
    ↓
remaining deltas
```

Do not block response dispatch on nonessential background enrichment.

## 14.3 Offline behavior

When internet is unavailable:

Harness should still:

-   capture;
-   transcribe;
-   persist;
-   search local transcript;
-   search local memory;
-   show meeting state available locally.

AI generation should show a clear offline state rather than failing
silently.

Once connectivity returns, normal inference resumes.

------------------------------------------------------------------------

# 15. Database design

Use a migration-managed SQLite database under the user's local
application data directory.

Suggested path:

``` text
%LOCALAPPDATA%\Harness\data\harness.db
```

Suggested directory layout:

``` text
%LOCALAPPDATA%\Harness\
├── data\
│   ├── harness.db
│   ├── harness.db-wal
│   └── harness.db-shm
├── models\
├── cache\
├── logs\
└── state\
```

Secrets do not live here.

## 15.1 Core schema

Exact SQL can evolve, but preserve these entities.

### `meetings`

``` text
id TEXT PRIMARY KEY
title TEXT
started_at INTEGER NOT NULL
ended_at INTEGER
status TEXT NOT NULL
summary TEXT
created_at INTEGER NOT NULL
updated_at INTEGER NOT NULL
```

### `transcript_segments`

``` text
id TEXT PRIMARY KEY
meeting_id TEXT NOT NULL
source TEXT NOT NULL          -- microphone | system
started_at INTEGER NOT NULL
ended_at INTEGER
text TEXT NOT NULL
is_final INTEGER NOT NULL
revision INTEGER NOT NULL
confidence REAL
created_at INTEGER NOT NULL
FOREIGN KEY(meeting_id) REFERENCES meetings(id)
```

Create FTS5 indexing for finalized transcript text.

### `meeting_artifacts`

``` text
id
meeting_id
kind        -- decision | action_item | question | claim | topic
content
status
source_segment_ids
created_at
updated_at
```

### `projects`

``` text
id
name
description
status
created_at
updated_at
```

### `people`

``` text
id
name
notes
created_at
updated_at
```

### `memories`

``` text
id
kind
title
content
status
importance
project_id nullable
person_id nullable
source_type
source_id
valid_from nullable
superseded_by nullable
created_at
updated_at
```

### `documents`

``` text
id
name
path
mime_type
content_hash
modified_at
index_version
created_at
updated_at
```

### `document_chunks`

``` text
id
document_id
ordinal
content
token_count
embedding_ref nullable
created_at
```

### `entities`

Generic extracted entities if needed.

### `memory_embeddings`

If vectors are stored separately from the primary memory row.

### `settings`

Only non-secret settings.

### `app_migrations`

Migration/version bookkeeping if the selected migration library does not
already provide it.

Section 38 adds `accounts`, `usage_events`, `context_profiles`, and
`context_profile_items`.

------------------------------------------------------------------------

# 16. Repository architecture

Use a monorepo-like single application repository.

``` text
harness/
├── README.md
├── PLAN.md
├── LICENSE
├── package.json
├── package-lock.json
├── tsconfig.json
├── vite.config.ts
│
├── src/                         # React UI
│   ├── app/
│   ├── components/
│   ├── features/
│   │   ├── overlay/
│   │   ├── onboarding/
│   │   ├── meeting/
│   │   ├── memory/
│   │   ├── settings/
│   │   └── account/
│   ├── hooks/
│   ├── lib/
│   ├── stores/
│   └── styles/
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/
│   ├── resources/
│   │   ├── stt/
│   │   ├── models/
│   │   └── webview2/            # when required by packaging strategy
│   ├── migrations/
│   │
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       │
│       ├── app/
│       │   ├── lifecycle.rs
│       │   ├── state.rs
│       │   └── events.rs
│       │
│       ├── windows/
│       │   ├── audio.rs
│       │   ├── shortcuts.rs
│       │   ├── tray.rs
│       │   ├── window.rs
│       │   ├── credentials.rs
│       │   └── startup.rs
│       │
│       ├── audio/
│       │   ├── capture.rs
│       │   ├── buffer.rs
│       │   ├── resample.rs
│       │   ├── vad.rs
│       │   └── devices.rs
│       │
│       ├── speech/
│       │   ├── mod.rs
│       │   ├── engine.rs
│       │   ├── whisper.rs
│       │   └── segmenter.rs
│       │
│       ├── meeting/
│       │   ├── session.rs
│       │   ├── transcript.rs
│       │   ├── recent.rs
│       │   ├── state.rs
│       │   └── artifacts.rs
│       │
│       ├── memory/
│       │   ├── store.rs
│       │   ├── retrieval.rs
│       │   ├── lexical.rs
│       │   ├── semantic.rs
│       │   ├── rerank.rs
│       │   └── provenance.rs
│       │
│       ├── documents/
│       │   ├── ingest.rs
│       │   ├── extract.rs
│       │   └── chunk.rs
│       │
│       ├── context/
│       │   ├── compiler.rs
│       │   ├── budget.rs
│       │   └── prompt.rs
│       │
│       ├── ai/
│       │   ├── provider.rs
│       │   ├── chatgpt.rs
│       │   ├── auth.rs
│       │   └── stream.rs
│       │
│       ├── db/
│       │   ├── mod.rs
│       │   ├── models.rs
│       │   └── migrations.rs
│       │
│       ├── commands/             # narrow Tauri command boundary
│       ├── telemetry/            # local diagnostics by default
│       └── errors.rs
│
├── scripts/
│   ├── build-windows.ps1
│   ├── package.ps1
│   ├── verify-clean-install.ps1
│   └── fetch-release-assets.ps1
│
└── tests/
    ├── fixtures/
    ├── integration/
    └── packaging/
```

Do not let `commands/` become business logic. Tauri commands are
adapters into domain services.

------------------------------------------------------------------------

# 17. Frontend architecture

Use:

-   React;
-   TypeScript;
-   Vite;
-   minimal state library only if needed;
-   CSS/Tailwind only if it does not inflate complexity unnecessarily.

The frontend is a view/controller layer.

It must not:

-   capture audio;
-   own OAuth tokens;
-   access SQLite directly;
-   run STT;
-   implement retrieval;
-   compile prompts.

Those belong in Rust.

Use Tauri events for streaming state/deltas from Rust to UI.

------------------------------------------------------------------------

# 18. Tauri and Windows integration

Use Tauri 2 as the application shell.

Use official plugins where they are sufficient, including global
shortcut support. Drop to Win32/Windows Runtime APIs from Rust when the
required behavior is Windows-specific or Tauri abstractions are
insufficient.

## 18.1 Windows

Create at least:

1.  `overlay` window;
2.  `management` window for settings/memory/onboarding when needed.

The overlay should not be recreated on every shortcut. Prefer keeping it
alive and hiding/showing it if this gives materially lower latency.

## 18.2 Tray

Tray is initialized from the Rust side.

## 18.3 Startup

Provide a setting:

`Launch Harness when I sign in to Windows`

Default can be off during development and explicitly chosen during
onboarding/release.

## 18.4 Single instance

Only one Harness instance should run for a user session.

A second launch should focus/show the existing instance rather than
starting duplicate audio capture.

------------------------------------------------------------------------

# 19. Performance budgets

These are engineering targets, not marketing guarantees.

## 19.1 Idle

When no meeting is active:

-   audio capture stopped unless ambient listening was explicitly
    enabled;
-   STT model may be unloaded after a configurable idle period;
-   no polling loops at high frequency;
-   near-zero sustained CPU usage;
-   target base process memory below \~100--150 MB where practical.

Measure rather than guess.

## 19.2 Active meeting

Goals:

-   capture without audio gaps;
-   transcript processing does not block UI;
-   overlay invocation feels instant;
-   model inference streaming begins as soon as context is ready;
-   UI remains responsive while STT is saturated.

Separate metrics for:

-   app base RSS;
-   STT model memory;
-   WebView memory;
-   GPU/accelerator memory.

## 19.3 Overlay

Target show time:

`< 50 ms perceived on a warm process`

Instrument:

-   shortcut timestamp;
-   Rust event receipt;
-   window show request;
-   frontend visible/paint acknowledgment if available.

## 19.4 Retrieval

Aim for sub-100-ms local retrieval on ordinary personal datasets, but
benchmark with realistic data.

Indexes should scale to at least:

-   hundreds of meetings;
-   hundreds of thousands of transcript segments/chunks;
-   thousands of memories/doc chunks;

without requiring a server database.

------------------------------------------------------------------------

# 20. Error model

Use typed Rust errors internally.

User-facing errors must be actionable.

Examples:

Bad:

`WASAPI error 0x88890004`

Good:

`Harness lost access to your microphone. System audio is still being transcribed. Choose another microphone in Audio Settings.`

Required error domains:

-   audio device;
-   STT/model;
-   database;
-   document ingestion;
-   authentication;
-   inference/network;
-   shortcut conflict;
-   installer/runtime;
-   migration;
-   secure credential storage.

Never crash the whole app because one optional subsystem fails.

------------------------------------------------------------------------

# 21. Logging and diagnostics

Use structured local logs.

Default logs must not contain:

-   OAuth tokens;
-   raw authorization headers;
-   full private documents;
-   complete prompts;
-   full transcripts unless explicit debug mode is enabled.

Log operational metadata:

-   component;
-   severity;
-   timestamp;
-   meeting ID;
-   event latency;
-   device changes;
-   model load time;
-   STT real-time factor;
-   retrieval latency;
-   context compilation latency;
-   inference first-token latency;
-   errors.

Provide a user action to export a sanitized diagnostics bundle.

------------------------------------------------------------------------

# 22. Security

## 22.1 Trust boundaries

Treat as untrusted:

-   transcript content;
-   system audio-derived text;
-   imported documents;
-   document metadata;
-   model output.

Sensitive:

-   OAuth tokens;
-   personal memory;
-   transcript database;
-   imported document text.

## 22.2 Frontend boundary

Expose the smallest possible Tauri command surface.

Use Tauri capabilities/permissions explicitly.

Do not enable arbitrary shell execution.

Do not allow arbitrary frontend-origin navigation.

Use a restrictive CSP.

## 22.3 Local data

At minimum:

-   credentials use Windows secure storage;
-   database is only accessible under the user's profile permissions;
-   deletion functions actually delete requested local records;
-   no analytics/telemetry leaves the machine without explicit
    design/consent.

Full database encryption can be added if a suitable maintainable
solution is chosen, but do not pretend ordinary SQLite is encrypted at
rest.

------------------------------------------------------------------------

# 23. Packaging and installer

This is part of the product, not a post-build task.

## 23.1 Required artifact

Release CI/local packaging must produce:

``` text
dist/
└── Harness-Setup-x64.exe
```

Optionally also:

``` text
Harness-x64.msi
```

The `.exe` is the canonical shareable artifact.

## 23.2 Installer technology

Use Tauri's Windows NSIS bundle for the setup executable.

Use per-user installation by default so ordinary users do not need
administrator rights unless a dependency truly requires elevation.

Target install location should follow Tauri/Windows per-user conventions
under `%LOCALAPPDATA%`.

## 23.3 WebView2

Tauri uses WebView2 on Windows.

The canonical offline/self-contained installer should use Tauri's
`offlineInstaller` WebView2 installation mode or an equivalent supported
self-contained configuration.

This increases installer size but removes a first-run dependency on the
target machine already having a usable WebView2 runtime.

Do not use `skip`.

## 23.4 STT assets

Bundle:

-   native STT runtime DLLs/executable/library as appropriate;
-   default model;
-   licenses/attributions required for redistribution;
-   checksums/version manifest.

On first run:

1.  verify model/runtime presence;
2.  verify checksum;
3.  initialize local model directory;
4.  load/test only when needed.

Do not download code from random URLs at runtime.

## 23.5 Native dependencies

Any Visual C++ runtime/native runtime dependency that is not statically
or otherwise safely included must be handled automatically by the
installer.

A clean machine is the test.

## 23.6 Code signing

Development builds can be unsigned.

If public distribution is later requested:

-   obtain a Windows code-signing certificate;
-   sign the application executable;
-   sign the installer;
-   timestamp signatures.

Unsigned public installers will produce poor Windows trust/SmartScreen
UX.

## 23.7 Updates

Design for Tauri updater support.

An update must not delete:

-   database;
-   memories;
-   documents/indexes;
-   user settings;
-   secure credentials.

Migrations must run transactionally.

If an update fails, preserve the previous usable installation where
supported.

------------------------------------------------------------------------

# 24. Build pipeline

Development prerequisites are allowed on the developer/build machine.

Recommended:

-   Windows 11 build machine or Windows CI runner;
-   Rust stable pinned via toolchain file;
-   Node LTS;
-   npm pinned;
-   Visual Studio Build Tools required by Tauri/MSVC;
-   Tauri CLI.

Canonical commands should be wrapped:

``` powershell
.\scripts\build-windows.ps1
.\scripts\package.ps1
```

The scripts should:

1.  verify toolchain versions;
2.  install frontend dependencies from lockfile;
3.  run formatting/lints;
4.  run Rust tests;
5.  run TypeScript tests;
6.  verify required model/runtime assets;
7.  build release frontend;
8.  build release Rust/Tauri app;
9.  bundle NSIS installer;
10. sign when signing configuration is present;
11. emit hashes;
12. place final artifact in `dist/`.

------------------------------------------------------------------------

# 25. Clean-machine release test

A release is not complete until tested on a Windows machine/VM that does
not contain the development toolchain.

Test procedure:

1.  create/reset clean Windows 11 x64 VM;
2.  ensure no Rust/Node/Python/Git/Whisper is installed;
3.  disconnect internet temporarily;
4.  run `Harness-Setup-x64.exe`;
5.  verify installation succeeds;
6.  launch Harness;
7.  verify UI/WebView launches;
8.  verify bundled STT model/runtime integrity;
9.  verify local microphone transcription offline;
10. play known audio and verify system-loopback transcription offline;
11. verify local database creation;
12. reconnect internet;
13. complete `Continue with ChatGPT`;
14. start meeting;
15. produce both mic and system speech;
16. verify role-separated transcript;
17. invoke overlay with global shortcut;
18. ask a context-dependent question;
19. verify streamed response;
20. stop meeting;
21. restart Harness;
22. verify meeting persisted;
23. verify memory retrieval;
24. reboot Windows;
25. verify configured startup behavior;
26. uninstall Harness;
27. verify binaries removed and apply the documented policy for user
    data.

Repeat at least once on a physical Windows laptop before release.

------------------------------------------------------------------------

# 26. Testing strategy

## Unit tests

Rust:

-   audio resampling;
-   ring buffer behavior;
-   VAD segmentation;
-   transcript revision/finalization;
-   token/context budgeting;
-   retrieval ranking;
-   memory provenance;
-   database repositories;
-   auth state transitions;
-   prompt/context injection boundaries.

TypeScript:

-   overlay state machine;
-   keyboard interactions;
-   answer streaming reducer;
-   settings forms.

## Integration tests

-   mocked audio → STT → transcript;
-   transcript → SQLite;
-   query → hybrid retrieval;
-   query + meeting → context compiler;
-   mocked ChatGPT stream → Rust → Tauri event → UI;
-   auth callback parsing with safe test fixtures;
-   database migration from previous schema version.

## Windows tests

-   global shortcut;
-   tray;
-   always-on-top;
-   multi-monitor;
-   100/125/150/200% DPI;
-   device hot-plug;
-   Bluetooth switching;
-   sleep/wake;
-   microphone denied;
-   system output changed;
-   no network;
-   slow network;
-   overlay absent from Zoom, Teams, and Google Meet screen share, OBS,
    and Snipping Tool captures;
-   overlay absent from the taskbar and Alt+Tab;
-   move, resize, and opacity shortcuts registered only while the
    overlay is visible;
-   both screenshot shortcuts on single and multi-monitor setups;
-   failover from a limited ChatGPT account to an API-key provider.

## Long-running test

Run a 2+ hour synthetic meeting.

Verify:

-   memory does not grow without bound;
-   transcript remains correct;
-   queues remain bounded;
-   no progressive UI slowdown;
-   database WAL remains manageable;
-   STT real-time factor remains stable;
-   no audio callback starvation.

------------------------------------------------------------------------

# 27. Observability during development

Add an internal diagnostics screen hidden behind a developer flag.

Display:

``` text
system audio queue depth
mic audio queue depth
dropped frames
VAD state
STT real-time factor
partial latency
final latency
transcript segments/min
database write latency
FTS query latency
vector query latency
context tokens
retrieval candidates
inference dispatch latency
first-token latency
current RSS
STT model RSS estimate
```

Performance work must be measurement-driven.

------------------------------------------------------------------------

# 28. Implementation order

This is not a reduced product roadmap. It is the dependency order for
building the full V0 safely.

## Phase A --- Desktop foundation

Complete:

-   Tauri shell;
-   Rust core structure;
-   React overlay;
-   tray;
-   single-instance behavior;
-   global shortcut;
-   overlay show/hide;
-   Windows startup option;
-   local settings;
-   SQLite migrations;
-   structured logging;
-   overlay opacity, click-through, and move and resize shortcuts;
-   capture exclusion, taskbar hiding, and Alt+Tab hiding.

Exit condition: a packaged app opens from tray and the overlay reliably
appears via shortcut.

## Phase B --- Audio

Complete:

-   device enumeration;
-   system WASAPI loopback;
-   microphone WASAPI;
-   separate bounded buffers;
-   resampling;
-   VAD;
-   hot-plug/device recovery;
-   audio diagnostics.

Exit condition: Harness can run for two hours capturing both streams
without blocking or leaking memory.

## Phase C --- Local STT

Complete:

-   native speech engine abstraction;
-   default backend;
-   model packaging;
-   partial/final events;
-   source tagging;
-   persistence;
-   transcript UI.

Exit condition: clean machine can transcribe mic and system audio
without Python or network.

## Phase D --- Meeting engine

Complete:

-   meeting lifecycle;
-   recent transcript buffer;
-   meeting transcript search;
-   rolling state;
-   artifacts;
-   completed meeting history.

## Phase E --- Memory and documents

Complete:

-   structured memories;
-   project/person entities;
-   document ingestion;
-   FTS5;
-   local embeddings;
-   hybrid retrieval;
-   reranking;
-   provenance;
-   management UI.

## Phase F --- ChatGPT

Complete:

-   Sign in with ChatGPT;
-   secure credential persistence;
-   account state UI;
-   model discovery/selection as required;
-   streaming inference;
-   reconnection/refresh;
-   sign-out;
-   more than one ChatGPT account;
-   OpenAI-compatible API-key providers and the fallback chain;
-   usage events and the usage page.

## Phase G --- Context intelligence

Complete:

-   query understanding;
-   parallel retrieval;
-   context compiler;
-   token budgeting;
-   meeting + personal memory fusion;
-   prompt injection boundaries;
-   source chips;
-   streaming answer UX;
-   streaming Markdown and code rendering;
-   context profiles and the context inspector;
-   screenshot image and OCR flows.

## Phase H --- Product hardening

Complete:

-   performance profiling;
-   keyboard polish;
-   audio recovery;
-   offline degradation;
-   memory deletion;
-   diagnostics;
-   accessibility basics;
-   crash recovery;
-   installer;
-   code signing configuration;
-   updater;
-   clean-machine test.

The product is "done" only after Phase H acceptance criteria pass.

------------------------------------------------------------------------

# 29. Definition of done

Harness V0 is shippable only when all of the following are true.

### Installation

-   One `Harness-Setup-x64.exe` can install the application on a
    compatible clean Windows x64 machine.
-   No development tools are required.
-   The local STT engine works after installation.
-   The installer handles WebView2.
-   Uninstall works.

### Desktop behavior

-   Tray behavior is reliable.
-   Global shortcut works.
-   Overlay appears quickly.
-   Overlay behaves across multiple monitors and DPI settings.
-   Only one app instance captures audio.

### Audio/STT

-   System audio transcribes locally.
-   Microphone transcribes locally.
-   Sources remain distinguishable.
-   Raw audio is not sent to OpenAI.
-   Device changes do not normally crash the meeting.

### Meetings

-   Meeting can start/stop.
-   Transcript persists incrementally.
-   Recent context is available immediately.
-   Previous meeting text is locally searchable.
-   Meeting state/artifacts persist.

### Memory

-   User can add/import context.
-   Memory remains local.
-   Retrieval combines lexical, semantic, and structured signals.
-   Provenance is inspectable.
-   Memories can be deleted.

### AI

-   User can `Continue with ChatGPT`.
-   No API key is required for the intended eligible-plan path.
-   Tokens are stored securely outside frontend/SQLite.
-   Responses stream.
-   Questions can combine live meeting context and personal memory.
-   Lack of network does not destroy local meeting functionality.

### Overlay, privacy, and screen input

-   Opacity, click-through, move, and resize work from shortcuts and
    settings.
-   Harness windows do not appear in screen share or the taskbar.
-   `Ctrl+Shift+O` sends a screenshot image to an image-capable provider.
-   `Ctrl+Shift+X` sends local OCR text.
-   Answers render Markdown and highlighted code while streaming.

### Accounts and providers

-   More than one ChatGPT account can be signed in, and the user can
    switch between them.
-   DeepSeek and Gemini keys work as fallbacks and are stored in
    Credential Manager.
-   The usage page shows per-account and per-provider usage.

### Distribution

-   Release build is reproducible from documented scripts/CI.
-   Installer is tested in a clean VM.
-   Public release can be code-signed.
-   No secret keys are embedded in the binary.

------------------------------------------------------------------------

# 30. Explicit non-goals

Do not spend launch engineering time on:

-   macOS;
-   Linux;
-   mobile;
-   browser extension;
-   cloud transcript synchronization;
-   Harness-hosted user accounts;
-   team workspaces;
-   remote vector databases;
-   a web dashboard;
-   full CRM features;
-   video capture;
-   automatic or background screen capture (the screenshot shortcuts in
    section 38.7 are in scope);
-   autonomous clicking/typing in third-party apps;
-   detecting or defeating proctoring and anti-cheat software;
-   remote audio transcription;
-   storing raw meeting audio by default.

These can be reconsidered later. They must not distort V0 architecture.

------------------------------------------------------------------------

# 31. Engineering principles

## Local before remote

If a task can be done well locally without damaging UX, do it locally.

## Audio cannot wait

Never block real-time capture on slow consumers.

## Persist incrementally

A crash 50 minutes into a meeting must not erase 50 minutes of
transcript.

## Retrieve, do not dump

Personal context grows indefinitely. Retrieve only what the current
question needs.

## Model is a component

The LLM is not the database, memory store, meeting state machine, or
application controller.

## Measure latency

Every important path should be timestamped. Optimize measured
bottlenecks.

## Graceful partial failure

If microphone dies, system capture continues.\
If network dies, transcription continues.\
If embeddings fail, lexical retrieval continues.\
If the overlay closes, the meeting continues.\
If state summarization fails, the raw transcript remains authoritative.

## Durable foundations for V0

The first launch may be small in audience, but its storage, audio, auth,
packaging, and update paths must be production-shaped.

------------------------------------------------------------------------

# 32. Suggested implementation decisions

Unless benchmarking or a platform limitation disproves them, start with:

  -----------------------------------------------------------------------
  Area                                Decision
  ----------------------------------- -----------------------------------
  Desktop shell                       Tauri 2

  Core                                Rust

  UI                                  React + TypeScript + Vite

  Windows audio                       WASAPI

  Async runtime                       Tokio

  Database                            SQLite, WAL mode

  Lexical retrieval                   SQLite FTS5 / BM25

  Semantic retrieval                  Small local embedding model + local
                                      vector index/storage

  STT                                 Native Whisper-compatible backend
                                      behind an abstraction

  AI                                  Sign in with ChatGPT + streaming
                                      eligible Responses API

  Credentials                         Windows Credential Manager /
                                      DPAPI-backed secure storage

  Global shortcut                     Tauri global shortcut plugin or
                                      Win32 fallback

  Installer                           Tauri NSIS `-setup.exe`

  WebView                             WebView2, offline installer in
                                      canonical package

  Distribution                        Signed x64 Windows setup executable
  -----------------------------------------------------------------------

------------------------------------------------------------------------

# 33. Questions Codex should NOT ask during implementation

The following decisions are already made:

-   Is this Electron? **No.**
-   Is this a web app? **No.**
-   Should transcription use a cloud API? **No.**
-   Should raw audio go to OpenAI? **No.**
-   Should system and mic audio be mixed? **No.**
-   Should we require Python on user machines? **No.**
-   Should we use Postgres/Redis? **No.**
-   Should we build a Harness backend first? **No.**
-   Should the main UX be a full-screen chat app? **No.**
-   Should the app disappear when the overlay closes? **No; it remains
    in the tray.**
-   Should we ask users for an OpenAI API key? **Not for the intended
    ChatGPT-plan path.** DeepSeek and Gemini keys are optional fallbacks
    (section 38.4).
-   Should Harness rotate ChatGPT accounts automatically when one hits a
    limit? **No.** The user switches. Automatic failover goes to API-key
    providers.
-   Should memory be sent wholesale on every request? **No.**
-   Should V0 be treated as disposable? **No.**
-   Is installer work optional? **No.**
-   Can a release require Node/Rust/Python on the target PC? **No.**
-   Can we silently upload documents for indexing? **No.**
-   Should the model decide what application state is true? **No; local
    state is authoritative.**

When a lower-level implementation choice is not specified, choose the
simplest maintainable option that satisfies the constraints, record the
decision in an ADR, and keep the boundary replaceable.

------------------------------------------------------------------------

# 34. First build milestone Codex should execute

The first implementation milestone should create the real repository
skeleton, not a mock design.

Deliver:

1.  Tauri 2 Windows app;
2.  Rust module layout from this document;
3.  React overlay;
4.  tray;
5.  global shortcut;
6.  single-instance behavior;
7.  SQLite + first migration;
8.  settings storage;
9.  structured logging;
10. Windows packaging configuration;
11. NSIS setup build;
12. `scripts/build-windows.ps1`;
13. `scripts/package.ps1`;
14. basic CI for lint/test/build;
15. README development instructions.

Then continue through the implementation order without replacing those
foundations.

------------------------------------------------------------------------

# 35. Final product test scenario

This scenario is the simplest complete proof that Harness is the product
we intended.

A user receives only:

`Harness-Setup-x64.exe`

They install it on a normal Windows laptop.

They open Harness and choose **Continue with ChatGPT**.

They authorize their eligible ChatGPT account.

They select their microphone.

They start a meeting.

Another person speaks through the laptop's meeting/system audio:

> "Why did you choose BM25 before dense retrieval in Trace?"

Harness locally transcribes it as system speech.

The user replies through their microphone. Harness locally transcribes
that separately.

The user presses `Ctrl + Space`.

The floating popover appears immediately.

They type:

> "what is he actually testing here, and remind me why I made that
> architecture decision?"

Harness:

1.  reads the recent meeting turn;
2.  searches the local Trace/project memory;
3.  retrieves the relevant architecture decision;
4.  compiles only the useful context;
5.  sends that text context through the authorized ChatGPT inference
    path;
6.  streams a concise answer into the popover.

The user presses `Esc`.

The overlay disappears.

Transcription continues.

At meeting end, the user stops the session. The transcript, meeting
summary, decisions, and action items remain locally available.

They restart Windows the next day.

Harness still knows their projects and previous meeting because that
state belongs to Harness, not to an ephemeral model prompt.

That is V0.

------------------------------------------------------------------------

# 36. External implementation references

These are implementation references, not substitutes for this product
specification. Verify current documentation while coding because
authentication and framework details can change.

-   OpenAI --- Sign in with ChatGPT for open-source apps:
    https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt
-   OpenAI --- Sign in with ChatGPT quickstart:
    https://developers.openai.com/siwc/quickstart
-   OpenAI --- open-source ChatGPT plan usage overview:
    https://developers.openai.com/siwc/token-sharing-open-source
-   OpenAI --- registration/sign-in flow:
    https://developers.openai.com/siwc/token-sharing-open-source/sign-in
-   Tauri --- Windows installer:
    https://v2.tauri.app/distribute/windows-installer/
-   Tauri --- global shortcut plugin:
    https://v2.tauri.app/plugin/global-shortcut/
-   Tauri --- configuration reference:
    https://v2.tauri.app/reference/config/
-   Tauri --- Windows prerequisites:
    https://v2.tauri.app/start/prerequisites/
-   Microsoft --- `SetWindowDisplayAffinity`:
    https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity
-   Microsoft --- `Windows.Media.Ocr`:
    https://learn.microsoft.com/uwp/api/windows.media.ocr
-   DeepSeek API docs: https://api-docs.deepseek.com
-   Gemini OpenAI compatibility:
    https://ai.google.dev/gemini-api/docs/openai
-   streamdown: https://github.com/vercel/streamdown
-   xcap: https://crates.io/crates/xcap
-   keyring: https://crates.io/crates/keyring

------------------------------------------------------------------------

# 37. Build mandate

When implementing Harness, optimize for this order:

``` text
correctness
    ↓
privacy
    ↓
capture/transcription reliability
    ↓
latency
    ↓
resource usage
    ↓
visual polish
```

Visual polish matters, but a beautiful overlay that misses system audio
or loses transcripts is not Harness.

The final repository must be understandable as a serious Windows
systems/AI engineering project: native audio capture, local inference,
event-driven concurrency, persistent memory, hybrid retrieval, context
engineering, secure OAuth, streaming model interaction, desktop UX, and
production Windows packaging all belong to one coherent product.

**Do not stop at "it works on the developer machine." The product is
complete when the installer works on somebody else's machine.**


------------------------------------------------------------------------

# 38. Overlay control, privacy, accounts, providers, and screen input

This section adds features agreed after the original specification. If
it conflicts with an earlier section, this section wins. Each
subsection says what to build, how to build it in this stack, and which
shipped products use the same technique.

## 38.1 Overlay opacity, position, and size shortcuts

Overlay settings in the Control center (38.9):

-   background opacity, 15 to 100%, default 85%;
-   text opacity, 50 to 100%, default 100%;
-   click-through, default off;
-   move step, default 20 px logical;
-   resize step, default 40 px logical.

Harness saves position, size, monitor, and opacity on every change.

Default shortcuts. Every one is configurable.

| Action | Default | Registered |
|---|---|---|
| Show or hide the overlay | `Ctrl+Space` | always |
| Send unsent transcript | `Ctrl+Shift+Enter` | always |
| Send screenshot as image | `Ctrl+Shift+O` | always |
| Send screenshot as OCR text | `Ctrl+Shift+X` | always |
| Switch to the next ChatGPT account | `Ctrl+Alt+A` | always |
| Move the overlay | `Ctrl+Alt+Arrow` | overlay visible |
| Resize the overlay | `Ctrl+Alt+Shift+Arrow` | overlay visible |
| Lower or raise opacity | `Ctrl+Alt+[` and `Ctrl+Alt+]` | overlay visible |
| Toggle click-through | `Ctrl+Alt+I` | overlay visible, and always while click-through is on |
| Reset position, size, and opacity | `Ctrl+Alt+0` | overlay visible |

How to build it:

-   `tauri-plugin-global-shortcut` calls Win32 `RegisterHotKey`. A
    registered combination stops reaching every other app. Register the
    "always" rows at startup. Register the "overlay visible" rows when
    the overlay shows and unregister them when it hides, so other apps
    keep those keys while the overlay is hidden.
-   If registration fails, another app already owns the combination.
    The shortcut editor shows "Taken by another app" and keeps the old
    binding.
-   Shortcuts that live inside one app never fail registration, and
    Harness takes them over. The shortcut editor warns about the known
    collisions. `Ctrl+Shift+O` opens Chrome's bookmark manager and VS
    Code's Go to Symbol. `Ctrl+Shift+X` opens VS Code's Extensions view.
    `Ctrl+Alt+Arrow` rotates the display on some Intel graphics drivers.
-   To move or resize the overlay, call `set_position` or `set_size` on
    the `WebviewWindow` with logical units. Then clamp the window to
    `current_monitor()` work area, the same rule as section 4.3.
    Debounce the settings write by 500 ms.
-   For opacity, set `transparent: true` and `decorations: false` on
    the overlay window in `tauri.conf.json`. Settings drive two CSS
    variables, `--overlay-bg-alpha` and `--overlay-text-alpha`. No Win32
    layered-window call is needed, and text stays sharp because the
    background alpha changes on its own.
-   For click-through, call `set_ignore_cursor_events(true)`. Mouse
    input then goes to the window below, and shortcuts still work. Show
    a small "Click-through" badge so the user knows why clicks pass
    through. Only the shortcut or the tray can turn click-through off.
-   `Ctrl+Space` shows the overlay with focus in the input. The
    transcript and screenshot shortcuts show it without taking focus
    from the meeting app or editor. To do that, get the HWND from
    `window.hwnd()` and call `ShowWindow(hwnd, SW_SHOWNOACTIVATE)`
    through the `windows` crate.

Shipped examples: Electron overlay apps in this category, such as the
open-source Interview Coder and its successor Cluely, use
`setAlwaysOnTop`, `setIgnoreMouseEvents`, `setContentProtection`, and
`setSkipTaskbar`, with arrow-key shortcuts to move the window. Tauri 2
has a direct equivalent for each call.

## 38.2 Hide from taskbar, Alt+Tab, and screen capture

Privacy settings in the Control center:

| Setting | Default |
|---|---|
| Exclude Harness windows from screen capture and screen share | on |
| Hide from the taskbar | on |
| Hide the overlay from Alt+Tab | on |
| Suppress Windows notifications while a meeting is active | on |
| Hide the tray icon | off |

How to build it:

-   For capture exclusion, set `contentProtected: true` in the window
    config, or call `window.set_content_protected(true)` at runtime. On
    Windows this calls
    `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)`. Electron's
    `setContentProtection(true)` makes the same call. Apply it to the
    overlay and the management window.
-   `WDA_EXCLUDEFROMCAPTURE` needs Windows 10 2004 (build 19041) or
    later. Older builds support only `WDA_MONITOR`, which shows a black
    rectangle in the share instead of hiding the window. On those
    builds, the Privacy page says so.
-   For the taskbar, set `skipTaskbar: true` in the window config.
-   Windows lists every visible top-level window in Alt+Tab unless the
    window has the `WS_EX_TOOLWINDOW` extended style. Add it with
    `SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_TOOLWINDOW)`
    (`windows` crate, `Win32_UI_WindowsAndMessaging` feature).
-   Windows notifications, the tray menu, and native file dialogs are
    separate windows that Harness cannot protect. Harness raises no
    notifications during a meeting. The tray menu appears only when the
    user clicks it.
-   If the user hides the tray icon, they can still reach Harness from
    the Start menu. A second launch shows the existing instance
    (section 18.4).

The Privacy page states what this does not hide:

-   a camera or phone pointed at the screen;
-   hardware capture devices;
-   the Harness process in Task Manager;
-   the tray icon, unless the user hides it.

Detecting or defeating proctoring, monitoring, or anti-cheat software
stays out of scope (section 1.5). Harness sets the documented display
affinity flag and the window styles above, and nothing more.

## 38.3 More than one ChatGPT account

-   The Accounts page lists each signed-in account with its label,
    email, status, last limit hit, and reset time.
-   **Add account** runs the Sign in with ChatGPT flow again. Each
    account's tokens go into Credential Manager under the target
    `Harness/chatgpt/<account_id>`. The `keyring` crate covers this.
-   One account is active at a time. The pre-meeting settings panel
    picks it, and the meeting snapshots it. `Ctrl+Alt+A` switches to
    the next account, and the overlay shows the new account's label for
    two seconds.
-   When the active account hits a usage limit, Harness keeps the
    unsent message, shows "Account <label> limited until <time>", and
    offers three single-key choices: next account, next fallback
    provider, or wait.
-   Harness never rotates ChatGPT accounts automatically to get past a
    limit. OpenAI's terms of use forbid circumventing rate limits, and
    breaking them can cost the accounts. Automatic failover goes only to
    API-key providers (38.4).
-   Check the Sign in with ChatGPT docs to see whether one
    `ext_agent_host_id` can register more than one user. If it cannot,
    generate one host ID per account.

### `accounts`

``` text
id TEXT PRIMARY KEY
provider TEXT NOT NULL          -- chatgpt | deepseek | gemini
label TEXT NOT NULL
email TEXT
credential_ref TEXT NOT NULL    -- Credential Manager target name, never the secret
is_active INTEGER NOT NULL
status TEXT NOT NULL            -- ok | limited | signed_out | error
limited_until INTEGER
created_at INTEGER NOT NULL
last_used_at INTEGER
```

## 38.4 Fallback providers: DeepSeek and Gemini

-   The Providers page takes an API key for DeepSeek, Gemini, or both.
    Keys go into Credential Manager under `Harness/apikey/<provider>`.
    They never go into SQLite or React state (section 14.1). React
    receives only "key set" and the last four characters.
-   Both services offer an OpenAI-compatible Chat Completions endpoint.
    One `OpenAiCompatProvider { base_url, model, credential_ref }`
    behind the existing `AiProvider` trait covers both:
    -   DeepSeek: `https://api.deepseek.com`;
    -   Gemini: `https://generativelanguage.googleapis.com/v1beta/openai/`.
-   Fetch each provider's model list from `GET /models` and let the user
    pick. Do not hardcode model names.
-   The fallback order is a list the user can reorder. The default is
    the active ChatGPT account, then Gemini, then DeepSeek. Any provider
    can be first. If Sign in with ChatGPT does not work for a private
    app, an API-key provider becomes primary and the product still
    works.
-   Harness moves to the next provider only when a request fails before
    the first token: a usage limit, HTTP 429, HTTP 5xx, an auth error,
    or no first token within 10 seconds. After tokens have streamed,
    Harness never switches silently. It keeps the partial answer and
    offers "Retry with <next provider>".
-   Screenshot images go only to providers that accept image input.
    Gemini does. Check DeepSeek's current API docs. If DeepSeek is
    still text-only, Harness sends it the OCR text instead.
-   Each answer shows a provider chip, such as `ChatGPT · work` or
    `Gemini`.
-   The context compiler stays provider-neutral. Each provider maps the
    compiled message list to its wire format (Responses API for
    ChatGPT, Chat Completions for the others) and reports its context
    limit to the budgeter (section 13.2).
-   A fallback sends the same compiled context to another company with
    its own retention policy. DeepSeek's privacy policy says it stores
    data on servers in China. Context profiles (38.6) can disable any
    provider.

Shipped examples: Raycast AI and Cursor let the user pick a model and
bring their own keys. OpenRouter and LiteLLM retry the next provider in
an ordered list on 429 and 5xx errors. Harness does the same thing in
process, with no proxy server.

## 38.5 Usage management

Harness writes one `usage_events` row per model request. The Responses
API and Chat Completions both return a `usage` object with input and
output token counts, plus cached input tokens where the provider
reports them.

### `usage_events`

``` text
id TEXT PRIMARY KEY
account_id TEXT NOT NULL
provider TEXT NOT NULL
model TEXT NOT NULL
meeting_id TEXT
kind TEXT NOT NULL              -- question | transcript | screenshot_image | screenshot_ocr | rolling_state
input_tokens INTEGER
output_tokens INTEGER
cached_tokens INTEGER
image_count INTEGER NOT NULL
first_token_ms INTEGER
total_ms INTEGER
status TEXT NOT NULL            -- ok | limited | error | fell_back
created_at INTEGER NOT NULL
```

The Usage page shows, per account and per provider, today and the last
seven days: request count, tokens, and limit hits. For API-key
providers it also shows estimated cost from a price table the user can
edit, because prices change after release.

Each API key can have an optional daily token or cost cap. At the cap,
that provider leaves the fallback chain until local midnight.

The ChatGPT plan path may not expose remaining quota. Show what Harness
knows: local request counts, the last limit hit, and the reset time
from the error response when it has one. Do not draw quota bars Harness
cannot measure (the design rule against fabricated live data).

## 38.6 Context center

The Context page sits on top of the memory system (section 11). It
holds context profiles. A profile is a named set such as "Backend
interview, Acme" with:

-   a custom instruction the user writes;
-   pinned items, such as a resume, a job description, project notes,
    or any memory or document, sent on every request within a pinned
    token budget;
-   a retrieval scope: which projects and documents retrieval may
    search;
-   the providers allowed for this profile.

The pre-meeting settings panel picks a profile, and the meeting
snapshots it. The Context page shows the pinned items' token count next
to the budget. If the pinned items exceed the budget, the page names
the items the compiler will drop.

Every answer has a **What was sent** view. It shows the compiled
request: each context section with its provenance label (section 13.3),
token counts, provider, and account. Harness stores the compiled
request with the answer, and the transcript deletion controls apply to
it too.

ChatGPT Projects and Claude Projects use the same shape: project
instructions, pinned files, and retrieval once the files outgrow the
context window.

### `context_profiles`

``` text
id TEXT PRIMARY KEY
name TEXT NOT NULL
instruction TEXT
pinned_budget_tokens INTEGER NOT NULL
allowed_providers TEXT NOT NULL     -- JSON array
created_at INTEGER NOT NULL
updated_at INTEGER NOT NULL
```

### `context_profile_items`

``` text
profile_id TEXT NOT NULL
item_type TEXT NOT NULL         -- memory | document | project | text
item_id TEXT
text TEXT
ordinal INTEGER NOT NULL
pinned INTEGER NOT NULL
```

## 38.7 Screenshot to model

Two global shortcuts send the screen to the model:

-   `Ctrl+Shift+O` captures the screen and sends the image;
-   `Ctrl+Shift+X` captures the screen, runs local OCR, and sends the
    text.

Both send right away as a new user message in the current
conversation. If the input box holds typed text, that text is the
instruction. Otherwise Harness uses the active response mode, or the
default instruction "Answer or solve what is on screen", which the user
can edit in settings. Both work with or without an active meeting.

### Capture

-   Capture the monitor under the mouse cursor with the `xcap` crate
    (`Monitor::from_point`, then `capture_image`). Capture runs in Rust,
    and pixels never pass through React.
-   Capture exclusion (38.2) keeps the overlay out of its own
    screenshot, so Harness does not hide it first. If the user turned
    exclusion off, Harness hides the overlay, waits one frame (about
    50 ms), captures, and shows the overlay again.

### Image path (`Ctrl+Shift+O`)

-   Downscale so the long edge is at most 2048 px, then encode as PNG.
    PNG keeps small text sharp.
-   For the Responses API, send an `input_image` part with a base64 data
    URL. For Gemini's OpenAI-compatible endpoint, send an `image_url`
    part with a data URL.
-   If the active provider has no image input, use the next
    image-capable provider in the chain. If none exists, run the OCR
    path and tell the user.

### OCR path (`Ctrl+Shift+X`)

-   Use the OCR engine built into Windows, `Windows.Media.Ocr`, through
    the `windows` crate (`Media_Ocr` and `Graphics_Imaging` features).
    It runs locally, costs nothing, and needs no bundled model. Call
    `OcrEngine::TryCreateFromUserProfileLanguages()`, build a BGRA8
    `SoftwareBitmap` from the capture, call `RecognizeAsync`, and read
    `Lines` and their `Words` with bounding boxes.
-   Windows OCR drops indentation. For code, rebuild it from each
    line's left x-coordinate divided by the average character width.
    Python stays readable that way. Wrap detected code in a fenced
    block.
-   Check `OcrEngine::MaxImageDimension`. If a capture exceeds it,
    downscale or tile.
-   Show the OCR text in the message so the user sees exactly what
    Harness sent.
-   If Windows OCR fails on real screens in a measured test, swap in
    PaddleOCR through ONNX Runtime behind the same function. Do not
    bundle it before then.

### Storage

Harness stores OCR text with the message. It writes screenshot images
to `%LOCALAPPDATA%\Harness\cache\screens\` and deletes them when the
meeting ends, unless the user turns on "Keep screenshots". Logs never
contain image bytes or OCR text (section 21).

Shipped examples: the open-source Interview Coder takes a screenshot on
a shortcut and sends it to a vision model, the same flow as
`Ctrl+Shift+O`. Microsoft PowerToys Text Extractor uses
`Windows.Media.Ocr` for local OCR, the same engine as `Ctrl+Shift+X`.

## 38.8 Markdown and code in answers

-   The system prompt tells every provider to answer in GitHub-flavored
    Markdown and to tag every fenced code block with its language.
-   Render answers with `streamdown`, Vercel's replacement for
    `react-markdown` built for streaming model output. Plain
    `react-markdown` re-parses the whole answer on every token and shows
    broken output while a code fence or bold span is still open.
    `streamdown` handles unterminated blocks and memoizes finished ones.
    Check its bundle size against the overlay show-time target
    (section 19.3). The fallback is `react-markdown` with `remark-gfm`,
    `rehype-highlight`, and per-block memoization.
-   Highlight code with Shiki, using a theme built from the dark design
    tokens.
-   Each code block has a language label and a copy button, and scrolls
    horizontally instead of wrapping.
-   GFM gives tables, task lists, and strikethrough.
-   Model output is untrusted (section 22.1). Disable raw HTML. Open
    links in the default browser through the Tauri opener plugin, and
    never navigate the webview. The CSP blocks remote images, so a
    Markdown image renders as a link.

## 38.9 Control center

The management window (section 18.1) becomes the Control center, with
these pages: Accounts, Providers, Usage, Context, Shortcuts, Overlay,
Privacy, Audio, Memory, and Meetings. The tray opens it.

Settings changed during a meeting apply to the next meeting, following
the snapshot rule. Overlay, Shortcuts, and Privacy settings are the
exception and apply at once.

The Shortcuts page has a press-to-record field per action, shows
registration conflicts, and has a reset-to-default button per row.

## 38.10 Shipped products that use these techniques

| Feature | Shipped example | Their mechanism | Harness |
|---|---|---|---|
| Overlay hidden from share and taskbar | Interview Coder, Cluely (Electron) | `setContentProtection`, `setSkipTaskbar` | `set_content_protected`, `skipTaskbar`, `WS_EX_TOOLWINDOW` |
| Click-through overlay | same | `setIgnoreMouseEvents` | `set_ignore_cursor_events` |
| Meeting audio without a bot | Granola | captures device audio on the laptop | WASAPI loopback and mic, with local STT |
| Screenshot to model | Interview Coder | shortcut screenshot to a vision model | 38.7 image path |
| Local OCR | PowerToys Text Extractor | `Windows.Media.Ocr` | 38.7 OCR path |
| Own keys and model choice | Raycast AI, Cursor | provider picker, user keys | 38.4 |
| Ordered fallback | OpenRouter, LiteLLM | retry next provider on 429 and 5xx | in-process fallback chain |
| Project context | ChatGPT Projects, Claude Projects | instructions, pinned files, retrieval | context profiles |
| Streaming Markdown | Vercel streamdown | repairs unterminated blocks while streaming | same library |
