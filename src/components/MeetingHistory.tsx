import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import type { SavedMeeting, TranscriptCursor, TranscriptLine } from '../bridge';
import { desktop, savedMeetings, savedTranscript, searchTranscript } from '../bridge';
import { Button, Icon, IconButton } from './primitives';
import '../styles/history.css';

const MAX_QUERY_BYTES = 512;
const MAX_QUERY_TERMS = 16;

function formatDate(value: number) {
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value));
}

function formatTime(startMs: number) {
  const seconds = Math.max(0, Math.floor(startMs / 1000));
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

function errorText(error: unknown) { return error instanceof Error ? error.message : String(error); }

export default function MeetingHistory({ onOpenAssistant }: { onOpenAssistant: (id: number) => Promise<void> }) {
  const [hasMore, setHasMore] = useState(false);
  const [meetings, setMeetings] = useState<SavedMeeting[]>([]);
  const [selected, setSelected] = useState<SavedMeeting | null>(null);
  const [lines, setLines] = useState<TranscriptLine[]>([]);
  const [next, setNext] = useState<TranscriptCursor | null>(null);
  const [query, setQuery] = useState('');
  const [searching, setSearching] = useState(false);
  const [searchedQuery, setSearchedQuery] = useState('');
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);
  const generation = useRef(0);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    const load = async () => {
      const request = ++generation.current;
      setBusy(true); setError(null);
      try {
        const result = await savedMeetings();
        if (mounted.current && request === generation.current) { setMeetings(result.meetings); setHasMore(result.hasMore); }
      } catch (cause) {
        if (mounted.current && request === generation.current) setError(errorText(cause));
      } finally {
        if (mounted.current && request === generation.current) setBusy(false);
      }
    };
    void load();
    return () => { mounted.current = false; generation.current += 1; };
  }, []);

  async function refresh() {
    const request = ++generation.current;
    setBusy(true); setError(null);
    try {
      const result = await savedMeetings();
      if (mounted.current && request === generation.current) { setMeetings(result.meetings); setHasMore(result.hasMore); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) setBusy(false);
    }
  }

  async function loadOlder() {
    const last = meetings.at(-1);
    if (!last) return;
    const request = ++generation.current;
    setBusy(true); setError(null);
    try {
      const result = await savedMeetings(last.id);
      if (mounted.current && request === generation.current) { setMeetings(result.meetings); setHasMore(result.hasMore); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) setBusy(false);
    }
  }

  async function selectMeeting(meeting: SavedMeeting) {
    const request = ++generation.current;
    setSelected(meeting); setLines([]); setNext(null); setQuery(''); setSearchedQuery(''); setTranscriptError(null); setBusy(true);
    try {
      const page = await savedTranscript(meeting.id);
      if (mounted.current && request === generation.current) { setLines(page.segments); setNext(page.next); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setTranscriptError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) setBusy(false);
    }
  }

  async function loadNext() {
    if (!selected || !next) return;
    const request = ++generation.current;
    setBusy(true); setTranscriptError(null);
    try {
      const page = await savedTranscript(selected.id, next);
      if (mounted.current && request === generation.current) { setLines(page.segments); setNext(page.next); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setTranscriptError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) setBusy(false);
    }
  }

  async function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected || busy) return;
    const text = query.trim();
    const bytes = new TextEncoder().encode(text).byteLength;
    const terms = text ? text.split(/\s+/).length : 0;
    if (bytes > MAX_QUERY_BYTES || terms > MAX_QUERY_TERMS) {
      setTranscriptError(`Search is limited to ${MAX_QUERY_BYTES} UTF-8 bytes and ${MAX_QUERY_TERMS} terms.`);
      return;
    }
    if (!text) { await clearSearch(); return; }
    const request = ++generation.current;
    setSearching(true); setSearchedQuery(''); setBusy(true); setTranscriptError(null);
    try {
      const result = await searchTranscript(selected.id, text);
      if (mounted.current && request === generation.current) { setLines(result); setNext(null); setSearchedQuery(text); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setTranscriptError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) { setSearching(false); setBusy(false); }
    }
  }

  async function clearSearch() {
    if (!selected || busy) return;
    const request = ++generation.current;
    setQuery(''); setSearchedQuery(''); setBusy(true); setSearching(false); setTranscriptError(null);
    try {
      const page = await savedTranscript(selected.id);
      if (mounted.current && request === generation.current) { setLines(page.segments); setNext(page.next); }
    } catch (cause) {
      if (mounted.current && request === generation.current) setTranscriptError(errorText(cause));
    } finally {
      if (mounted.current && request === generation.current) setBusy(false);
    }
  }

  async function openAssistant() {
    if (!selected || busy) return;
    setBusy(true); setTranscriptError(null);
    try { await onOpenAssistant(selected.id); }
    catch (cause) { if (mounted.current) setTranscriptError(errorText(cause)); }
    finally { if (mounted.current) setBusy(false); }
  }

  return <section className="history" aria-labelledby="history-title" aria-busy={busy}>
    <header className="history-heading">
      <div><h1 id="history-title">Saved meetings</h1><p>{desktop ? 'Browse locally saved conversations.' : 'Saved meetings appear here in the desktop app.'}</p></div>
      <IconButton icon="refresh" label="Refresh saved meetings" onClick={() => void refresh()} disabled={busy} />
    </header>

    {error && <p className="history-message history-error" role="alert">Could not load saved meetings. {error}</p>}
    {busy && !selected && <p className="history-message" role="status">Loading saved meetings…</p>}
    {!busy && !error && !meetings.length && <p className="history-message">{desktop ? 'No saved meetings yet.' : 'No saved meetings in the browser preview.'}</p>}
    {!!meetings.length && <div className="meeting-list" aria-label="Saved meetings">
      {meetings.map(meeting => <button className={`meeting-row${selected?.id === meeting.id ? ' selected' : ''}`} type="button" key={meeting.id} onClick={() => void selectMeeting(meeting)} disabled={busy} aria-pressed={selected?.id === meeting.id}>
        <span className="meeting-row-main"><strong>{meeting.title || 'Untitled meeting'}</strong><span>{formatDate(meeting.startedAt)}</span></span>
        <span className={`meeting-status ${meeting.status}`}>{meeting.status}</span><Icon name="chevron" />
      </button>)}
    </div>}
    {hasMore && <button className="history-text-button" type="button" onClick={() => void loadOlder()} disabled={busy}>Older meetings <span aria-hidden="true">↓</span></button>}

    {selected && <div className="history-detail">
      <div className="history-detail-heading"><div><p className="history-kicker">Selected meeting</p><h2>{selected.title || 'Untitled meeting'}</h2><p className="history-meta">{formatDate(selected.startedAt)} · {selected.status}</p></div><button className="history-text-button" type="button" onClick={() => void refresh()} disabled={busy}>Refresh list</button></div>
      <Button quiet onClick={() => void openAssistant()} disabled={busy || !desktop}>Open in assistant</Button>
      <form className="history-search" onSubmit={submitSearch}>
        <Icon name="search" /><input value={query} onChange={event => setQuery(event.target.value)} maxLength={512} disabled={busy} aria-label="Search selected meeting transcript" placeholder="Search this transcript" /><button className="history-search-submit" type="submit" disabled={busy || !query.trim()}>Search</button>{query && <button className="history-clear" type="button" onClick={() => void clearSearch()} disabled={busy}>Clear</button>}
      </form>
      {transcriptError && <p className="history-message history-error" role="alert">{transcriptError}</p>}
      {busy && <p className="history-message" role="status">{searching ? 'Searching transcript…' : 'Loading transcript…'}</p>}
      {!busy && !transcriptError && searchedQuery && <p className="history-search-hint">Up to 100 matches; refine your search.</p>}
      {!busy && !transcriptError && !lines.length && <p className="history-message">{searchedQuery ? 'No matching lines.' : 'No transcript lines saved for this meeting.'}</p>}
      {!!lines.length && <div className="history-transcript" aria-label="Transcript lines">{lines.map(line => <div className="history-line" key={line.id}><span className="history-source">{line.source === 'microphone' ? 'MIC' : 'SYSTEM'}</span><span className="history-time">{formatTime(line.startMs)}</span><span>{line.text}</span></div>)}</div>}
      {next && <button className="history-text-button" type="button" onClick={() => void loadNext()} disabled={busy}>Next lines <span aria-hidden="true">↓</span></button>}
    </div>}
  </section>;
}
