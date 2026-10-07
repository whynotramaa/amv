import { listen } from '@tauri-apps/api/event';
import { lazy, Suspense, useEffect, useRef, useState, type FormEvent, type KeyboardEvent, type MouseEvent } from 'react';
import { askMeeting, cancelResponse, desktop, defaults, getMeetingState, getResponseState, restoreSavedMeeting, openMemory, openDocuments, hideOverlay, loadState, sendMeetingSpeech, saveSettings, startChat, startDragging, startMeeting, stopMeeting, type AppState, type MeetingState, type ResponseState, type Settings } from './bridge';
import { Button, Icon, IconButton } from './components/primitives';
import Connections from './components/Connections';
import MeetingHistory from './components/MeetingHistory';
const ResponseView = lazy(() => import('./components/ResponseView'));
const UsageView = lazy(() => import('./components/UsageView'));
import './styles/app.css';

const idleMeeting: MeetingState = { status: 'idle', meetingId: null, title: '', startedAt: null, transcript: [], error: null };
const opacityKey = 'harness-background-opacity';
const clampOpacity = (value: number) => Math.min(1, Math.max(0.15, Math.round(value * 100) / 100));
function savedOpacity() {
  try { const value = Number(localStorage.getItem(opacityKey)); return value ? clampOpacity(value) : 0.85; } catch { return 0.85; }
}

export default function App() {
  const [state, setState] = useState<AppState>({ settings: defaults, audioAvailable: false, chatgptConnected: false, inferenceAvailable: false, shortcutError: null });
  const [view, setView] = useState<'assistant' | 'settings' | 'connections' | 'history' | 'usage'>('assistant');
  const [draft, setDraft] = useState(defaults);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<{ text: string; error?: boolean } | null>(null);
  const [meeting, setMeeting] = useState<MeetingState>(idleMeeting);
  const [meetingBusy, setMeetingBusy] = useState(false);
  const [question, setQuestion] = useState('');
  const [meetingSetup, setMeetingSetup] = useState(false);
  const [meetingTitle, setMeetingTitle] = useState('');
  const [captureConsent, setCaptureConsent] = useState(false);
  const [prefs, setPrefs] = useState(defaults);
  const [response, setResponse] = useState<ResponseState | null>(null);
  const [sentThrough, setSentThrough] = useState(0);
  const [opacity, setOpacity] = useState(savedOpacity);
  const meetingVersion = useRef(0);
  const responseGeneration = useRef(0);
  const responseRequestId = useRef(0);
  const meetingRef = useRef(meeting);
  const responseRef = useRef<ResponseState | null>(null);
  const commandPending = useRef(false);
  const [commandBusy, setCommandBusy] = useState(false);

  const responseBusy = response?.status === 'preparing' || response?.status === 'streaming';
  const listening = meeting.status === 'active' || meeting.status === 'starting' || meeting.status === 'stopping';
  const retainedMeetingId = Number.isInteger(meeting.meetingId) && meeting.meetingId != null ? meeting.meetingId : null;
  const unsent = meeting.transcript.filter(line => line.id > sentThrough);
  const canAsk = desktop && state.inferenceAvailable && !responseBusy && !commandBusy;
  const commitMeeting = (next: MeetingState) => {
    if (next.meetingId !== meetingRef.current.meetingId) { setQuestion(''); setSentThrough(0); responseGeneration.current++; responseRequestId.current = 0; responseRef.current = null; setResponse(null); }
    meetingRef.current = next;
    setMeeting(next);
  };
  const acceptResponse = (next: ResponseState | null, generation: number, meetingId: number) => {
    if (!next || generation !== responseGeneration.current || meetingRef.current.meetingId !== meetingId || next.meetingId !== meetingId || next.requestId < responseRequestId.current) return;
    const previous = responseRef.current;
    if (previous?.requestId === next.requestId) {
      const before = previous.attemptId ?? 0, after = next.attemptId ?? 0;
      if (after < before || (after === before && (next.answer.length < previous.answer.length ||
        (!['preparing', 'streaming'].includes(previous.status) && ['preparing', 'streaming'].includes(next.status))))) return;
    }
    if (next.requestId !== responseRequestId.current) setSentThrough(Math.max(0, ...meetingRef.current.transcript.map(line => line.id)));
    responseRequestId.current = next.requestId;
    responseRef.current = next;
    setResponse(next);
  };

  useEffect(() => {
    let cancelled = false;
    loadState().then(next => {
      if (!cancelled) { setState(next); setDraft(next.settings); if (next.shortcutError) setNotice({ text: next.shortcutError, error: true }); }
    }).catch(error => { if (!cancelled) setNotice({ text: `Couldn't load settings. ${String(error)}`, error: true }); })
      .finally(() => { if (!cancelled) setLoading(false); });
    const listeners = desktop ? [
      listen<string>('app-notice', event => { if (!cancelled) setNotice({ text: event.payload }); }),
      listen('connections-changed', () => { if (!cancelled) void loadState().then(setState).catch(() => {}); }),
      listen<MeetingState>('meeting-state', event => { if (cancelled) return; meetingVersion.current++; commitMeeting(event.payload); if (event.payload.status === 'error') setNotice({ text: event.payload.error || 'Audio capture failed.', error: true }); }),
      listen<ResponseState>('response-state', event => { if (cancelled) return; acceptResponse(event.payload, responseGeneration.current, event.payload.meetingId); }),
    ] : [];
    void Promise.all(listeners).then(async () => {
      const version = meetingVersion.current;
      const next = await getMeetingState();
      if (!cancelled && version === meetingVersion.current) commitMeeting(next);
      if (!cancelled) { const savedResponse = await getResponseState(); if (savedResponse) acceptResponse(savedResponse, responseGeneration.current, savedResponse.meetingId); }
    }).catch(error => { if (!cancelled) setNotice({ text: `Status unavailable. ${String(error)}`, error: true }); });
    const keys = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape') void hideOverlay().catch(error => setNotice({ text: String(error), error: true }));
      if (event.ctrlKey && event.altKey && (event.key === '[' || event.key === ']')) { event.preventDefault(); setOpacity(old => clampOpacity(old + (event.key === ']' ? 0.05 : -0.05))); }
      if (event.ctrlKey && event.altKey && event.key === '0') { event.preventDefault(); setOpacity(0.85); }
    };
    document.addEventListener('keydown', keys);
    return () => { cancelled = true; document.removeEventListener('keydown', keys); for (const pending of listeners) void pending.then(unlisten => unlisten()).catch(() => {}); };
  }, []);

  useEffect(() => {
    document.documentElement.style.setProperty('--background-opacity', String(opacity));
    try { localStorage.setItem(opacityKey, String(opacity)); } catch { return; }
  }, [opacity]);

  useEffect(() => {
    if (!desktop) return;
    const pending = listen('open-settings', () => { setDraft(state.settings); setView('settings'); setNotice(null); });
    return () => { void pending.then(unlisten => unlisten()).catch(() => {}); };
  }, [state.settings]);

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => { setDraft(old => ({ ...old, [key]: value })); setNotice(null); };
  function openSettings() { setDraft(state.settings); setNotice(null); setView('settings'); }
  function drag(event: MouseEvent<HTMLElement>) {
    if (event.button === 0 && !(event.target as HTMLElement).closest('button, select, input, textarea, a')) void startDragging().catch(() => {});
  }
  async function toggleListening() {
    if (!listening) { setCaptureConsent(false); setPrefs(state.settings); setMeetingSetup(true); return; }
    setMeetingBusy(true); setNotice(null);
    const version = meetingVersion.current;
    try {
      const next = await stopMeeting();
      if (version === meetingVersion.current) commitMeeting(next);
    } catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setMeetingBusy(false); }
  }
  async function beginMeeting(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!captureConsent || !meetingTitle.trim() || meetingBusy) return;
    if (prefs.responseMode === 'custom' && !prefs.customInstruction.trim()) { setNotice({ text: 'Write an instruction for custom responses.', error: true }); return; }
    setMeetingBusy(true); setNotice(null);
    const version = meetingVersion.current;
    try {
      const saved = await saveSettings(prefs);
      setState(old => ({ ...old, settings: saved }));
      const next = await startMeeting(meetingTitle.trim(), captureConsent, null, saved.sendMode === 'automatic');
      if (version === meetingVersion.current) commitMeeting(next);
      setMeetingSetup(false); setCaptureConsent(false);
    } catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setMeetingBusy(false); }
  }
  async function openSavedMeeting(id: number) {
    const version = meetingVersion.current;
    const next = await restoreSavedMeeting(id);
    if (version === meetingVersion.current) commitMeeting(next);
    setView('assistant'); setNotice(null);
  }
  async function askQuestion() {
    if (!canAsk || commandPending.current) return;
    const text = question.trim();
    if (!text) return;
    if (new TextEncoder().encode(text).length > 64 * 1024) { setNotice({ text: 'Messages must be 64 KiB or smaller.', error: true }); return; }
    commandPending.current = true; setCommandBusy(true);
    setNotice(null); setQuestion('');
    let generation = responseGeneration.current;
    let meetingId = retainedMeetingId;
    const version = meetingVersion.current;
    try {
      if (meetingId === null) {
        const next = await startChat();
        if (version !== meetingVersion.current && next.meetingId !== meetingRef.current.meetingId) return;
        commitMeeting(next); meetingId = next.meetingId; generation = responseGeneration.current;
      }
      if (meetingId === null) throw new Error('Chat could not start.');
      acceptResponse(await askMeeting(meetingId, text, false), generation, meetingId);
    } catch (error) { if (generation === responseGeneration.current && meetingId === meetingRef.current.meetingId) { setQuestion(old => old || text); setNotice({ text: String(error), error: true }); } }
    finally { commandPending.current = false; setCommandBusy(false); }
  }
  async function sendSpeech() {
    if (!desktop || retainedMeetingId === null || !state.inferenceAvailable || responseBusy || commandPending.current) return;
    const generation = responseGeneration.current;
    commandPending.current = true; setCommandBusy(true); setNotice(null);
    try { const next = await sendMeetingSpeech(retainedMeetingId); acceptResponse(next, generation, retainedMeetingId); if (!next && generation === responseGeneration.current) setNotice({ text: 'No new speech to send.' }); }
    catch (error) { if (generation === responseGeneration.current) setNotice({ text: String(error), error: true }); }
    finally { commandPending.current = false; setCommandBusy(false); }
  }
  function onQuestionKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) { if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); void (question.trim() || retainedMeetingId === null ? askQuestion() : sendSpeech()); } }
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (new TextEncoder().encode(draft.customInstruction).length > 8192) { setNotice({ text: 'Custom instructions must be 8,192 UTF-8 bytes or smaller.', error: true }); return; }
    if (draft.responseMode === 'custom' && !draft.customInstruction.trim()) { setNotice({ text: 'Write an instruction for custom responses.', error: true }); return; }
    setSaving(true); setNotice(null);
    try {
      const saved = await saveSettings(draft);
      setState(old => ({ ...old, settings: saved, shortcutError: null }));
      setDraft(saved); setView('assistant');
    } catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setSaving(false); }
  }

  return <main className={desktop ? '' : 'preview-environment'}>
    <section className="assistant" aria-label="Harness assistant" aria-busy={loading}>
      <header className="header" onMouseDown={drag}>
        <span className={`status-dot ${meeting.status}`} title={listening ? 'Listening' : 'Not listening'} />
        <div className="header-actions">
          {view === 'assistant' ? <IconButton icon="settings" label="Settings" onClick={openSettings} disabled={loading} /> : <IconButton icon="back" label="Back" onClick={() => { setView('assistant'); setNotice(null); }} disabled={saving} />}
          {desktop && <IconButton icon="close" label="Hide Harness" onClick={() => void hideOverlay().catch(error => setNotice({ text: String(error), error: true }))} />}
        </div>
      </header>

      <div className="view">
        {view === 'usage' ? <Suspense fallback={<p className="help" role="status">Loading local usage…</p>}><UsageView /></Suspense> : view === 'history' ? <MeetingHistory onOpenAssistant={openSavedMeeting} /> : view === 'connections' ? <Connections onChanged={() => void loadState().then(setState).catch(() => {})} /> : view === 'assistant' ? <>
          {meetingSetup && <form className="settings-form" onSubmit={beginMeeting} aria-label="Prepare a meeting">
            <div className="field"><label htmlFor="meeting-title">Meeting title</label><input id="meeting-title" className="input" value={meetingTitle} maxLength={200} onChange={event => setMeetingTitle(event.target.value)} disabled={meetingBusy} required /></div>
            <label className="checkbox"><input type="checkbox" checked={captureConsent} onChange={event => setCaptureConsent(event.target.checked)} disabled={meetingBusy} />I have permission to capture audio for this meeting.</label>
            <div className="field">
              <span className="field-label" id="setup-mode-label">Output</span>
              <div className="choice-group" role="radiogroup" aria-labelledby="setup-mode-label">
                <label className="choice"><input type="radio" name="setupMode" checked={prefs.responseMode === 'suggested_answers'} onChange={() => setPrefs(old => ({ ...old, responseMode: 'suggested_answers' }))} disabled={meetingBusy} />Suggested answers</label>
                <label className="choice"><input type="radio" name="setupMode" checked={prefs.responseMode === 'summary'} onChange={() => setPrefs(old => ({ ...old, responseMode: 'summary' }))} disabled={meetingBusy} />Summary</label>
                <label className="choice"><input type="radio" name="setupMode" checked={prefs.responseMode === 'custom'} onChange={() => setPrefs(old => ({ ...old, responseMode: 'custom' }))} disabled={meetingBusy} />Custom</label>
              </div>
            </div>
            {prefs.responseMode === 'custom' && <div className="field"><label htmlFor="setup-instruction">Custom instruction</label><textarea id="setup-instruction" className="input" maxLength={4000} value={prefs.customInstruction} onChange={event => setPrefs(old => ({ ...old, customInstruction: event.target.value }))} disabled={meetingBusy} placeholder="For each new message, identify decisions and open questions." /></div>}
            <div className="field">
              <span className="field-label" id="setup-send-label">Send speech</span>
              <div className="choice-group" role="radiogroup" aria-labelledby="setup-send-label">
                <label className="choice"><input type="radio" name="setupSend" checked={prefs.sendMode === 'on_hotkey'} onChange={() => setPrefs(old => ({ ...old, sendMode: 'on_hotkey' }))} disabled={meetingBusy} />On hotkey ({prefs.sendShortcut})</label>
                <label className="choice"><input type="radio" name="setupSend" checked={prefs.sendMode === 'automatic'} onChange={() => setPrefs(old => ({ ...old, sendMode: 'automatic' }))} disabled={meetingBusy} />After a pause</label>
              </div>
            </div>
            {prefs.sendMode === 'automatic' && <div className="field"><label htmlFor="setup-delay">Send after {prefs.autoSendDelayMs / 1000}s of silence</label><input id="setup-delay" type="range" min={500} max={10000} step={500} value={prefs.autoSendDelayMs} onChange={event => setPrefs(old => ({ ...old, autoSendDelayMs: Number(event.target.value) }))} disabled={meetingBusy} /></div>}
            <label className="checkbox"><input type="checkbox" checked={prefs.includeMicrophone} onChange={event => setPrefs(old => ({ ...old, includeMicrophone: event.target.checked }))} disabled={meetingBusy} />Include my microphone as context</label>
            {notice && <p className={`notice${notice.error ? ' error' : ''}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
            <div className="form-actions"><Button quiet disabled={meetingBusy} onClick={() => setMeetingSetup(false)}>Cancel meeting setup</Button><Button primary type="submit" disabled={meetingBusy || !captureConsent || !meetingTitle.trim()}>Start local meeting</Button></div>
          </form>}
          {listening && unsent.length > 0 && <div className="pending-speech" aria-label="Unsent speech">{unsent.map(line => <p key={line.id}>{line.text}</p>)}</div>}
          {response && <Suspense fallback={null}><ResponseView response={response} onRetry={response.status === 'error' || response.status === 'cancelled' ? sendSpeech : undefined} /></Suspense>}
          {!meetingSetup && notice && <p className={`notice${notice.error ? ' error' : ''}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
        </> : <form className="settings-form" onSubmit={submit}>
          <div className="settings-links">
            <Button quiet onClick={() => { setView('connections'); setNotice(null); }}>{state.inferenceAvailable ? 'ChatGPT connected' : state.chatgptConnected ? 'Pick a model' : 'Connect ChatGPT'}</Button>
            <Button quiet onClick={() => { setView('usage'); setNotice(null); }}>Usage</Button>
            <Button quiet onClick={() => { setView('history'); setNotice(null); }}>Saved meetings</Button>
            <Button quiet onClick={() => void openDocuments().catch(error => setNotice({ text: String(error), error: true }))}>Documents</Button>
            <Button quiet onClick={() => void openMemory().catch(error => setNotice({ text: String(error), error: true }))}>Memory</Button>
          </div>
          <div className="field"><label htmlFor="opacity">Background opacity {Math.round(opacity * 100)}%</label><input id="opacity" type="range" min={15} max={100} value={Math.round(opacity * 100)} onChange={event => setOpacity(clampOpacity(Number(event.target.value) / 100))} /><small>Ctrl+Alt+[ and Ctrl+Alt+] adjust it. Ctrl+Alt+0 resets it.</small></div>
          <div className="shortcut-fields">
            <div className="field"><label htmlFor="overlay-shortcut">Show or hide</label><input id="overlay-shortcut" className="input" maxLength={100} value={draft.overlayShortcut} onChange={event => update('overlayShortcut', event.target.value)} required spellCheck={false} /></div>
            <div className="field"><label htmlFor="send-shortcut">Send speech</label><input id="send-shortcut" className="input" maxLength={100} value={draft.sendShortcut} onChange={event => update('sendShortcut', event.target.value)} required spellCheck={false} /></div>
            <div className="field"><label htmlFor="new-chat-shortcut">New chat</label><input id="new-chat-shortcut" className="input" maxLength={100} value={draft.newChatShortcut} onChange={event => update('newChatShortcut', event.target.value)} required spellCheck={false} /></div>
          </div>
          <label className="checkbox"><input type="checkbox" checked={draft.launchOnLogin} onChange={event => update('launchOnLogin', event.target.checked)} />Launch when I sign in to Windows</label>
          {notice && <p className={`notice${notice.error ? ' error' : ''}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
          <div className="form-actions"><Button primary type="submit" disabled={saving || loading}>{saving ? 'Saving…' : 'Save'}</Button></div>
        </form>}
      </div>

      {view === 'assistant' && !meetingSetup && <div className="composer">
        <div className="composer-input">
          <textarea aria-label="Message" placeholder={!desktop ? 'Available in the Windows app' : state.inferenceAvailable ? (retainedMeetingId !== null ? `Ask anything, or Enter / ${state.settings.sendShortcut} to send speech` : 'Ask anything') : state.chatgptConnected ? 'Pick a ChatGPT model in settings first' : 'Connect ChatGPT in settings first'} value={question} onChange={event => setQuestion(event.target.value)} onKeyDown={onQuestionKeyDown} disabled={!canAsk} rows={1} />
          <IconButton icon="arrow" label="Send" onClick={() => void askQuestion()} disabled={!canAsk || !question.trim()} />
        </div>
        <div className="composer-bar">
          <div className="composer-actions">
            {responseBusy && <Button quiet onClick={() => void cancelResponse().catch(error => setNotice({ text: String(error), error: true }))}>Stop</Button>}
            <button type="button" className={`icon-button mic${listening ? ' on' : ''}`} aria-label={listening ? 'Stop listening' : 'Start listening'} aria-pressed={listening} title={listening ? 'Stop listening' : 'Start listening'} onClick={() => void toggleListening()} disabled={!desktop || meetingBusy || meeting.status === 'starting' || meeting.status === 'stopping'}>
              <Icon name="mic" />
            </button>
          </div>
        </div>
      </div>}
    </section>
  </main>;
}
