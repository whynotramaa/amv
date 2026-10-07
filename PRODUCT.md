# Harness product

<!-- impeccable:product-schema 1 -->

## Platform

web

Harness uses React in a Tauri WebView, with Windows-native desktop behavior. Windows 11 x64 is the primary target; this is not a hosted website.

## Stack

Tauri 2, Rust, React, TypeScript, Vite, SQLite. These choices come from PLAN.md. npm is the implementation package manager because it is already installed; commit its lockfile.

## Users

Private use for permission-based sales calls in Zoom, Google Meet, Teams, or other meeting applications. The assistant stays beside the user's work in a compact floating window.

## Product purpose

Transcribe microphone and system audio locally and separately. Produce meeting notes and answer sales questions and statistics with source context from the current meeting, prior meetings, and retained project information. Preserve transcripts incrementally and recover after interruption.

## Operating context

Before a meeting, the user chooses automatic sending or hotkey-triggered sending of new finalized system speech. Response behavior can be suggested answers, summaries, or a custom instruction. Each new transcript message joins a local conversation history. The microphone remains separate context and is not automatically sent as a new message.

## Capabilities and constraints

- Private use only; no public distribution or commercial launch is required.
- Windows laptops with 8 GB RAM and no dedicated GPU must be supported. Minimize resident memory and sustained CPU usage.
- No remote audio transcription, uploaded PCM, mandatory Harness backend, or runtime Python.
- Sign in with ChatGPT is preferred. Actual account/client eligibility must be proven. DeepSeek and Gemini API-key providers are configurable fallbacks with editable base URLs and explicit context-sharing permission.
- The installation and local transcription must work offline. Sign-in can happen later.
- PLAN.md remains the implementation contract; PROGRESS.md records completed work and verification.

## Brand commitments

Harness. The user supplied a binding design brief: premium, calm, restrained, dark-first and desktop-native, with Linear-level clarity. Use Poppins UI text and Geist Mono code. No serif fonts, Sparkles icon, CSS shadows, or native window shadows; use thin subtle borders and tonal separation. Create an original interface; do not copy another product. Shared tokens and reusable primitives must precede UI implementation.

## Evidence on hand

PLAN.md and the decisions in this conversation. Implementation ideas and shipped-product references are now included in revised PLAN.md. No actual visual reference images have been attached. The foundation is implemented; native Windows runtime acceptance remains pending. Never invent a connected account, live capture, participants, transcripts, or benchmarks to fill the UI.

## Product principles

- Local state is authoritative.
- Reliable audio and transcript persistence come before enrichment.
- Settings make remote sending explicit before a meeting.
- Show useful state with minimal chrome.

## Accessibility and inclusion

Readable text contrast, visible keyboard focus, semantic controls, clear capture status, keyboard operation, reduced-motion support, and correct Windows DPI behavior are required.
