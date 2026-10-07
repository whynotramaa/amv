# Literal PLAN bullet inventory

606 bullets outside code fences. This includes informational reference bullets; those are not independent runtime obligations. An unchecked row is not a bug claim. Mark a row complete only with evidence. REQUIREMENTS.md supplies the grouped assessment.


## Agreed implementation decisions

- [ ] `B-239da9702f41` · PLAN lines 19–19: **Use and distribution:** private personal use only. Public code signing and public release infrastructure are not V0 blockers. Do not assume private-app ChatGPT-plan eligibility; prove the official sign-in/inference path before claiming it works. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5d6a5f58f0c3` · PLAN lines 20–20: **Minimum hardware:** Windows x64 with 8 GB RAM, CPU-only operation, no dedicated GPU required. Keep the model unloaded when idle, bound PCM queues, and avoid loading embedding and STT models concurrently without a measured budget. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5bbc66f85a91` · PLAN lines 21–21: **Sending mode:** before each meeting, the settings panel offers `On hotkey` and `Automatically`. Default to `On hotkey`. Save the chosen settings and snapshot them at meeting start. Editing those settings takes effect for the next meeting. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-712b86a048d3` · PLAN lines 22–22: **New messages:** only new finalized system-audio speech becomes a transcript user message. Never repeatedly append the whole meeting as a new message. Automatic mode batches finalized speech while one response is running; hotkey mode sends all unsent finalized system speech. Preserve order, segment provenance, and retry state. Microphone transcripts stay separate and are available as explicitly compiled context. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-979a915ea73c` · PLAN lines 23–23: **History:** Harness owns local message history. A new transcript message is appended to that conversation. The HTTP inference request must include prior context because the intended API path does not provide persistent server conversation storage. Bound older context to model limits while keeping the full transcript/history locally; disclose incomplete context rather than silently losing the current message. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-12ae49257117` · PLAN lines 24–24: **Response mode:** offer `Suggested answers`, `Summary`, and `Custom instruction`. All modes are configurable before the meeting; custom mode requires a non-empty instruction. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-259203062490` · PLAN lines 25–25: **Hotkeys:** the popover shortcut and the send-transcript shortcut are distinct and configurable. Opening or hiding the popover must not accidentally send speech. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a6ddcd3f064f` · PLAN lines 26–26: **Offline onboarding:** local setup may finish before ChatGPT sign-in. Local capture, transcription, persistence, and search remain available without account/network access. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c32e9b99f013` · PLAN lines 27–27: **Remote policy:** automatic mode authorizes sending new finalized system text during that meeting. Hotkey mode makes no background summarization requests. No raw audio is sent in either mode. Inference errors and usage limits preserve unsent speech and local transcripts. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-426b28e70b90` · PLAN lines 28–28: **Purpose and consent:** assist sales calls with meeting notes, grounded statistics, and clear answers. Capture and remote sending require explicit user control and meeting permissions. Do not present this product as an interview-cheating or covert-monitoring tool. Treat transcripts and documents as untrusted context, and give statistical answers provenance rather than invented certainty. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3bf0b39de6bb` · PLAN lines 29–29: **Design:** clarity first, with Linear-level restraint and an original desktop utility composition. Poppins for UI text, Geist Mono for code, no serif fonts, no Sparkles icon. Use original consistent icons. No CSS or native window shadows: separate surfaces with thin low-opacity borders and subtle luminance changes. Establish shared spacing, typography, surfaces, radii, borders, icon sizes, control heights, and states before components. The latest direction overrides the earlier system-font and shadow examples. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-50d9b2a18a54` · PLAN lines 30–30: **Provider configuration:** offer editable base URLs and securely entered API keys, with DeepSeek and Gemini preconfigured. Model choices come from provider discovery. Confirm allowed providers and remote-context sharing before automatic fallback; never rotate ChatGPT accounts automatically to bypass a limit. ChatGPT sign-in remains the preferred path, uses official dynamic registration as confirmed by the user, and must be verified with an actual eligible account/client configuration. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dafdc62ac72c` · PLAN lines 31–31: **Completion evidence:** a working Windows installer and end-to-end ChatGPT sign-in, smooth separate audio transcription, and fallback inference are required. Track every normative plan bullet with evidence and pending checks. Run multiple verification passes for correctness, integration, UI/accessibility, and Windows runtime. Measure memory on the minimum 8 GB machine; bounded buffers and unload-on-idle are requirements, not proof of zero growth. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f48df6ac7007` · PLAN lines 32–32: **Overlay privacy:** by default, Harness windows are excluded from screen capture and screen share, and hidden from the taskbar and Alt+Tab. Harness uses documented Windows window flags for this and nothing else. Section 38.2 sets the limits. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-15112a1fb7f6` · PLAN lines 33–33: **Overlay control:** opacity, click-through, position, and size all have settings and keyboard shortcuts. Section 38.1 has the full shortcut table. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-984f5c8ffa46` · PLAN lines 34–34: **Accounts and providers:** the user can sign in more than one ChatGPT account and add DeepSeek or Gemini API keys as fallbacks. Switching ChatGPT accounts is a user action. Automatic failover goes only to API-key providers. See sections 38.3 to 38.5. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a27dc738f4a2` · PLAN lines 35–35: **Screen input:** `Ctrl+Shift+O` sends a full screenshot as an image. `Ctrl+Shift+X` sends local OCR text of the screenshot. Harness captures the screen only when the user presses one of these shortcuts. See section 38.7. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-82c44750f7d8` · PLAN lines 36–36: **Answer formatting:** answers render as streaming GitHub-flavored Markdown with highlighted code blocks. See section 38.8. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a207c732b40c` · PLAN lines 37–37: **Milestones:** package the desktop foundation first; then prove simultaneous local STT and authorized streaming inference early, before expanding embeddings and reranking. The directory tree in section 16 is illustrative: create modules when they contain working behavior. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 1.1 Windows application

- [ ] `B-0c8e1139d4e9` · PLAN lines 90–90: Windows 11 x64 Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9447d4e1355d` · PLAN lines 91–92: Windows 10 22H2 x64 should be supported where the required Windows APIs and WebView2 runtime behave correctly. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b9a77b89f98` · PLAN lines 93–94: ARM64 is not a launch blocker. Keep architecture boundaries portable enough that an ARM64 build can be added later. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-be28b9fb6d5c` · PLAN lines 98–98: start normally from the Start menu; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3c468390cc0c` · PLAN lines 99–99: optionally launch at Windows login; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-574d7fe07cba` · PLAN lines 100–100: live in the system tray; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-95644ef1995a` · PLAN lines 101–101: remain usable without a full-size main window; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e39a4e6a215` · PLAN lines 102–102: expose a global shortcut; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4f0aaba375a3` · PLAN lines 103–103: show/hide its popover quickly; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-43a06c6d593d` · PLAN lines 104–104: capture system loopback audio; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e34fc0ee4bb` · PLAN lines 105–105: capture microphone audio; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-83e8cfd6d79c` · PLAN lines 106–106: operate correctly across multiple monitors; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2bada80d2e33` · PLAN lines 107–107: handle Windows DPI scaling; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-52b39e77a0dd` · PLAN lines 108–108: recover from audio-device changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ea44348503c3` · PLAN lines 109–109: recover from sleep/wake; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0282e764422b` · PLAN lines 110–110: shut down cleanly. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 1.2 Local-first

- [ ] `B-36237fcaf1cf` · PLAN lines 116–116: microphone PCM; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-adab5c7f3b46` · PLAN lines 117–117: system/loopback PCM; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-371c82edd0fd` · PLAN lines 118–118: VAD; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ab900cd03790` · PLAN lines 119–119: resampling; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-779b6fda1cf5` · PLAN lines 120–120: speech-to-text; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b35ae48bbc87` · PLAN lines 121–121: transcript database; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c2adb3a668c8` · PLAN lines 122–122: meeting summaries/state cache; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2b87e7b7d3b3` · PLAN lines 123–123: imported documents; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-198e6581f7ef` · PLAN lines 124–124: local search indexes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-256497a713dc` · PLAN lines 125–125: personal memories; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-da7c9cb9db45` · PLAN lines 126–126: user/project/entity records; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e0f8968069ae` · PLAN lines 127–127: retrieval; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1ed9c9800329` · PLAN lines 128–128: context selection; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-095e7350157c` · PLAN lines 129–129: settings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-642113107745` · PLAN lines 130–130: logs; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a52f4af6cede` · PLAN lines 131–131: authentication secrets/tokens in OS-backed secure storage. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 1.3 Clean-machine installation

- [ ] `B-618f77c74707` · PLAN lines 154–154: Rust; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-acff764ad96c` · PLAN lines 155–155: Cargo; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fdd7331228b7` · PLAN lines 156–156: Node.js; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0ac5f138afd8` · PLAN lines 157–157: npm/pnpm; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f23ff144e4eb` · PLAN lines 158–158: Python; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-40a129f2366b` · PLAN lines 159–159: Git; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b928f3e8e28` · PLAN lines 160–160: CMake; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9ebb8f0696f1` · PLAN lines 161–161: Visual Studio Build Tools; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9d8a3c7c5631` · PLAN lines 162–162: Whisper; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7f99a7c1cb1d` · PLAN lines 163–163: FFmpeg; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1c38b66397aa` · PLAN lines 164–164: SQLite; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-756b6354090d` · PLAN lines 165–165: a local database server; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5bd7bcd3ae12` · PLAN lines 166–166: model-serving software; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-53f0eaee0946` · PLAN lines 167–167: Docker. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-758cc2c253fd` · PLAN lines 176–176: NSIS setup executable generated through Tauri; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-67c3a125fcf2` · PLAN lines 177–177: offline WebView2 installer included; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8629fca5373b` · PLAN lines 178–178: local STT runtime included; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cb9174fcbc72` · PLAN lines 179–179: default STT model included if licensing permits redistribution; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-634946b745b6` · PLAN lines 180–180: database migrations included; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-eef03f0327c2` · PLAN lines 181–181: app assets included. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 1.4 No hidden server dependency

- [ ] `B-42583037516c` · PLAN lines 194–194: hosted PostgreSQL; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-719c6585df22` · PLAN lines 195–195: Redis; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-870c3ef58a47` · PLAN lines 196–196: remote vector database; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e58c0936bbaf` · PLAN lines 197–197: remote transcription service; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-35a2c781fc2d` · PLAN lines 198–198: hosted memory service; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e5f977986cc8` · PLAN lines 199–199: a Harness account server; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8af316c3f9c6` · PLAN lines 200–200: a Docker daemon. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 1.5 Privacy and consent

- [ ] `B-dc75b93ceb21` · PLAN lines 213–214: exclude Harness windows from screen capture and screen share with the documented `WDA_EXCLUDEFROMCAPTURE` window flag; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-37fc47a69823` · PLAN lines 215–215: hide Harness windows from the taskbar and Alt+Tab. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e688d77ffabb` · PLAN lines 219–220: detecting, hiding from, or defeating proctoring, monitoring, or anti-cheat software; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bfda7823283d` · PLAN lines 221–222: secretly persisting audio against the user's configured retention behavior. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Screen 1 --- Welcome

- [ ] `B-607bb88c56db` · PLAN lines 288–288: Harness identity/wordmark; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-27c216e8ac03` · PLAN lines 289–289: one-sentence explanation; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dfc989849670` · PLAN lines 290–290: `Continue with ChatGPT`. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Screen 3 --- Audio

- [ ] `B-2e6df0a21f8c` · PLAN lines 318–318: default microphone; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8fbc2dfff840` · PLAN lines 319–319: default system output device. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Screen 4 --- Local AI setup

- [ ] `B-9babadd04384` · PLAN lines 329–329: STT runtime is present; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-633ea256a864` · PLAN lines 330–330: bundled model exists and checksum matches; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-276fecdaa189` · PLAN lines 331–331: database is writable; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3297f9595136` · PLAN lines 332–332: local search index can initialize. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 4.2 Global shortcut

- [ ] `B-c7805c2bdab5` · PLAN lines 398–398: if popover hidden → show; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6bbfb1885a88` · PLAN lines 399–399: if visible and interactive → hide; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c944ea408012` · PLAN lines 400–400: `Esc` → hide; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9b2f320e0bc9` · PLAN lines 401–401: shortcut should work while Zoom/Meet/Teams/browser/IDE has focus; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-125404817a40` · PLAN lines 402–404: this shortcut shows the overlay with keyboard focus in the input. The send-transcript and screenshot shortcuts show it without taking focus (section 38.1). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 4.3 Popover

- [ ] `B-65f00acf09bb` · PLAN lines 414–414: always-on-top while visible; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c9606bf2adfc` · PLAN lines 415–415: no traditional title bar; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-acd0ea4d862d` · PLAN lines 416–416: rounded visual surface; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ba095baff57e` · PLAN lines 417–417: draggable; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5d0a74b3b716` · PLAN lines 418–418: resizable within defined min/max bounds; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c6dc4afcc0e7` · PLAN lines 419–419: remembers last position; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7a6ffe94b14d` · PLAN lines 420–421: clamps itself into the visible work area if monitor topology changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a14f924d6330` · PLAN lines 422–422: per-monitor DPI aware; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-41ac164f54bc` · PLAN lines 423–423: multi-monitor safe; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0f8d1a7d32e3` · PLAN lines 424–424: fast show/hide; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b54c577eb76` · PLAN lines 425–425: keyboard-first; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-851fac5a5068` · PLAN lines 426–426: supports mouse interaction; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-26a09371d603` · PLAN lines 427–428: adjustable background and text opacity, click-through, and keyboard move and resize (section 38.1); Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-faced33f0aac` · PLAN lines 429–430: excluded from screen capture, the taskbar, and Alt+Tab by default (section 38.2). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-edc7c863d160` · PLAN lines 434–434: width: \~520 px logical; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e46dd8779b59` · PLAN lines 435–435: collapsed height: \~72--110 px; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e1dc70e61de` · PLAN lines 436–436: answer state: grows up to \~560 px; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-56add74416fe` · PLAN lines 437–437: minimum width: \~420 px; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-74022a802bcb` · PLAN lines 438–438: maximum width: \~720 px. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Asking

- [ ] `B-c404271c6d83` · PLAN lines 467–467: "what is he asking?" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-222e525f6503` · PLAN lines 468–468: "what should I mention here?" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b1931696828d` · PLAN lines 469–469: "what did we decide about the deployment?" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d31a08120079` · PLAN lines 470–470: "pull up the relevant details from Trace" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d87cd9313306` · PLAN lines 471–471: "summarize his last point" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fbb3f35c076e` · PLAN lines 472–472: "what did I say earlier about this?" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3d997167b5a6` · PLAN lines 473–473: "give me a concise answer" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0254fb7bd566` · PLAN lines 474–474: "what are the action items so far?" Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 6.1 Separate streams

- [ ] `B-6a9fdbd8cd24` · PLAN lines 570–570: microphone ≈ user; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e79bed0d34cd` · PLAN lines 571–571: system loopback ≈ remote participants. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 6.2 Windows system audio

- [ ] `B-26e0e0a90db1` · PLAN lines 582–582: capture the current default render endpoint; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b50626fb27b1` · PLAN lines 583–583: capture what the user hears from conferencing/browser apps; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e139a5c8e17a` · PLAN lines 584–584: handle device-format negotiation; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4668acaa8a78` · PLAN lines 585–585: resample to the STT engine's expected format; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-009d5766cc69` · PLAN lines 586–586: detect endpoint changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b9e25856794` · PLAN lines 587–587: restart the capture stream safely after device changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2ce18e693916` · PLAN lines 588–588: recover from Bluetooth/headset changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b2b8bdc2711f` · PLAN lines 589–590: avoid writing raw audio to disk unless explicitly required by a future recording feature. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 6.5 Audio pipeline

- [ ] `B-431f9c2258a4` · PLAN lines 641–641: SQLite; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-335653e94010` · PLAN lines 642–642: network; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-226d38c1ce8e` · PLAN lines 643–643: React; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fa5afbced6b1` · PLAN lines 644–644: model inference; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-07d569938941` · PLAN lines 645–645: disk; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-61985e26cdf9` · PLAN lines 646–646: long mutex waits. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 6.6 VAD

- [ ] `B-07f72c2f0114` · PLAN lines 657–657: identify speech onset; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-be5eca89697b` · PLAN lines 658–658: identify likely end of utterance; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2a31c16555b5` · PLAN lines 659–659: avoid sending long silence spans to STT; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e1537096d2c5` · PLAN lines 660–660: allow a small pre-roll buffer so initial phonemes are not clipped; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-353a5feb7899` · PLAN lines 661–661: merge very short pauses where appropriate. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 6.7 Audio failures

- [ ] `B-df07619bc4a0` · PLAN lines 670–670: unplugged microphone; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-31aae5ccbbf8` · PLAN lines 671–671: Bluetooth device switching; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-755f1fb61d62` · PLAN lines 672–672: default output change; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a25d4570f9b6` · PLAN lines 673–673: USB headset removal; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d88ea5739b0c` · PLAN lines 674–674: sleep/wake; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9f8e5a8dfa26` · PLAN lines 675–675: temporary exclusive-mode conflict; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-03c0347d5bca` · PLAN lines 676–676: no microphone permission; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ed6aff1874c7` · PLAN lines 677–677: no system audio. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 7.2 Backend selection

- [ ] `B-2be2ba7b8b8c` · PLAN lines 715–715: can be redistributed legally; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f9e3d98ec790` · PLAN lines 716–716: works on Windows x64; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bd8dc80a36a6` · PLAN lines 717–717: has a CPU path; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bd9412766e45` · PLAN lines 718–718: can use hardware acceleration where practical; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0222729db88a` · PLAN lines 719–719: can stream/approximate streaming with partial decoding; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-31e0b4b5460d` · PLAN lines 720–720: does not require Python at runtime. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 7.5 Target latency

- [ ] `B-d541f8d8e376` · PLAN lines 767–767: speech onset detected quickly; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-76d2e06c6f19` · PLAN lines 768–769: partial text begins appearing roughly within hundreds of milliseconds to low seconds depending on hardware/model; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-939df847f760` · PLAN lines 770–770: finalized utterance arrives shortly after the speaker pauses. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 9. Transcript storage

- [ ] `B-c56a7d4ea116` · PLAN lines 841–841: WAL mode; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-45850e8b9f2e` · PLAN lines 842–842: foreign keys; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-891552d92223` · PLAN lines 843–843: appropriate busy timeout; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6c55613bccee` · PLAN lines 844–844: migrations; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-aea464fae619` · PLAN lines 845–845: indexes on meeting/time/source fields; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-db226b1100d6` · PLAN lines 846–846: FTS5 for textual retrieval. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 10.1 Recent transcript buffer

- [ ] `B-9396270dcb44` · PLAN lines 887–887: last \~90 seconds, adjusted by token budget; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3a5c8cdd9ec9` · PLAN lines 888–888: preserve source role and timestamps; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-657ba9cb48fc` · PLAN lines 889–889: prioritize current conversational turn. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 10.2 Rolling state

- [ ] `B-191f2096a8b5` · PLAN lines 905–905: enough finalized text accumulated; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a220abd7713f` · PLAN lines 906–906: topic boundary; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6b6ca672c2f9` · PLAN lines 907–907: explicit question/decision language; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a297108ca507` · PLAN lines 908–908: elapsed interval while active. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 10.3 Final meeting record

- [ ] `B-4c27791244fe` · PLAN lines 917–917: full transcript; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bbbbd973e4eb` · PLAN lines 918–918: summary; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4843b7a075e4` · PLAN lines 919–919: decisions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e2464afdb391` · PLAN lines 920–920: action items; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-607457d908c0` · PLAN lines 921–921: extracted entities; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4483ac8c8462` · PLAN lines 922–922: links to memories referenced/created; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8cd60da52162` · PLAN lines 923–923: start/end timestamps. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Structured memory

- [ ] `B-5006de2f5442` · PLAN lines 939–939: projects; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1765329ed846` · PLAN lines 940–940: project technologies; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c37aec51a257` · PLAN lines 941–941: project decisions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-66bf904b2a88` · PLAN lines 942–942: work experience; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7ed2cc0506d8` · PLAN lines 943–943: education; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-86776720d228` · PLAN lines 944–944: people; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f59d07537371` · PLAN lines 945–945: organizations; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e4aa3f6c00c8` · PLAN lines 946–946: preferences; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c7b75b869ff6` · PLAN lines 947–947: recurring terms/entities. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Semantic memory

- [ ] `B-a921e1fe4159` · PLAN lines 953–953: project descriptions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c7199e81812c` · PLAN lines 954–954: notes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d96441b61024` · PLAN lines 955–955: documents; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-adadd41df96e` · PLAN lines 956–956: previous meeting summaries; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f0c29510a710` · PLAN lines 957–957: imported markdown/text/PDF-derived text when supported. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Episodic memory

- [ ] `B-34f97dee1e80` · PLAN lines 963–963: meetings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b75b4efbf98` · PLAN lines 964–964: conversations; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-33b6b5da710e` · PLAN lines 965–965: decisions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c9755654ea49` · PLAN lines 966–966: action items; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-17c4c284982b` · PLAN lines 967–967: project changes. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 11.3 Provenance

- [ ] `B-f347abf72319` · PLAN lines 992–992: where did this come from? Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-320ec94faf12` · PLAN lines 993–993: when was it created? Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3b176f090fe9` · PLAN lines 994–994: which meeting/document/manual entry? Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-45b8583e3d7f` · PLAN lines 995–995: has it been superseded? Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 11.5 Memory UI

- [ ] `B-aa2b19ea95fc` · PLAN lines 1040–1040: search memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-39cbee276012` · PLAN lines 1041–1041: inspect a memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f42eb1714c0d` · PLAN lines 1042–1042: see provenance; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3400b27ba67f` · PLAN lines 1043–1043: add a memory manually; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6e00017941e4` · PLAN lines 1044–1044: edit user-created memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a73889766f0e` · PLAN lines 1045–1045: delete memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-317f25c388ec` · PLAN lines 1046–1046: inspect projects/entities; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9efe7361f5e5` · PLAN lines 1047–1047: import supported documents; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0708b52f8baa` · PLAN lines 1048–1048: delete meeting history. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 12. Document ingestion

- [ ] `B-a8e865370d85` · PLAN lines 1062–1062: `.md` Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d4e7a677d0e1` · PLAN lines 1063–1063: `.txt` Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-86f709d5f6a7` · PLAN lines 1064–1064: `.json` where text extraction is meaningful Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b593b3676f94` · PLAN lines 1065–1065: `.pdf` if a reliable local text extraction library is bundled Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d2cf26eaa242` · PLAN lines 1066–1066: optionally `.docx` after the core path is stable Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d1c715bb855b` · PLAN lines 1094–1094: original path; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-45afc76fbe0a` · PLAN lines 1095–1095: content hash; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8614bc78edfe` · PLAN lines 1096–1096: modified time; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d2f615ec6a25` · PLAN lines 1097–1097: extracted text/chunks; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c63575933193` · PLAN lines 1098–1098: indexing version. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 14. ChatGPT authentication and inference

- [ ] `B-406b4f1f1f8c` · PLAN lines 1192–1192: eligible Plus/Pro users can authorize ChatGPT-plan usage; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5187688cf4db` · PLAN lines 1193–1193: open-source clients can dynamically register; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-34b931d47bcd` · PLAN lines 1194–1194: first registration uses an installation host ID; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-049ca1b3e894` · PLAN lines 1195–1195: no user-supplied API key should be required for the intended path; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-28cead0b2dfd` · PLAN lines 1196–1197: no client secret should be embedded for the open-source dynamic flow; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dcc0140e44d7` · PLAN lines 1198–1199: the authorization does not provide access to ChatGPT conversation history; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d5ad586e195d` · PLAN lines 1200–1200: requests use eligible Responses API access and should stream. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 14.1 Secrets

- [ ] `B-74a29cf840e3` · PLAN lines 1224–1224: in localStorage; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-940e8e0f66d1` · PLAN lines 1225–1225: in frontend state longer than necessary; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c3ddeaf85f45` · PLAN lines 1226–1226: in plaintext JSON; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0e89fd415651` · PLAN lines 1227–1227: in SQLite; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-86bb0b475c2f` · PLAN lines 1228–1228: in logs. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 14.3 Offline behavior

- [ ] `B-0e1e8134b16a` · PLAN lines 1274–1274: capture; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f0431e403763` · PLAN lines 1275–1275: transcribe; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-30a15e66dc41` · PLAN lines 1276–1276: persist; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dcb68cddf171` · PLAN lines 1277–1277: search local transcript; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-960f15d9bbee` · PLAN lines 1278–1278: search local memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bf9061adee13` · PLAN lines 1279–1279: show meeting state available locally. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 17. Frontend architecture

- [ ] `B-801f0380855a` · PLAN lines 1582–1582: React; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-748c3f4b18cb` · PLAN lines 1583–1583: TypeScript; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c136331df35c` · PLAN lines 1584–1584: Vite; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0f7b1138e524` · PLAN lines 1585–1585: minimal state library only if needed; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-42be15f1fdad` · PLAN lines 1586–1586: CSS/Tailwind only if it does not inflate complexity unnecessarily. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-16359cbce654` · PLAN lines 1592–1592: capture audio; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b32ee96e6b08` · PLAN lines 1593–1593: own OAuth tokens; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8d3c281ce9cf` · PLAN lines 1594–1594: access SQLite directly; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-728cdac38fe5` · PLAN lines 1595–1595: run STT; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bc1ccb6d7c1f` · PLAN lines 1596–1596: implement retrieval; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b4e2ca310efe` · PLAN lines 1597–1597: compile prompts. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 19.1 Idle

- [ ] `B-9766cc47e747` · PLAN lines 1654–1655: audio capture stopped unless ambient listening was explicitly enabled; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-603a9c3dcf84` · PLAN lines 1656–1656: STT model may be unloaded after a configurable idle period; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6ce315868937` · PLAN lines 1657–1657: no polling loops at high frequency; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cb641cdd6a89` · PLAN lines 1658–1658: near-zero sustained CPU usage; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-393c9b41b110` · PLAN lines 1659–1659: target base process memory below \~100--150 MB where practical. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 19.2 Active meeting

- [ ] `B-13d62998e8e3` · PLAN lines 1667–1667: capture without audio gaps; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cd7e4b54deaa` · PLAN lines 1668–1668: transcript processing does not block UI; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3bcbf2bdc376` · PLAN lines 1669–1669: overlay invocation feels instant; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a65af837c47f` · PLAN lines 1670–1670: model inference streaming begins as soon as context is ready; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-500334817fa3` · PLAN lines 1671–1671: UI remains responsive while STT is saturated. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6d763f75ada2` · PLAN lines 1675–1675: app base RSS; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-82129fc831b3` · PLAN lines 1676–1676: STT model memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f44f446b692d` · PLAN lines 1677–1677: WebView memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3a64360368e8` · PLAN lines 1678–1678: GPU/accelerator memory. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 19.3 Overlay

- [ ] `B-bc64ae7b10ad` · PLAN lines 1688–1688: shortcut timestamp; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-adffb9caebba` · PLAN lines 1689–1689: Rust event receipt; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f857ffdc75e6` · PLAN lines 1690–1690: window show request; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7a6daff25c13` · PLAN lines 1691–1691: frontend visible/paint acknowledgment if available. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 19.4 Retrieval

- [ ] `B-aca07cac6992` · PLAN lines 1700–1700: hundreds of meetings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cededcd85d58` · PLAN lines 1701–1701: hundreds of thousands of transcript segments/chunks; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c68d0bc179cd` · PLAN lines 1702–1702: thousands of memories/doc chunks; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 20. Error model

- [ ] `B-5dcb2dbd1987` · PLAN lines 1726–1726: audio device; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-73f46732fd53` · PLAN lines 1727–1727: STT/model; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ba00b53ee59a` · PLAN lines 1728–1728: database; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-39e10fe71117` · PLAN lines 1729–1729: document ingestion; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5ec91e8fd323` · PLAN lines 1730–1730: authentication; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5f9b47f03210` · PLAN lines 1731–1731: inference/network; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ea167576801e` · PLAN lines 1732–1732: shortcut conflict; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-42c93b941f13` · PLAN lines 1733–1733: installer/runtime; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a94d4341b83f` · PLAN lines 1734–1734: migration; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a3e07f0acae7` · PLAN lines 1735–1735: secure credential storage. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 21. Logging and diagnostics

- [ ] `B-779be309e974` · PLAN lines 1747–1747: OAuth tokens; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-410e02aa1154` · PLAN lines 1748–1748: raw authorization headers; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-303938136cda` · PLAN lines 1749–1749: full private documents; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e2230f21736d` · PLAN lines 1750–1750: complete prompts; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5f9e9e3c321e` · PLAN lines 1751–1751: full transcripts unless explicit debug mode is enabled. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1b02e7389ec5` · PLAN lines 1755–1755: component; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cec942b6f99e` · PLAN lines 1756–1756: severity; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e2465befc51e` · PLAN lines 1757–1757: timestamp; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b605fc97286b` · PLAN lines 1758–1758: meeting ID; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9f83f4ebe524` · PLAN lines 1759–1759: event latency; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c3cf51f539a6` · PLAN lines 1760–1760: device changes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-73f658d1589b` · PLAN lines 1761–1761: model load time; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-44bd38abf1ce` · PLAN lines 1762–1762: STT real-time factor; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b3adcc428081` · PLAN lines 1763–1763: retrieval latency; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-647bc27a83c4` · PLAN lines 1764–1764: context compilation latency; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1074187049b9` · PLAN lines 1765–1765: inference first-token latency; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c3365c20906b` · PLAN lines 1766–1766: errors. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 22.1 Trust boundaries

- [ ] `B-c117421532e6` · PLAN lines 1778–1778: transcript content; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c74809d0947a` · PLAN lines 1779–1779: system audio-derived text; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-559f22462b6b` · PLAN lines 1780–1780: imported documents; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e268e1b57eae` · PLAN lines 1781–1781: document metadata; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-639943480d4e` · PLAN lines 1782–1782: model output. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-36bc44f5829a` · PLAN lines 1786–1786: OAuth tokens; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7dcc0e324d73` · PLAN lines 1787–1787: personal memory; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-64935adddf75` · PLAN lines 1788–1788: transcript database; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-592f390af27a` · PLAN lines 1789–1789: imported document text. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 22.3 Local data

- [ ] `B-049d62caca19` · PLAN lines 1807–1807: credentials use Windows secure storage; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-632303215948` · PLAN lines 1808–1808: database is only accessible under the user's profile permissions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5d78ac92d8a7` · PLAN lines 1809–1809: deletion functions actually delete requested local records; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cd62818253e6` · PLAN lines 1810–1811: no analytics/telemetry leaves the machine without explicit design/consent. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 23.4 STT assets

- [ ] `B-b81b59a35350` · PLAN lines 1867–1867: native STT runtime DLLs/executable/library as appropriate; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d20634a0e45e` · PLAN lines 1868–1868: default model; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2e90fe2ebe14` · PLAN lines 1869–1869: licenses/attributions required for redistribution; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2f8398c893cb` · PLAN lines 1870–1870: checksums/version manifest. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 23.6 Code signing

- [ ] `B-09a3090c22f8` · PLAN lines 1895–1895: obtain a Windows code-signing certificate; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-809ecc00e675` · PLAN lines 1896–1896: sign the application executable; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d57d44742140` · PLAN lines 1897–1897: sign the installer; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e7d569b6dd2d` · PLAN lines 1898–1898: timestamp signatures. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 23.7 Updates

- [ ] `B-3f85384f5fdc` · PLAN lines 1909–1909: database; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9037eb604129` · PLAN lines 1910–1910: memories; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-584882b16c02` · PLAN lines 1911–1911: documents/indexes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-73421a441e30` · PLAN lines 1912–1912: user settings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f3b8601517a0` · PLAN lines 1913–1913: secure credentials. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 24. Build pipeline

- [ ] `B-21b4cd312a8e` · PLAN lines 1928–1928: Windows 11 build machine or Windows CI runner; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4e5e4f8dbf95` · PLAN lines 1929–1929: Rust stable pinned via toolchain file; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2a95d29c9e66` · PLAN lines 1930–1930: Node LTS; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6c952eb331e9` · PLAN lines 1931–1931: npm pinned; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d5ebd237d29e` · PLAN lines 1932–1932: Visual Studio Build Tools required by Tauri/MSVC; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e915fcee2ff0` · PLAN lines 1933–1933: Tauri CLI. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Unit tests

- [ ] `B-1ee3a83ab678` · PLAN lines 2005–2005: audio resampling; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-59ee6055714a` · PLAN lines 2006–2006: ring buffer behavior; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3b8d6b156542` · PLAN lines 2007–2007: VAD segmentation; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-75fb5cbf2ed9` · PLAN lines 2008–2008: transcript revision/finalization; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7cc90bb16299` · PLAN lines 2009–2009: token/context budgeting; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bb19df0e56b1` · PLAN lines 2010–2010: retrieval ranking; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4895d10482bb` · PLAN lines 2011–2011: memory provenance; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-660e9d53476c` · PLAN lines 2012–2012: database repositories; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4d3ec29e4d89` · PLAN lines 2013–2013: auth state transitions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cce331503338` · PLAN lines 2014–2014: prompt/context injection boundaries. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e7cc07c4bd0` · PLAN lines 2018–2018: overlay state machine; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ddb52a29379c` · PLAN lines 2019–2019: keyboard interactions; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-66f6681bdcb4` · PLAN lines 2020–2020: answer streaming reducer; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a18b16437d58` · PLAN lines 2021–2021: settings forms. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Integration tests

- [ ] `B-59bee6dcad27` · PLAN lines 2025–2025: mocked audio → STT → transcript; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7f7aac8a1c85` · PLAN lines 2026–2026: transcript → SQLite; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-66668bda1b6e` · PLAN lines 2027–2027: query → hybrid retrieval; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7aed53834fc4` · PLAN lines 2028–2028: query + meeting → context compiler; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d0cdf932536a` · PLAN lines 2029–2029: mocked ChatGPT stream → Rust → Tauri event → UI; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-83dfc12a40cd` · PLAN lines 2030–2030: auth callback parsing with safe test fixtures; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f33582aa2a68` · PLAN lines 2031–2031: database migration from previous schema version. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Windows tests

- [ ] `B-70c03dd7e678` · PLAN lines 2035–2035: global shortcut; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-450c80a67a70` · PLAN lines 2036–2036: tray; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-35dd40efb6e6` · PLAN lines 2037–2037: always-on-top; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-321eb7d9aed7` · PLAN lines 2038–2038: multi-monitor; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-358880fe7346` · PLAN lines 2039–2039: 100/125/150/200% DPI; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-60fff0563c18` · PLAN lines 2040–2040: device hot-plug; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7b48f03031a1` · PLAN lines 2041–2041: Bluetooth switching; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1c8a88989a9c` · PLAN lines 2042–2042: sleep/wake; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-92433b53923a` · PLAN lines 2043–2043: microphone denied; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c342abdeb737` · PLAN lines 2044–2044: system output changed; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0505a17e10ac` · PLAN lines 2045–2045: no network; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7ce437b419f6` · PLAN lines 2046–2046: slow network; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0c506745954e` · PLAN lines 2047–2048: overlay absent from Zoom, Teams, and Google Meet screen share, OBS, and Snipping Tool captures; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d18b0dc15825` · PLAN lines 2049–2049: overlay absent from the taskbar and Alt+Tab; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-284b91f40119` · PLAN lines 2050–2051: move, resize, and opacity shortcuts registered only while the overlay is visible; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d3cd7fcfaea4` · PLAN lines 2052–2052: both screenshot shortcuts on single and multi-monitor setups; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1f6191c89321` · PLAN lines 2053–2053: failover from a limited ChatGPT account to an API-key provider. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Long-running test

- [ ] `B-40d7fc2fa837` · PLAN lines 2061–2061: memory does not grow without bound; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-03e70f22301e` · PLAN lines 2062–2062: transcript remains correct; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-75418a5dcb16` · PLAN lines 2063–2063: queues remain bounded; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7ffbc894d5db` · PLAN lines 2064–2064: no progressive UI slowdown; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ac4346a93ae6` · PLAN lines 2065–2065: database WAL remains manageable; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6795106d0999` · PLAN lines 2066–2066: STT real-time factor remains stable; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bf422b4a3b82` · PLAN lines 2067–2067: no audio callback starvation. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase A --- Desktop foundation

- [ ] `B-8de64efcabde` · PLAN lines 2110–2110: Tauri shell; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-330f47411da6` · PLAN lines 2111–2111: Rust core structure; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-73cd82d50185` · PLAN lines 2112–2112: React overlay; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3a214c867c99` · PLAN lines 2113–2113: tray; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7d61867a75f2` · PLAN lines 2114–2114: single-instance behavior; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-114733361794` · PLAN lines 2115–2115: global shortcut; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a73c3f34ae4d` · PLAN lines 2116–2116: overlay show/hide; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b4ff40431a04` · PLAN lines 2117–2117: Windows startup option; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-39fdb7db2ea7` · PLAN lines 2118–2118: local settings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2fcf3b4c82dc` · PLAN lines 2119–2119: SQLite migrations; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f85f0cbf49ad` · PLAN lines 2120–2120: structured logging; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-92ee86065bd5` · PLAN lines 2121–2121: overlay opacity, click-through, and move and resize shortcuts; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ba71cf012718` · PLAN lines 2122–2122: capture exclusion, taskbar hiding, and Alt+Tab hiding. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase B --- Audio

- [ ] `B-a5309f6a254f` · PLAN lines 2131–2131: device enumeration; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c354368a4f0e` · PLAN lines 2132–2132: system WASAPI loopback; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-608f9092ca02` · PLAN lines 2133–2133: microphone WASAPI; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a4884a1241f9` · PLAN lines 2134–2134: separate bounded buffers; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c0a53679f922` · PLAN lines 2135–2135: resampling; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-126e7649e675` · PLAN lines 2136–2136: VAD; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bcae4b1bfab7` · PLAN lines 2137–2137: hot-plug/device recovery; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-96c4c167116b` · PLAN lines 2138–2138: audio diagnostics. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase C --- Local STT

- [ ] `B-e2b9c953e1d7` · PLAN lines 2147–2147: native speech engine abstraction; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7f5a0cb4a35d` · PLAN lines 2148–2148: default backend; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e9a70f9f7dff` · PLAN lines 2149–2149: model packaging; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-621501333ee0` · PLAN lines 2150–2150: partial/final events; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ee94294bf5ea` · PLAN lines 2151–2151: source tagging; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-13e319aae93d` · PLAN lines 2152–2152: persistence; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d3ebcde2be10` · PLAN lines 2153–2153: transcript UI. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase D --- Meeting engine

- [ ] `B-783efc28bbcf` · PLAN lines 2162–2162: meeting lifecycle; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-26c999e4cb84` · PLAN lines 2163–2163: recent transcript buffer; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c5b4cd74f6d4` · PLAN lines 2164–2164: meeting transcript search; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c659bce58658` · PLAN lines 2165–2165: rolling state; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7490a47256e7` · PLAN lines 2166–2166: artifacts; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7bdf2ee68984` · PLAN lines 2167–2167: completed meeting history. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase E --- Memory and documents

- [ ] `B-8335eafd482e` · PLAN lines 2173–2173: structured memories; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-694ead31056a` · PLAN lines 2174–2174: project/person entities; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e446df0df7e` · PLAN lines 2175–2175: document ingestion; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f322ee1b77a8` · PLAN lines 2176–2176: FTS5; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-790e2a136e52` · PLAN lines 2177–2177: local embeddings; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4d4d473ac10e` · PLAN lines 2178–2178: hybrid retrieval; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fc37cd980355` · PLAN lines 2179–2179: reranking; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6f47dabee78c` · PLAN lines 2180–2180: provenance; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f800e1602790` · PLAN lines 2181–2181: management UI. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase F --- ChatGPT

- [ ] `B-03fe463c2930` · PLAN lines 2187–2187: Sign in with ChatGPT; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ef36d920ddc0` · PLAN lines 2188–2188: secure credential persistence; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9954d29b0ea6` · PLAN lines 2189–2189: account state UI; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7c68f0b6b872` · PLAN lines 2190–2190: model discovery/selection as required; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-65f20d62f90b` · PLAN lines 2191–2191: streaming inference; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-94fa78c4e9bb` · PLAN lines 2192–2192: reconnection/refresh; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c7ba15f1b138` · PLAN lines 2193–2193: sign-out; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-21ff5787dd49` · PLAN lines 2194–2194: more than one ChatGPT account; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0d497ecf98c2` · PLAN lines 2195–2195: OpenAI-compatible API-key providers and the fallback chain; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d2973186c47c` · PLAN lines 2196–2196: usage events and the usage page. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase G --- Context intelligence

- [ ] `B-cc9858493884` · PLAN lines 2202–2202: query understanding; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dd7b0158e6ef` · PLAN lines 2203–2203: parallel retrieval; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-061e3e69197a` · PLAN lines 2204–2204: context compiler; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-eb7e30d6bbe3` · PLAN lines 2205–2205: token budgeting; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c7f6e3e80de9` · PLAN lines 2206–2206: meeting + personal memory fusion; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-156740c9bfa7` · PLAN lines 2207–2207: prompt injection boundaries; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-33a9132cdab6` · PLAN lines 2208–2208: source chips; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5a530556bcd4` · PLAN lines 2209–2209: streaming answer UX; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-43b57928ff8a` · PLAN lines 2210–2210: streaming Markdown and code rendering; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-55cdb6df921f` · PLAN lines 2211–2211: context profiles and the context inspector; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5b10dc990fe8` · PLAN lines 2212–2212: screenshot image and OCR flows. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Phase H --- Product hardening

- [ ] `B-5da8a2e78ce7` · PLAN lines 2218–2218: performance profiling; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c2fdad43e497` · PLAN lines 2219–2219: keyboard polish; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4234af79ecc9` · PLAN lines 2220–2220: audio recovery; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-32c27980c25d` · PLAN lines 2221–2221: offline degradation; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-38e94c117c0c` · PLAN lines 2222–2222: memory deletion; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c0e6e27ef62e` · PLAN lines 2223–2223: diagnostics; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6af265bdc578` · PLAN lines 2224–2224: accessibility basics; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f84d480a586b` · PLAN lines 2225–2225: crash recovery; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-823b78e67923` · PLAN lines 2226–2226: installer; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d9ce03a5f50d` · PLAN lines 2227–2227: code signing configuration; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-920c09b75eb8` · PLAN lines 2228–2228: updater; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-37c8f59adab5` · PLAN lines 2229–2229: clean-machine test. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Installation

- [ ] `B-d5c4899e8afa` · PLAN lines 2241–2242: One `Harness-Setup-x64.exe` can install the application on a compatible clean Windows x64 machine. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8a3a7b846902` · PLAN lines 2243–2243: No development tools are required. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-66a2d5f81fc9` · PLAN lines 2244–2244: The local STT engine works after installation. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7061257baf01` · PLAN lines 2245–2245: The installer handles WebView2. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-baf65bc5021f` · PLAN lines 2246–2246: Uninstall works. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Desktop behavior

- [ ] `B-9417f407e65a` · PLAN lines 2250–2250: Tray behavior is reliable. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4f751a16de0b` · PLAN lines 2251–2251: Global shortcut works. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f3e10937b79c` · PLAN lines 2252–2252: Overlay appears quickly. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-006651745e97` · PLAN lines 2253–2253: Overlay behaves across multiple monitors and DPI settings. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cd8b5757b870` · PLAN lines 2254–2254: Only one app instance captures audio. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Audio/STT

- [ ] `B-e03138b975e9` · PLAN lines 2258–2258: System audio transcribes locally. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-457ec2c67652` · PLAN lines 2259–2259: Microphone transcribes locally. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4171816eaa6f` · PLAN lines 2260–2260: Sources remain distinguishable. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9b04f57b3981` · PLAN lines 2261–2261: Raw audio is not sent to OpenAI. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2425b83b9a79` · PLAN lines 2262–2262: Device changes do not normally crash the meeting. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Meetings

- [ ] `B-258b01bc56e3` · PLAN lines 2266–2266: Meeting can start/stop. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-97204d8e1d41` · PLAN lines 2267–2267: Transcript persists incrementally. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1eb8058faf65` · PLAN lines 2268–2268: Recent context is available immediately. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-91e16d6fb5aa` · PLAN lines 2269–2269: Previous meeting text is locally searchable. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e76d4a7f9493` · PLAN lines 2270–2270: Meeting state/artifacts persist. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Memory

- [ ] `B-65018e7a1eec` · PLAN lines 2274–2274: User can add/import context. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-76b52575a202` · PLAN lines 2275–2275: Memory remains local. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-aff4a041cc48` · PLAN lines 2276–2276: Retrieval combines lexical, semantic, and structured signals. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cb5b7affb690` · PLAN lines 2277–2277: Provenance is inspectable. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ac42a26b7616` · PLAN lines 2278–2278: Memories can be deleted. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## AI

- [ ] `B-7d3214700fa4` · PLAN lines 2282–2282: User can `Continue with ChatGPT`. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e324b5c2f02c` · PLAN lines 2283–2283: No API key is required for the intended eligible-plan path. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4b43e3a25f5a` · PLAN lines 2284–2284: Tokens are stored securely outside frontend/SQLite. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3559ff6246e5` · PLAN lines 2285–2285: Responses stream. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7912d4a4888c` · PLAN lines 2286–2286: Questions can combine live meeting context and personal memory. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8868c902b76d` · PLAN lines 2287–2287: Lack of network does not destroy local meeting functionality. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Overlay, privacy, and screen input

- [ ] `B-c0a7de36239e` · PLAN lines 2291–2292: Opacity, click-through, move, and resize work from shortcuts and settings. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2576a87c8f7d` · PLAN lines 2293–2293: Harness windows do not appear in screen share or the taskbar. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5ddadba1d59b` · PLAN lines 2294–2294: `Ctrl+Shift+O` sends a screenshot image to an image-capable provider. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e62b1d1389a7` · PLAN lines 2295–2295: `Ctrl+Shift+X` sends local OCR text. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-31cd1b96788f` · PLAN lines 2296–2296: Answers render Markdown and highlighted code while streaming. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Accounts and providers

- [ ] `B-72161a8b8bd5` · PLAN lines 2300–2301: More than one ChatGPT account can be signed in, and the user can switch between them. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d318740c7e87` · PLAN lines 2302–2303: DeepSeek and Gemini keys work as fallbacks and are stored in Credential Manager. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-04a324e35fcf` · PLAN lines 2304–2304: The usage page shows per-account and per-provider usage. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Distribution

- [ ] `B-e61ed062dea9` · PLAN lines 2308–2308: Release build is reproducible from documented scripts/CI. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-58d244ca2c49` · PLAN lines 2309–2309: Installer is tested in a clean VM. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d2b13fc3a424` · PLAN lines 2310–2310: Public release can be code-signed. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-72e52219d5d8` · PLAN lines 2311–2311: No secret keys are embedded in the binary. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 30. Explicit non-goals

- [ ] `B-767641d7447b` · PLAN lines 2319–2319: macOS; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ae62b4fcbceb` · PLAN lines 2320–2320: Linux; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3ccfcac08113` · PLAN lines 2321–2321: mobile; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2605415ecb8c` · PLAN lines 2322–2322: browser extension; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-72bc7813ace8` · PLAN lines 2323–2323: cloud transcript synchronization; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f9c9c535e264` · PLAN lines 2324–2324: Harness-hosted user accounts; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f85b0236652c` · PLAN lines 2325–2325: team workspaces; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7cd78c354dab` · PLAN lines 2326–2326: remote vector databases; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b944139b1ac3` · PLAN lines 2327–2327: a web dashboard; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7215147c8aed` · PLAN lines 2328–2328: full CRM features; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3169d002a924` · PLAN lines 2329–2329: video capture; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c31673a66889` · PLAN lines 2330–2331: automatic or background screen capture (the screenshot shortcuts in section 38.7 are in scope); Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f23145e0113b` · PLAN lines 2332–2332: autonomous clicking/typing in third-party apps; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6bdd4efac2a2` · PLAN lines 2333–2333: detecting or defeating proctoring and anti-cheat software; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-23cee0e8865a` · PLAN lines 2334–2334: remote audio transcription; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b835cafa1e9e` · PLAN lines 2335–2335: storing raw meeting audio by default. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 33. Questions Codex should NOT ask during implementation

- [ ] `B-a3d5a3f6f702` · PLAN lines 2436–2436: Is this Electron? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8d15c5e88500` · PLAN lines 2437–2437: Is this a web app? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0f48ee8543d6` · PLAN lines 2438–2438: Should transcription use a cloud API? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fae1761599f8` · PLAN lines 2439–2439: Should raw audio go to OpenAI? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c2f4901be295` · PLAN lines 2440–2440: Should system and mic audio be mixed? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-90b2d7cf5fb1` · PLAN lines 2441–2441: Should we require Python on user machines? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-076801200676` · PLAN lines 2442–2442: Should we use Postgres/Redis? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b4a5d9df2cec` · PLAN lines 2443–2443: Should we build a Harness backend first? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-311f9b020de9` · PLAN lines 2444–2444: Should the main UX be a full-screen chat app? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4e3e02695697` · PLAN lines 2445–2446: Should the app disappear when the overlay closes? **No; it remains in the tray.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-242677af300f` · PLAN lines 2447–2449: Should we ask users for an OpenAI API key? **Not for the intended ChatGPT-plan path.** DeepSeek and Gemini keys are optional fallbacks (section 38.4). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-abd48e70d7f0` · PLAN lines 2450–2452: Should Harness rotate ChatGPT accounts automatically when one hits a limit? **No.** The user switches. Automatic failover goes to API-key providers. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c78c49d0eecb` · PLAN lines 2453–2453: Should memory be sent wholesale on every request? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9c8bbc23b1bd` · PLAN lines 2454–2454: Should V0 be treated as disposable? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-26e558f369aa` · PLAN lines 2455–2455: Is installer work optional? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7900f858744f` · PLAN lines 2456–2456: Can a release require Node/Rust/Python on the target PC? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-924bfd076395` · PLAN lines 2457–2457: Can we silently upload documents for indexing? **No.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c48fa810f508` · PLAN lines 2458–2459: Should the model decide what application state is true? **No; local state is authoritative.** Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 36. External implementation references

- [ ] `B-f065bf58927c` · PLAN lines 2566–2567: OpenAI --- Sign in with ChatGPT for open-source apps: https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-dac185449f90` · PLAN lines 2568–2569: OpenAI --- Sign in with ChatGPT quickstart: https://developers.openai.com/siwc/quickstart Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d84e5624aa61` · PLAN lines 2570–2571: OpenAI --- open-source ChatGPT plan usage overview: https://developers.openai.com/siwc/token-sharing-open-source Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-baf6709281cf` · PLAN lines 2572–2573: OpenAI --- registration/sign-in flow: https://developers.openai.com/siwc/token-sharing-open-source/sign-in Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6da2b0ce33c2` · PLAN lines 2574–2575: Tauri --- Windows installer: https://v2.tauri.app/distribute/windows-installer/ Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c8aa65abd64f` · PLAN lines 2576–2577: Tauri --- global shortcut plugin: https://v2.tauri.app/plugin/global-shortcut/ Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d482172a7986` · PLAN lines 2578–2579: Tauri --- configuration reference: https://v2.tauri.app/reference/config/ Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4f9a7f01ec3f` · PLAN lines 2580–2581: Tauri --- Windows prerequisites: https://v2.tauri.app/start/prerequisites/ Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-bae81043231e` · PLAN lines 2582–2583: Microsoft --- `SetWindowDisplayAffinity`: https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7cdebf4d7aa8` · PLAN lines 2584–2585: Microsoft --- `Windows.Media.Ocr`: https://learn.microsoft.com/uwp/api/windows.media.ocr Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-0104cf1d46ac` · PLAN lines 2586–2586: DeepSeek API docs: https://api-docs.deepseek.com Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b0780aace0bd` · PLAN lines 2587–2588: Gemini OpenAI compatibility: https://ai.google.dev/gemini-api/docs/openai Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-350bcbee7125` · PLAN lines 2589–2589: streamdown: https://github.com/vercel/streamdown Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ae119dad3420` · PLAN lines 2590–2590: xcap: https://crates.io/crates/xcap Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-48e9c59d9ad2` · PLAN lines 2591–2591: keyring: https://crates.io/crates/keyring Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.1 Overlay opacity, position, and size shortcuts

- [ ] `B-35268a2962fb` · PLAN lines 2639–2639: background opacity, 15 to 100%, default 85%; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ddc06ec69e4b` · PLAN lines 2640–2640: text opacity, 50 to 100%, default 100%; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-00827435fb64` · PLAN lines 2641–2641: click-through, default off; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-967385308410` · PLAN lines 2642–2642: move step, default 20 px logical; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a67d262e8901` · PLAN lines 2643–2643: resize step, default 40 px logical. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3153e1f29e50` · PLAN lines 2664–2668: `tauri-plugin-global-shortcut` calls Win32 `RegisterHotKey`. A registered combination stops reaching every other app. Register the "always" rows at startup. Register the "overlay visible" rows when the overlay shows and unregister them when it hides, so other apps keep those keys while the overlay is hidden. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-64798de5fea3` · PLAN lines 2669–2671: If registration fails, another app already owns the combination. The shortcut editor shows "Taken by another app" and keeps the old binding. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-32fc0439e773` · PLAN lines 2672–2676: Shortcuts that live inside one app never fail registration, and Harness takes them over. The shortcut editor warns about the known collisions. `Ctrl+Shift+O` opens Chrome's bookmark manager and VS Code's Go to Symbol. `Ctrl+Shift+X` opens VS Code's Extensions view. `Ctrl+Alt+Arrow` rotates the display on some Intel graphics drivers. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cb7641074d05` · PLAN lines 2677–2680: To move or resize the overlay, call `set_position` or `set_size` on the `WebviewWindow` with logical units. Then clamp the window to `current_monitor()` work area, the same rule as section 4.3. Debounce the settings write by 500 ms. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e016fe086818` · PLAN lines 2681–2685: For opacity, set `transparent: true` and `decorations: false` on the overlay window in `tauri.conf.json`. Settings drive two CSS variables, `--overlay-bg-alpha` and `--overlay-text-alpha`. No Win32 layered-window call is needed, and text stays sharp because the background alpha changes on its own. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-9663a30f7fc4` · PLAN lines 2686–2689: For click-through, call `set_ignore_cursor_events(true)`. Mouse input then goes to the window below, and shortcuts still work. Show a small "Click-through" badge so the user knows why clicks pass through. Only the shortcut or the tray can turn click-through off. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-970a255b1e09` · PLAN lines 2690–2694: `Ctrl+Space` shows the overlay with focus in the input. The transcript and screenshot shortcuts show it without taking focus from the meeting app or editor. To do that, get the HWND from `window.hwnd()` and call `ShowWindow(hwnd, SW_SHOWNOACTIVATE)` through the `windows` crate. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.2 Hide from taskbar, Alt+Tab, and screen capture

- [ ] `B-38d172058b96` · PLAN lines 2716–2721: For capture exclusion, set `contentProtected: true` in the window config, or call `window.set_content_protected(true)` at runtime. On Windows this calls `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)`. Electron's `setContentProtection(true)` makes the same call. Apply it to the overlay and the management window. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-006bff4c45e4` · PLAN lines 2722–2725: `WDA_EXCLUDEFROMCAPTURE` needs Windows 10 2004 (build 19041) or later. Older builds support only `WDA_MONITOR`, which shows a black rectangle in the share instead of hiding the window. On those builds, the Privacy page says so. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4bea49fc30f5` · PLAN lines 2726–2726: For the taskbar, set `skipTaskbar: true` in the window config. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fef13a62ad8c` · PLAN lines 2727–2730: Windows lists every visible top-level window in Alt+Tab unless the window has the `WS_EX_TOOLWINDOW` extended style. Add it with `SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_TOOLWINDOW)` (`windows` crate, `Win32_UI_WindowsAndMessaging` feature). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-fa4fc976f826` · PLAN lines 2731–2734: Windows notifications, the tray menu, and native file dialogs are separate windows that Harness cannot protect. Harness raises no notifications during a meeting. The tray menu appears only when the user clicks it. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7e733129b895` · PLAN lines 2735–2737: If the user hides the tray icon, they can still reach Harness from the Start menu. A second launch shows the existing instance (section 18.4). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c42a9c7177ea` · PLAN lines 2741–2741: a camera or phone pointed at the screen; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-ecd242cd5ac6` · PLAN lines 2742–2742: hardware capture devices; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2e01dc1b6083` · PLAN lines 2743–2743: the Harness process in Task Manager; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-2e9f75e330e7` · PLAN lines 2744–2744: the tray icon, unless the user hides it. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.3 More than one ChatGPT account

- [ ] `B-245cc55001c0` · PLAN lines 2752–2753: The Accounts page lists each signed-in account with its label, email, status, last limit hit, and reset time. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-351bee585ab7` · PLAN lines 2754–2756: **Add account** runs the Sign in with ChatGPT flow again. Each account's tokens go into Credential Manager under the target `Harness/chatgpt/<account_id>`. The `keyring` crate covers this. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-89d6048e9526` · PLAN lines 2757–2760: One account is active at a time. The pre-meeting settings panel picks it, and the meeting snapshots it. `Ctrl+Alt+A` switches to the next account, and the overlay shows the new account's label for two seconds. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a55aac2d5ff3` · PLAN lines 2761–2764: When the active account hits a usage limit, Harness keeps the unsent message, shows "Account <label> limited until <time>", and offers three single-key choices: next account, next fallback provider, or wait. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7586d60c61c2` · PLAN lines 2765–2768: Harness never rotates ChatGPT accounts automatically to get past a limit. OpenAI's terms of use forbid circumventing rate limits, and breaking them can cost the accounts. Automatic failover goes only to API-key providers (38.4). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4471d3f10679` · PLAN lines 2769–2771: Check the Sign in with ChatGPT docs to see whether one `ext_agent_host_id` can register more than one user. If it cannot, generate one host ID per account. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.4 Fallback providers: DeepSeek and Gemini

- [ ] `B-a93c97bc157b` · PLAN lines 2790–2793: The Providers page takes an API key for DeepSeek, Gemini, or both. Keys go into Credential Manager under `Harness/apikey/<provider>`. They never go into SQLite or React state (section 14.1). React receives only "key set" and the last four characters. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8cf03ce5689b` · PLAN lines 2794–2796: Both services offer an OpenAI-compatible Chat Completions endpoint. One `OpenAiCompatProvider { base_url, model, credential_ref }` behind the existing `AiProvider` trait covers both: Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-231625a96a22` · PLAN lines 2797–2797: DeepSeek: `https://api.deepseek.com`; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-28aace849001` · PLAN lines 2798–2798: Gemini: `https://generativelanguage.googleapis.com/v1beta/openai/`. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-e9bf48592982` · PLAN lines 2799–2800: Fetch each provider's model list from `GET /models` and let the user pick. Do not hardcode model names. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-d35481e84bd0` · PLAN lines 2801–2805: The fallback order is a list the user can reorder. The default is the active ChatGPT account, then Gemini, then DeepSeek. Any provider can be first. If Sign in with ChatGPT does not work for a private app, an API-key provider becomes primary and the product still works. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-7067fbecf30c` · PLAN lines 2806–2810: Harness moves to the next provider only when a request fails before the first token: a usage limit, HTTP 429, HTTP 5xx, an auth error, or no first token within 10 seconds. After tokens have streamed, Harness never switches silently. It keeps the partial answer and offers "Retry with <next provider>". Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-af773a48348d` · PLAN lines 2811–2813: Screenshot images go only to providers that accept image input. Gemini does. Check DeepSeek's current API docs. If DeepSeek is still text-only, Harness sends it the OCR text instead. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3020fc50959b` · PLAN lines 2814–2815: Each answer shows a provider chip, such as `ChatGPT · work` or `Gemini`. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-61b85c95f1cf` · PLAN lines 2816–2819: The context compiler stays provider-neutral. Each provider maps the compiled message list to its wire format (Responses API for ChatGPT, Chat Completions for the others) and reports its context limit to the budgeter (section 13.2). Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-b0509362e7e5` · PLAN lines 2820–2823: A fallback sends the same compiled context to another company with its own retention policy. DeepSeek's privacy policy says it stores data on servers in China. Context profiles (38.6) can disable any provider. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.6 Context center

- [ ] `B-2c53824008ba` · PLAN lines 2875–2875: a custom instruction the user writes; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-5e12717a9dab` · PLAN lines 2876–2878: pinned items, such as a resume, a job description, project notes, or any memory or document, sent on every request within a pinned token budget; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-32382c277420` · PLAN lines 2879–2880: a retrieval scope: which projects and documents retrieval may search; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-f1f17c512261` · PLAN lines 2881–2881: the providers allowed for this profile. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.7 Screenshot to model

- [ ] `B-021630bd43e5` · PLAN lines 2925–2925: `Ctrl+Shift+O` captures the screen and sends the image; Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-3dd77a980f7d` · PLAN lines 2926–2927: `Ctrl+Shift+X` captures the screen, runs local OCR, and sends the text. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Capture

- [ ] `B-e317becc50fe` · PLAN lines 2937–2939: Capture the monitor under the mouse cursor with the `xcap` crate (`Monitor::from_point`, then `capture_image`). Capture runs in Rust, and pixels never pass through React. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-a80f94ab55e9` · PLAN lines 2940–2943: Capture exclusion (38.2) keeps the overlay out of its own screenshot, so Harness does not hide it first. If the user turned exclusion off, Harness hides the overlay, waits one frame (about 50 ms), captures, and shows the overlay again. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## Image path (`Ctrl+Shift+O`)

- [ ] `B-784be1efda48` · PLAN lines 2947–2948: Downscale so the long edge is at most 2048 px, then encode as PNG. PNG keeps small text sharp. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-834c0a9a6afc` · PLAN lines 2949–2951: For the Responses API, send an `input_image` part with a base64 data URL. For Gemini's OpenAI-compatible endpoint, send an `image_url` part with a data URL. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-4385f7be4cec` · PLAN lines 2952–2954: If the active provider has no image input, use the next image-capable provider in the chain. If none exists, run the OCR path and tell the user. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## OCR path (`Ctrl+Shift+X`)

- [ ] `B-62c0f754db16` · PLAN lines 2958–2963: Use the OCR engine built into Windows, `Windows.Media.Ocr`, through the `windows` crate (`Media_Ocr` and `Graphics_Imaging` features). It runs locally, costs nothing, and needs no bundled model. Call `OcrEngine::TryCreateFromUserProfileLanguages()`, build a BGRA8 `SoftwareBitmap` from the capture, call `RecognizeAsync`, and read `Lines` and their `Words` with bounding boxes. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6d75eac14db0` · PLAN lines 2964–2967: Windows OCR drops indentation. For code, rebuild it from each line's left x-coordinate divided by the average character width. Python stays readable that way. Wrap detected code in a fenced block. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c249d74a6bd2` · PLAN lines 2968–2969: Check `OcrEngine::MaxImageDimension`. If a capture exceeds it, downscale or tile. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-1ae73a3fb2bb` · PLAN lines 2970–2971: Show the OCR text in the message so the user sees exactly what Harness sent. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c0ec833e1509` · PLAN lines 2972–2974: If Windows OCR fails on real screens in a measured test, swap in PaddleOCR through ONNX Runtime behind the same function. Do not bundle it before then. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.

## 38.8 Markdown and code in answers

- [ ] `B-42be50756ae8` · PLAN lines 2990–2991: The system prompt tells every provider to answer in GitHub-flavored Markdown and to tag every fenced code block with its language. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-c43d5a6569c5` · PLAN lines 2992–2999: Render answers with `streamdown`, Vercel's replacement for `react-markdown` built for streaming model output. Plain `react-markdown` re-parses the whole answer on every token and shows broken output while a code fence or bold span is still open. `streamdown` handles unterminated blocks and memoizes finished ones. Check its bundle size against the overlay show-time target (section 19.3). The fallback is `react-markdown` with `remark-gfm`, `rehype-highlight`, and per-block memoization. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-cc5ddc44915c` · PLAN lines 3000–3001: Highlight code with Shiki, using a theme built from the dark design tokens. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-8ee9bb98877e` · PLAN lines 3002–3003: Each code block has a language label and a copy button, and scrolls horizontally instead of wrapping. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-047a96fd7770` · PLAN lines 3004–3004: GFM gives tables, task lists, and strikethrough. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
- [ ] `B-6e1b9780f823` · PLAN lines 3005–3008: Model output is untrusted (section 22.1). Disable raw HTML. Open links in the default browser through the Tauri opener plugin, and never navigate the webview. The CSP blocks remote images, so a Markdown image renders as a link. Evidence: pending individual verification; see REQUIREMENTS.md for the section assessment.
