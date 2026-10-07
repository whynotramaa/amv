---
name: Harness
description: A clear desktop utility for permission-based sales meetings.
colors:
  background: "#111316"
  surface: "#191c20"
  surface-elevated: "#22262b"
  surface-hover: "#2a3036"
  surface-active: "#303842"
  text-primary: "#e9edf2"
  text-secondary: "#b6bec9"
  text-tertiary: "#9aa5b3"
  text-muted: "#8e99a8"
  border-subtle: "#a7b3c21a"
  border-default: "#454e5a"
  accent: "#b2c9e5"
  accent-muted: "#2c3b4d"
  danger: "#f0aaa6"
  success: "#b3d1be"
typography:
  body:
    fontFamily: "Poppins, Segoe UI, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
  title:
    fontFamily: "Poppins, Segoe UI, sans-serif"
    fontSize: "26px"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.025em"
  section-title:
    fontFamily: "Poppins, Segoe UI, sans-serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: 1.5
  label:
    fontFamily: "Poppins, Segoe UI, sans-serif"
    fontSize: "14px"
    fontWeight: 500
    lineHeight: 1.5
  meta:
    fontFamily: "Poppins, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
  code:
    fontFamily: "Geist Mono, ui-monospace, monospace"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "22px"
spacing:
  1: "4px"
  2: "8px"
  3: "12px"
  4: "16px"
  5: "24px"
  6: "32px"
  7: "40px"
  8: "48px"
components:
  button-primary:
    backgroundColor: "{colors.text-primary}"
    textColor: "{colors.background}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
    height: "40px"
  button-default:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.text-primary}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
    height: "40px"
  button-quiet:
    backgroundColor: "transparent"
    textColor: "{colors.text-primary}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
    height: "40px"
  input:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.text-primary}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
    height: "40px"
---

# Design System: Harness

## Overview

Harness is a private, permission-based sales assistant for meeting notes and answers grounded in supplied facts. Clarity comes first. The interface is an original compact desktop utility with the restraint of professional tools such as Linear; references guide principles, not copied layouts or branding.

The current foundation supports saved preferences, tray access and shortcuts. Connections extends the same utility language to explicit ChatGPT account selection, browser-based sign-in, and permissioned API fallback settings. The meeting surface adds a permissioned local setup, explicit capture states, retained local meeting context, and finalized transcript provenance. Capture and dispatch are integrated in source. Live provider acceptance and Windows-native behavior remain explicit until verified.

Normative pins remain: Poppins and Geist Mono, no shadows, custom icons, no Sparkles, compact desktop. These describe the existing system and are not optional per-surface styling suggestions.

Current UI verification passes with no shadows, no runtime errors, and no overflow at the 420px Windows minimum width.

## Colors

The source of truth is `src/styles/tokens.css`. Use near-black neutral layers with readable text. Accent color is limited to focus, selection and meaningful state. Use a thin `border-subtle` boundary or a slight luminance difference to separate surfaces. Errors and successful saves use semantic colors with text and status roles.

## Typography

Poppins is the UI font at weights 400, 500 and 600. Geist Mono is the code and keyboard-hint font. Both are self-hosted WOFF2 assets; no font CDN is needed at runtime. No serif fonts.

Titles: 26px / 600 / 1.25. Section titles: 16px / 600. Body and labels: 14px, weights 400 and 500. Help and metadata: 12px. Small metadata: 11px. Body line height: 1.5. Titles use -0.025em tracking. Code blocks scroll horizontally and preserve formatting.

The shipped Latin font subsets cover the current English interface. System sans/mono fallbacks handle other scripts. Font source, versions, hashes and OFL licenses live in `src/assets/fonts/`.

## Layout

The assistant uses a single aligned floating surface, 520px wide by default, with 24px padding and conceptual gaps. Related controls use the 4, 8, 12 and 16px scale. Do not introduce dashboard grids, excessive cards, decorative illustrations or giant navigation.

Below 460px, padding becomes 16px, shortcut fields stack, and secondary action text moves below the primary action. The Windows minimum width is 420px. The Control center may use grouped settings navigation; the meeting popover stays compact.

Meeting setup keeps one aligned form: title first, saved preferences summarized below it, then an unchecked per-meeting permission checkbox and the primary start action. The assistant keeps the meeting title and current state above the transcript; after stop, the title and locally saved transcript remain the identity of the completed meeting. Elapsed time is a compact Geist Mono value and updates only while the document is visible.

## Elevation & Depth

No shadows, glows or decorative blur. Native window shadow is disabled. Thin low-opacity borders and slight neutral surface differences communicate layering. Keyboard focus uses a visible 2px accent outline with a 2px offset, rather than a box shadow.

## Shapes

Primary floating surface: 22px radius. Larger secondary surfaces: 16px. Controls: 12px. Compact icon controls: 8px. Keyboard hints: 4px. Ordinary controls are 40px high; compact icon controls are 32px. Borders are 1px. Avoid pill-shaped primary surfaces.

## Components

`src/components/primitives.tsx` owns shared buttons and original geometric stroke icons: 18px display size, 24px view box and consistent 1.5px stroke. Do not use Sparkles. Icons support actions and status rather than decoration.

Buttons use quiet tonal changes, coherent radii and visible hover, focus, disabled and loading states. Primary buttons use charcoal text on the light primary-text surface. Inputs use an elevated neutral surface and a structural border. Radio choices use a subtle selected surface and native semantics. Details progressively disclose secondary settings.

### Meeting setup

- **Title:** Required, trimmed, and limited to 200 characters; use the existing `input` treatment and show the count as supporting metadata.
- **Permission:** The checkbox starts unchecked for every new meeting. Starting is blocked until the user confirms permission to capture audio for that meeting; this is not a saved global preference.
- **Saved preferences:** Show the persisted send mode and response mode as read-only summary rows, with a quiet action to change them in the existing Meeting preferences view. Saving returns to the assistant and confirms locally saved preferences.
- **Preview boundary:** In the browser preview, native start and stop actions remain disabled and the copy says that no audio is captured. `meeting-active-fixture.png` is a test fixture showing root IPC mock proof only; it is not evidence of actual recording.

### Meeting states

Use the existing status dot, label, primary action, and notice language as one state group. `starting` shows “Starting local capture” and disables conflicting actions; `active` shows “Meeting in progress”, a success dot, and visibility-aware `MM:SS` elapsed time; `stopping` shows “Stopping local capture” and keeps stop disabled while finalization completes; `error` uses the danger role, preserves the error copy, and offers retry; an idle meeting with an id is presented as “Meeting saved locally”. The title is retained after stop, and finalized local transcript rows remain visible.

### Finalized transcript

Rows are factual, compact, and provenance-first: source (`MIC` or `SYSTEM`), start time in `MM:SS`, then finalized text. The list uses a thin top border, tonal spacing, Geist Mono for source/time metadata, and a bounded scroll region. An empty active list says “Waiting for finalized speech.” Do not represent partial or speculative speech as finalized.

### Remote composer

The composer is enabled only in the native app with a retained meeting and configured inference selection. Enter asks a question; Shift+Enter inserts a line. MIC context is a per-question opt-in and never enters automatic requests. Send new speech also retries a saved no-output request. Preparing/streaming states disable competing submissions and expose Cancel. Actual runtime connectivity is established by the request, not the availability label. Browser preview keeps native actions disabled.

### Connections

- **Account selection:** Show each connected ChatGPT account as an explicit row with its display name/email and a quiet `Use account` or `Selected` action. Do not imply automatic account rotation.
- **Sign-in:** Use the native browser sign-in action and expose the waiting/cancel state. Signed-out rows offer Sign in again; signed-in rows disclose reauthorization and sign-out through native Account options. The preview does not prove native OAuth acceptance.
- **Account identity and models:** Show a short stable account identifier and signed-in status alongside the name/email. Discover models only on explicit request for the selected account, retain the saved model, and reject late catalog replies after account changes.
- **Sign-out feedback:** Distinguish local credential removal from confirmed remote revocation. When remote confirmation fails, direct the user to disconnect Harness in ChatGPT Settings. Preserve unsaved provider settings during account operations.
- **API fallback:** Keep Gemini and DeepSeek as explicit, ordered provider rows. Each row contains an editable base URL, URL-bound API key field, discovered-model selection, and a native checkbox that gives permission to receive meeting text as fallback.
- **Provider order:** Make ordering visible and reversible with the existing quiet action; save the order with the shared primary action.
- **Security copy:** Explain that keys stay in the Windows credential vault and that a saved key only works with its original base URL. Never render key values or imply a configured account/provider that is not present.

State transitions use 180ms with `cubic-bezier(.2, .8, .2, 1)` and respect reduced motion. There are no automatic decorative entrances.

Original app icon source: `src-tauri/icons/harness.svg`, rasterized by `npm run tauri icon`. Font assets are licensed source assets, not generated imagery. No screenshots, mock data or accounts imply working integrations.

## Do's and Don'ts

- Keep the current meeting, clear state and next action dominant.
- Use shared tokens; extend them deliberately before adding component values.
- Make capture, permissions, provider choice, context sharing and errors understandable.
- Make account selection, browser sign-in, provider order, model discovery and fallback consent understandable.
- Keep meeting title, per-meeting consent, saved preferences, capture state, transcript source/time, and local-save status understandable.
- Preserve keyboard operation, contrast, visible focus and reduced motion.
- Keep model output untrusted and render no raw HTML.
- Do not add shadows, Sparkles, serif fonts, neon effects, excessive cards or copied product layouts.
- Do not fabricate capture, accounts, provider quotas, statistics or performance measurements.
- Do not claim live capture is proven: native Windows acceptance is pending.
- Do not treat `meeting-active-fixture.png` as a recording; it is a root IPC mock test fixture only.
- Do not mark OAuth, credential-vault, model-discovery or Windows-native acceptance complete from a preview screenshot alone.

The meeting setup also includes a native microphone select using the existing input primitive. Device discovery occurs only when preparing a meeting and does not start capture. The default follows Windows; other discovered microphone IDs are passed explicitly. System audio follows the playback device and remains separately tagged. Browser mocks verify selection wiring only, not device behavior.

## Saved meetings

The custom history icon opens a local list while live capture keeps running. Completed and interrupted meetings appear newest first, 20 per page. Selecting a meeting reveals source-tagged transcript lines and explicit local search. Each transcript page holds at most 50 rows and 64 KiB of text; next pages replace the previous content. Literal search accepts up to 16 words and 512 UTF-8 bytes, with up to 100 matches and the same text cap. Search results remain distinct from the editable query. Visible focus, original shared icons, neutral rows and thin separators use the existing tokens. No network or model request runs from this surface. Browser fixture screenshots verify UI behavior; Windows rendering and real hardware remain unverified.

## Streaming responses

The answer surface uses the existing spacing, radii, tonal surfaces and thin borders. It reports preparation, streaming, completion, partial output, failure and cancellation. Request attempts distinguish a legitimate retry from a stale event. Context omissions and actual returned usage remain quiet supporting text. Partial output remains visible and is never automatically replayed.

The lazy answer renderer uses React Markdown with remark-gfm and memoized code tokenization. Shiki uses its JavaScript engine, seven separately imported language grammars, and a theme read from the existing CSS tokens. Unknown languages and blocks over 16 KiB stay plain text. Code retains Geist Mono, a language label, copy feedback and horizontal scrolling. Raw HTML is suppressed; model images make no remote request. Web links use the validated native browser command. All generated token content renders as React text.

A response collapses the finalized transcript, which remains keyboard-expandable using the original chevron icon. Next-meeting preferences are disclosed separately from the active meeting. Saved history offers Open in assistant without restarting capture or automatic permission. Custom instructions enforce both character and UTF-8 byte bounds.

The response fixture checks 520×600 and 420×360 layouts, consent, attempt ordering, GFM, code/copy, unsafe HTML/links/images and saved restore. Screenshots are synthetic IPC proofs. Native rendering, eligible OAuth, real fallback, voice concurrency, memory and clipboard acceptance remain pending.
