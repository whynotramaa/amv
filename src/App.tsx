import { listen } from '@tauri-apps/api/event';
import { lazy, Suspense, useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from 'react';
import { askMeeting, audioDevices, cancelResponse, type AudioDevice, desktop, defaults, getMeetingState, getResponseState, restoreSavedMeeting, hideOverlay, loadState, sendMeetingSpeech, saveSettings, startMeeting, stopMeeting, type AppState, type MeetingState, type ResponseState, type Settings } from './bridge';
import { Button, Icon, IconButton } from './components/primitives';
import Connections from './components/Connections';
import MeetingHistory from './components/MeetingHistory';
const ResponseView = lazy(() => import('./components/ResponseView'));
import './styles/app.css';

const responseLabels = { suggested_answers: 'Suggested answers', summary: 'Summary', custom: 'Custom instruction' };
const idleMeeting: MeetingState = { status: 'idle', meetingId: null, title: '', startedAt: null, transcript: [], error: null };
const meetingStatusLabels = { idle: 'Ready for a meeting', starting: 'Starting local capture', active: 'Meeting in progress', stopping: 'Stopping local capture', error: 'Meeting needs attention' };

export default function App() {
  const [state, setState] = useState<AppState>({ settings: defaults, audioAvailable: false, chatgptConnected: false, inferenceAvailable: false, shortcutError: null });
  const [view, setView] = useState<'assistant' | 'setup' | 'settings' | 'connections' | 'history'>('assistant');
  const [draft, setDraft] = useState(defaults);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<{ text: string; error?: boolean } | null>(null);
  const [meeting, setMeeting] = useState<MeetingState>(idleMeeting);
  const [meetingTitle, setMeetingTitle] = useState('');
  const [microphones, setMicrophones] = useState<AudioDevice[]>([]);
  const [selectedMicId, setSelectedMicId] = useState<string | null>(null);
  const [deviceLoading, setDeviceLoading] = useState(false);
  const [deviceError, setDeviceError] = useState<string | null>(null);
  const [permission, setPermission] = useState(false);
  const [remoteConsent, setRemoteConsent] = useState(false);
  const [meetingBusy, setMeetingBusy] = useState(false);
  const [question, setQuestion] = useState('');
  const [includeMicrophone, setIncludeMicrophone] = useState(false);
  const [response, setResponse] = useState<ResponseState | null>(null);
  const [transcriptOpen, setTranscriptOpen] = useState(true);
  const [elapsed, setElapsed] = useState(0);
  const meetingVersion = useRef(0);
  const responseGeneration = useRef(0);
  const responseRequestId = useRef(0);
  const meetingRef = useRef(meeting);
  const responseRef = useRef<ResponseState | null>(null);
  const commandPending = useRef(false);
  const [commandBusy, setCommandBusy] = useState(false);

  const responseBusy = response?.status === 'preparing' || response?.status === 'streaming';
  const retainedMeetingId = Number.isInteger(meeting.meetingId) && meeting.meetingId != null ? meeting.meetingId : null;
  const canAsk = desktop && state.inferenceAvailable && retainedMeetingId !== null && !responseBusy && !commandBusy;
  const commitMeeting = (next: MeetingState) => {
    if (next.meetingId !== meetingRef.current.meetingId) { responseGeneration.current++; responseRequestId.current = 0; responseRef.current = null; setResponse(null); setQuestion(''); setIncludeMicrophone(false); }
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
    if (previous?.requestId !== next.requestId) setTranscriptOpen(false);
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
      listen<MeetingState>('meeting-state', event => { if (cancelled) return; meetingVersion.current++; commitMeeting(event.payload); if (event.payload.status === 'error') setNotice({ text: event.payload.error || 'Meeting capture failed.', error: true }); }),
      listen<ResponseState>('response-state', event => { if (cancelled) return; acceptResponse(event.payload, responseGeneration.current, event.payload.meetingId); }),
    ] : [];
    void Promise.all(listeners).then(async () => {
      const version = meetingVersion.current;
      const next = await getMeetingState();
      if (!cancelled && version === meetingVersion.current) commitMeeting(next);
      if (!cancelled) { const savedResponse = await getResponseState(); if (savedResponse) acceptResponse(savedResponse, responseGeneration.current, savedResponse.meetingId); }
    }).catch(error => { if (!cancelled) setNotice({ text: `Meeting status unavailable. ${String(error)}`, error: true }); });
    const escape = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape') void hideOverlay().catch(error => setNotice({ text: String(error), error: true }));
    };
    document.addEventListener('keydown', escape);
    return () => { cancelled = true; document.removeEventListener('keydown', escape); for (const pending of listeners) void pending.then(unlisten => unlisten()).catch(() => {}); };
  }, []);

  useEffect(() => {
    let timer: number | undefined;
    const update = () => setElapsed(meeting.startedAt ? Math.max(0, Date.now() - meeting.startedAt) : 0);
    const sync = () => {
      if (timer !== undefined) window.clearInterval(timer);
      timer = undefined;
      if (meeting.status === 'active' && document.visibilityState === 'visible') { update(); timer = window.setInterval(update, 1000); }
    };
    sync();
    document.addEventListener('visibilitychange', sync);
    return () => { if (timer !== undefined) window.clearInterval(timer); document.removeEventListener('visibilitychange', sync); };
  }, [meeting.status, meeting.startedAt]);

  useEffect(() => {
    if (!desktop) return;
    const pending = listen('open-settings', () => { setDraft(state.settings); setView('settings'); setNotice(null); });
    return () => { void pending.then(unlisten => unlisten()).catch(() => {}); };
  }, [state.settings]);

  useEffect(() => {
    if (view !== 'setup' || !desktop) return;
    let cancelled = false;
    setDeviceLoading(true); setDeviceError(null);
    audioDevices().then(devices => {
      if (!cancelled) setMicrophones(devices.filter(device => device.direction === 'microphone'));
    }).catch(error => { if (!cancelled) setDeviceError(String(error)); })
      .finally(() => { if (!cancelled) setDeviceLoading(false); });
    return () => { cancelled = true; };
  }, [view]);

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => { setDraft(old => ({ ...old, [key]: value })); setNotice(null); };
  function openSettings() { setDraft(state.settings); setNotice(null); setView('settings'); }
  function openSetup() { setMicrophones([]); setSelectedMicId(null); setDeviceError(null); setMeetingTitle(''); setPermission(false); setRemoteConsent(false); setNotice(null); setView('setup'); }
  async function beginMeeting(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const title = meetingTitle.trim();
    if (!title || title.length > 200) { setNotice({ text: 'Add a meeting title up to 200 characters.', error: true }); return; }
    if (!permission) { setNotice({ text: 'Confirm permission to capture audio for this meeting.', error: true }); return; }
    setMeetingBusy(true); setNotice(null);
    const version = meetingVersion.current;
    try { const next = await startMeeting(title, permission, selectedMicId, state.settings.sendMode === 'automatic' ? remoteConsent : false); if (version === meetingVersion.current) commitMeeting(next); setView('assistant'); }
    catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setMeetingBusy(false); }
  }
  async function openSavedMeeting(id: number) {
    const version = meetingVersion.current;
    const next = await restoreSavedMeeting(id);
    if (version === meetingVersion.current) commitMeeting(next);
    setView('assistant'); setNotice({ text: 'Saved meeting opened. Send new speech to retry a saved pending response. Automatic sending stays off.' });
  }
  async function endMeeting() {
    setMeetingBusy(true); setNotice(null);
    const version = meetingVersion.current;
    try { const next = await stopMeeting(); if (version === meetingVersion.current) commitMeeting(next); }
    catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setMeetingBusy(false); }
  }
  async function askQuestion() {
    if (!canAsk || retainedMeetingId === null || commandPending.current) return;
    const text = question.trim();
    if (!text) { setNotice({ text: 'Write a question first.', error: true }); return; }
    if (new TextEncoder().encode(text).length > 64 * 1024) { setNotice({ text: 'Questions must be 64 KiB or smaller.', error: true }); return; }
    const generation = responseGeneration.current;
    commandPending.current = true; setCommandBusy(true);
    setNotice(null); setQuestion('');
    try { acceptResponse(await askMeeting(retainedMeetingId, text, includeMicrophone), generation, retainedMeetingId); }
    catch (error) { if (generation === responseGeneration.current) { setQuestion(old => old || text); setNotice({ text: String(error), error: true }); } }
    finally { commandPending.current = false; setCommandBusy(false); }
  }
  async function sendSpeech() {
    if (!desktop || retainedMeetingId === null || !state.inferenceAvailable || responseBusy || commandPending.current) return;
    const generation = responseGeneration.current;
    commandPending.current = true; setCommandBusy(true); setNotice(null);
    try { const next = await sendMeetingSpeech(retainedMeetingId); acceptResponse(next, generation, retainedMeetingId); if (!next && generation === responseGeneration.current) setNotice({ text: 'No new system speech to send.' }); }
    catch (error) { if (generation === responseGeneration.current) setNotice({ text: String(error), error: true }); }
    finally { commandPending.current = false; setCommandBusy(false); }
  }
  function onQuestionKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) { if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); void askQuestion(); } }
  const meetingElapsed = `${String(Math.floor(elapsed / 60000)).padStart(2, '0')}:${String(Math.floor(elapsed / 1000) % 60).padStart(2, '0')}`;
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (new TextEncoder().encode(draft.customInstruction).length > 8192) { setNotice({ text: 'Custom instructions must be 8,192 UTF-8 bytes or smaller.', error: true }); return; }
    if (draft.responseMode === 'custom' && !draft.customInstruction.trim()) { setNotice({ text: 'Write an instruction for custom responses.', error: true }); return; }
    setSaving(true); setNotice(null);
    try {
      const saved = await saveSettings(draft);
      setState(old => ({ ...old, settings: saved, shortcutError: null }));
      setDraft(saved); setView('assistant'); setNotice({ text: 'Meeting preferences saved.' });
    } catch (error) { setNotice({ text: String(error), error: true }); }
    finally { setSaving(false); }
  }

  return <main className={desktop ? '' : 'preview-environment'}>
    <section className="assistant" aria-label="Harness meeting assistant" aria-busy={loading}>
      <header className="header" data-tauri-drag-region>
        <div className="identity" data-tauri-drag-region><Icon name="harness" /><span data-tauri-drag-region>Harness</span></div>
        <div className="header-actions">
          {view === 'assistant' && <IconButton icon="history" label="Saved meetings" onClick={() => { setView('history'); setNotice(null); }} disabled={loading} />}
          {view === 'assistant' ? <IconButton icon="settings" label="Meeting settings" onClick={openSettings} disabled={loading} /> : <IconButton icon="back" label="Back to assistant" onClick={() => { setView('assistant'); setNotice(null); }} disabled={saving || meetingBusy} />}
          {desktop && <IconButton icon="close" label="Hide Harness" onClick={() => void hideOverlay().catch(error => setNotice({ text: String(error), error: true }))} />}
        </div>
      </header>

      {view === 'history' ? <MeetingHistory onOpenAssistant={openSavedMeeting} /> : view === 'connections' ? <Connections onChanged={() => void loadState().then(setState).catch(() => {})} /> : view === 'setup' ? <>
        <div className="section-intro"><h1>Prepare a meeting</h1><p>Local transcription starts only after you give permission for this meeting.</p></div>
        <form className="settings-form meeting-setup" onSubmit={beginMeeting}>
          <div className="field"><label htmlFor="meeting-title">Meeting title</label><input id="meeting-title" className="input" maxLength={200} value={meetingTitle} onChange={event => setMeetingTitle(event.target.value)} placeholder="e.g. Acme discovery call" autoFocus required /><small>{meetingTitle.length} / 200 characters</small></div>
          <div className="field"><label htmlFor="meeting-microphone">Microphone</label><select id="meeting-microphone" className="input" aria-describedby="microphone-help" value={selectedMicId || ''} onChange={event => setSelectedMicId(event.target.value || null)} disabled={!desktop || deviceLoading || meetingBusy}>
            <option value="">Windows default microphone</option>{microphones.map(device => <option key={device.id} value={device.id}>{device.label}{device.isDefault ? ' (default)' : ''}</option>)}
          </select><small id="microphone-help">{deviceLoading ? 'Finding microphones…' : !desktop ? 'Microphone selection is available in the Windows app.' : deviceError ? 'Device discovery failed. Default selection will be checked when capture starts.' : 'System audio uses the current Windows playback device. Microphone speech stays separate.'}</small></div>
          <div className="setup-summary"><div className="summary-row"><span>Send system speech</span><span>{state.settings.sendMode === 'automatic' ? 'Automatically' : 'On hotkey'}</span></div><div className="summary-row"><span>Respond with</span><span>{responseLabels[state.settings.responseMode]}</span></div><button type="button" className="text-link" onClick={openSettings}>Change meeting settings</button></div>
          {state.settings.sendMode === 'automatic' && <div className="field remote-consent"><label className="checkbox"><input type="checkbox" checked={remoteConsent} onChange={event => setRemoteConsent(event.target.checked)} disabled={!state.inferenceAvailable} />Allow automatic sending of new system speech during this meeting</label><small>{state.inferenceAvailable ? 'Only finalized system speech is sent. Microphone speech stays local unless you include it in a question.' : 'Connect an inference provider and select a model before enabling automatic sending.'}</small></div>}
          <label className="checkbox permission-check"><input type="checkbox" checked={permission} onChange={event => setPermission(event.target.checked)} />I have permission to capture audio for this meeting.</label>
          {!desktop && <p className="notice">Browser preview: native start and stop are disabled. No audio is captured here.</p>}
          {notice && <p className="notice error" role="alert">{notice.text}</p>}
          <div className="form-actions"><Button quiet onClick={() => { setView('assistant'); setNotice(null); }} disabled={meetingBusy}>Cancel</Button><Button primary type="submit" disabled={meetingBusy || loading || deviceLoading || !desktop}>{meetingBusy ? 'Starting…' : 'Start local meeting'}</Button></div>
        </form>
      </> : view === 'assistant' ? <>
        <div className="status"><span className={`status-dot ${meeting.status}`} /><span>{loading ? 'Loading preferences' : meeting.status === 'idle' && meeting.meetingId ? 'Meeting saved locally' : meetingStatusLabels[meeting.status]}</span>{meeting.status === 'active' && <span className="meeting-clock">{meetingElapsed}</span>}</div>
        <div className="hero"><h1>{meeting.meetingId ? meeting.title : 'Sales conversations, in context.'}</h1><p>{meeting.meetingId ? 'Saved locally. Only authorized text is sent to your selected provider.' : 'Keep meeting notes close. Answer with the facts behind the conversation.'}</p></div>
        {meeting.status === 'active' || meeting.status === 'stopping' || meeting.transcript.length > 0 ? <details className="transcript" open={transcriptOpen} onToggle={event => setTranscriptOpen(event.currentTarget.open)}><summary className="transcript-heading"><span className="disclosure-label"><Icon name="chevron" />Finalized transcript</span><span>{meeting.transcript.length} lines</span></summary>{meeting.transcript.length ? meeting.transcript.map(line => <div className="transcript-line" key={line.id}><span className="source-tag">{line.source === 'microphone' ? 'MIC' : 'SYSTEM'}</span><span className="transcript-time">{Math.floor(line.startMs / 60000)}:{String(Math.floor(line.startMs / 1000) % 60).padStart(2, '0')}</span><span>{line.text}</span></div>) : <p className="help">Waiting for finalized speech.</p>}</details> : null}
        <details className="meeting-options"><summary><Icon name="chevron" />Next meeting preferences</summary>
          <div className="summary-row"><span>Send system speech</span><span>{state.settings.sendMode === 'automatic' ? 'Automatically' : 'On hotkey'}</span></div>
          <div className="summary-row"><span>Respond with</span><span>{responseLabels[state.settings.responseMode]}</span></div>
        </details>
        <div className="start-actions">{meeting.status === 'active' || meeting.status === 'starting' || meeting.status === 'stopping' ? <Button primary onClick={() => void endMeeting()} disabled={meetingBusy || meeting.status !== 'active'}>{meetingBusy || meeting.status === 'stopping' ? 'Stopping…' : 'Stop meeting'}</Button> : <Button primary onClick={openSetup} disabled={loading}>{meeting.status === 'error' ? 'Retry meeting' : 'Prepare a meeting'}</Button>}<span className="help">{desktop ? (meeting.status === 'active' ? 'Listening locally.' : state.audioAvailable ? 'Local capture is supported. Listening starts only after permission.' : 'Audio availability will be confirmed when capture starts.') : 'Browser preview · native meeting actions are unavailable.'}</span></div>
        {meeting.status === 'error' && <p className="notice error" role="alert">{meeting.error || 'Meeting capture failed.'} <button type="button" className="text-link" onClick={openSetup}>Try again</button></p>}
        {response && <Suspense fallback={<p className="help">Loading response view…</p>}><ResponseView response={response} onRetry={response.status === 'error' || response.status === 'cancelled' ? sendSpeech : undefined} /></Suspense>}
        <div className="composer">
          <div className="composer-input"><textarea aria-label="Ask about this meeting" aria-describedby="question-help" placeholder={canAsk ? 'Ask about this meeting' : 'Meeting questions are unavailable'} value={question} onChange={event => setQuestion(event.target.value)} onKeyDown={onQuestionKeyDown} disabled={!canAsk} rows={2} /><IconButton icon="arrow" label="Send question" onClick={() => void askQuestion()} disabled={!canAsk || !question.trim()} /></div>
          <div className="composer-options"><label className="checkbox"><input type="checkbox" checked={includeMicrophone} onChange={event => setIncludeMicrophone(event.target.checked)} disabled={!canAsk} />Include microphone context</label><small id="question-help">Press Enter to ask. Shift+Enter adds a new line. Microphone context is never sent automatically.</small></div>
          <div className="summary-row"><Button quiet onClick={() => void sendSpeech()} disabled={!desktop || !state.inferenceAvailable || retainedMeetingId === null || responseBusy || commandBusy}>Send new speech</Button>{responseBusy && <Button quiet onClick={() => void cancelResponse().then(() => setNotice({ text: 'Response stopped. Automatic sending is paused; send new speech to resume.' })).catch(error => setNotice({ text: String(error), error: true }))}>Cancel</Button>}<Button quiet onClick={() => { setView('connections'); setNotice(null); }} disabled={loading}>{state.inferenceAvailable ? 'Inference ready' : 'Connect and select a model'}</Button><span className="meta"><kbd>{state.settings.overlayShortcut}</kbd></span></div>
        </div>
      </> : <>
        <div className="section-intro"><h1>Meeting preferences</h1><p>Choose when Harness sends speech and how it responds.</p></div>
        <form className="settings-form" onSubmit={submit}>
          <div className="field">
            <span className="field-label" id="send-mode-label">Send system speech</span>
            <div className="choice-group" role="radiogroup" aria-labelledby="send-mode-label" aria-describedby="send-mode-help">
              <label className="choice"><input type="radio" name="sendMode" value="on_hotkey" checked={draft.sendMode === 'on_hotkey'} onChange={() => update('sendMode', 'on_hotkey')} />On hotkey</label>
              <label className="choice"><input type="radio" name="sendMode" value="automatic" checked={draft.sendMode === 'automatic'} onChange={() => update('sendMode', 'automatic')} />Automatically</label>
            </div>
            <small id="send-mode-help">{draft.sendMode === 'automatic' ? 'Once connected, new transcribed system speech will be sent automatically, using your ChatGPT allowance.' : 'Once available, speech will stay local until you press the send shortcut.'}</small>
          </div>
          <div className="field"><label htmlFor="response-mode">Respond with</label><select id="response-mode" className="input" aria-describedby="response-mode-help" value={draft.responseMode} onChange={event => update('responseMode', event.target.value as Settings['responseMode'])}>
            <option value="suggested_answers">Suggested answers</option><option value="summary">Summary</option><option value="custom">Custom instruction</option>
          </select><small id="response-mode-help">{draft.responseMode === 'suggested_answers' ? 'Concise answers and useful points for the conversation.' : draft.responseMode === 'summary' ? 'A clear summary of the new speech, with earlier messages as context.' : 'Tell Harness what to do with each new transcript message.'}</small></div>
          {draft.responseMode === 'custom' && <div className="field"><label htmlFor="custom-instruction">Your instruction</label><textarea id="custom-instruction" className="input" aria-describedby="custom-instruction-help" maxLength={4000} value={draft.customInstruction} onChange={event => update('customInstruction', event.target.value)} placeholder="For each new message, identify decisions and open questions." required /><small id="custom-instruction-help">{draft.customInstruction.length} / 4,000 characters · {new TextEncoder().encode(draft.customInstruction).length} / 8,192 UTF-8 bytes</small></div>}
          <div className="shortcut-fields">
            <div className="field"><label htmlFor="overlay-shortcut">Show or hide assistant</label><input id="overlay-shortcut" className="input" maxLength={100} value={draft.overlayShortcut} onChange={event => update('overlayShortcut', event.target.value)} required spellCheck={false} /></div>
            <div className="field"><label htmlFor="send-shortcut">Send new speech</label><input id="send-shortcut" className="input" maxLength={100} value={draft.sendShortcut} onChange={event => update('sendShortcut', event.target.value)} required spellCheck={false} /></div>
          </div>
          <details className="settings-details"><summary>Startup and privacy</summary><div>
            <label className="checkbox"><input type="checkbox" checked={draft.launchOnLogin} onChange={event => update('launchOnLogin', event.target.checked)} />Launch Harness when I sign in to Windows</label>
            <p className="help">Local transcription runs on your device. Only new finalized system speech is sent when you request it or authorize automatic sending for a meeting. Earlier messages provide bounded conversation context.</p>
          </div></details>
          {notice && <p className={`notice${notice.error ? ' error' : ''}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
          <div className="form-actions"><Button quiet onClick={() => { setView('assistant'); setNotice(null); }} disabled={saving}>Cancel</Button><Button primary type="submit" disabled={saving || loading}>{saving ? 'Saving…' : 'Save preferences'}</Button></div>
        </form>
      </>}
      {view === 'assistant' && notice && <p className={`notice${notice.error ? ' error' : ' success'}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
    </section>
    {!desktop && <p className="preview-note">Interface preview · desktop actions are unavailable</p>}
  </main>;
}
