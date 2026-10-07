import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { MemoryEntry, MemoryInput } from '../bridge';
import { closeMemory, deleteMemory, desktop, listMemories, saveMemory, searchMemories } from '../bridge';
import { Button, Icon, IconButton } from './primitives';
import '../styles/memory.css';

const categories = ['project', 'person', 'organization', 'preference', 'note', 'decision', 'experience', 'education', 'term'] as const;
const emptyDraft = (): MemoryInput => ({ title: '', body: '', category: 'note', project: null, enabled: false });
const inputFrom = (entry: MemoryEntry): MemoryInput => ({ title: entry.title, body: entry.body, category: entry.category, project: entry.project, enabled: entry.enabled });
const errorText = (error: unknown) => error instanceof Error ? error.message : String(error);
const bytes = (text: string) => new TextEncoder().encode(text).byteLength;
const date = (value: number) => new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value));

export default function MemoryView() {
  const [entries, setEntries] = useState<MemoryEntry[]>([]);
  const [next, setNext] = useState<number | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [query, setQuery] = useState('');
  const [searched, setSearched] = useState('');
  const [loading, setLoading] = useState(desktop);
  const [mutating, setMutating] = useState(false);
  const [listError, setListError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [selected, setSelected] = useState<MemoryEntry | null>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState<MemoryInput>(emptyDraft);
  const [baseline, setBaseline] = useState<MemoryInput>(emptyDraft);
  const [pendingSelection, setPendingSelection] = useState<MemoryEntry | 'new' | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmClose, setConfirmClose] = useState(false);
  const [closing, setClosing] = useState(false);
  const mounted = useRef(true);
  const generation = useRef(0);
  const mutation = useRef(false);
  const reading = useRef(false);
  const closingRef = useRef(false);
  const dirtyRef = useRef(false);
  const title = useRef<HTMLInputElement>(null);
  const dirty = JSON.stringify(draft) !== JSON.stringify(baseline);
  dirtyRef.current = dirty;
  const busy = loading || mutating || closing;

  async function load(search = '', beforeId: number | null = null) {
    if (!desktop || mutation.current || closingRef.current) return;
    const request = ++generation.current;
    reading.current = true; setLoading(true); setListError(null);
    try {
      const page = search ? { entries: await searchMemories(search), hasMore: false, next: null } : await listMemories(beforeId);
      if (mounted.current && request === generation.current) {
        setEntries(page.entries); setHasMore(page.hasMore); setNext(page.next); setSearched(search);
      }
    } catch (error) {
      if (mounted.current && request === generation.current) setListError(errorText(error));
    } finally {
      if (mounted.current && request === generation.current) { reading.current = false; setLoading(false); }
    }
  }

  useEffect(() => {
    mounted.current = true;
    void load();
    return () => { mounted.current = false; generation.current += 1; };
  }, []);

  async function close(discard = false) {
    if (closingRef.current) return;
    if (mutation.current) { setNotice('Finish the current change before closing memory.'); return; }
    if (dirtyRef.current && !discard) { setConfirmClose(true); setPendingSelection(null); setConfirmDelete(false); return; }
    closingRef.current = true; setClosing(true); setFormError(null);
    try { await closeMemory(); }
    catch (error) { if (mounted.current) setFormError(errorText(error)); }
    finally { closingRef.current = false; if (mounted.current) setClosing(false); }
  }

  useEffect(() => {
    if (!desktop) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen('memory-close-requested', () => { if (!disposed) void close(); }).then(stop => {
      if (disposed) stop(); else unlisten = stop;
    }).catch(error => { if (!disposed && mounted.current) setFormError(errorText(error)); });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  function choose(entry: MemoryEntry | 'new', discard = false) {
    if (mutation.current || closingRef.current) return;
    setConfirmClose(false);
    if (dirty && !discard) { setPendingSelection(entry); return; }
    const value = entry === 'new' ? emptyDraft() : inputFrom(entry);
    setSelected(entry === 'new' ? null : entry); setDraft(value); setBaseline(value); setEditing(true);
    setPendingSelection(null); setConfirmDelete(false); setFormError(null); setNotice(null);
    requestAnimationFrame(() => { if (mounted.current) title.current?.focus(); });
  }

  function update<K extends keyof MemoryInput>(key: K, value: MemoryInput[K]) {
    setDraft(previous => ({ ...previous, [key]: value })); setFormError(null); setNotice(null); setConfirmDelete(false);
  }

  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (reading.current || mutation.current || closingRef.current) return;
    const text = query.trim();
    if (bytes(text) > 512 || (text && text.split(/\s+/).length > 16)) {
      setListError('Search is limited to 512 UTF-8 bytes and 16 terms.'); return;
    }
    await load(text);
  }

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!desktop || mutation.current || reading.current || closingRef.current) return;
    const value: MemoryInput = { ...draft, title: draft.title.trim(), body: draft.body.trim(), project: draft.project?.trim() || null };
    if (!value.title || !value.body) { setFormError('Add a title and memory text.'); return; }
    if (Array.from(value.title).length > 200 || bytes(value.title) > 800) { setFormError('Title is limited to 200 characters and 800 UTF-8 bytes.'); return; }
    if (bytes(value.body) > 16384) { setFormError('Memory text is limited to 16 KiB of UTF-8 text.'); return; }
    if (value.project && (Array.from(value.project).length > 100 || bytes(value.project) > 400)) { setFormError('Project is limited to 100 characters and 400 UTF-8 bytes.'); return; }
    mutation.current = true; setMutating(true); setFormError(null); setNotice(null);
    try {
      const saved = await saveMemory(selected?.id ?? null, value);
      if (!mounted.current) return;
      setSelected(saved); setDraft(inputFrom(saved)); setBaseline(inputFrom(saved)); setConfirmDelete(false); setPendingSelection(null); setConfirmClose(false);
      setNotice('Saved locally.');
    } catch (error) {
      if (mounted.current) setFormError(errorText(error));
    } finally {
      mutation.current = false;
      if (mounted.current) setMutating(false);
    }
    if (mounted.current && !mutation.current) await load(searched);
  }

  async function remove() {
    if (!desktop || !selected || mutation.current || reading.current || closingRef.current) return;
    mutation.current = true; setMutating(true); setFormError(null); setNotice(null);
    try {
      await deleteMemory(selected.id);
      if (!mounted.current) return;
      setSelected(null); setDraft(emptyDraft()); setBaseline(emptyDraft()); setEditing(false); setConfirmDelete(false); setPendingSelection(null); setConfirmClose(false);
      setNotice('Memory deleted.');
      requestAnimationFrame(() => { if (mounted.current) document.getElementById('memory-add')?.focus(); });
    } catch (error) {
      if (mounted.current) setFormError(errorText(error));
    } finally {
      mutation.current = false;
      if (mounted.current) setMutating(false);
    }
    if (mounted.current) await load(searched);
  }

  return <section className="memory" aria-labelledby="memory-title" aria-busy={busy}>
    <header className="memory-heading"><div><h1 id="memory-title">Memory</h1><p>Keep useful context locally. You choose what can be used.</p></div>
      <div className="memory-actions"><IconButton icon="refresh" label="Reload memories" disabled={busy || !desktop} onClick={() => void load(searched)} /><Button id="memory-add" disabled={busy || !desktop} onClick={() => choose('new')}>Add memory</Button></div>
    </header>
    {!desktop && <p className="memory-message" role="status">Memory editing is available in the desktop app. This preview does not save memories.</p>}
    <p className="memory-help" id="memory-permission-help">Only memories you enable may be included when relevant in an authorized answer, including consented fallback providers.</p>
    <div className="memory-workspace">
      <section className="memory-library" aria-label="Saved memory">
        <form className="memory-search" onSubmit={search}><Icon name="search" /><input value={query} onChange={event => setQuery(event.target.value)} disabled={busy || !desktop} aria-label="Search memories" placeholder="Search memory" /><button type="submit" disabled={busy || !desktop}>Search</button></form>
        {searched && <div className="memory-search-summary"><span>Up to 20 matches for “{searched}”.</span><button type="button" className="text-link" disabled={busy} onClick={() => { setQuery(''); void load(); }}>Clear search</button></div>}
        {listError && <p className="memory-error" role="alert">{listError}</p>}
        {loading && <p className="memory-message" role="status">Loading memory…</p>}
        {!loading && !listError && !entries.length && <p className="memory-message">{searched ? 'No matching memories. Try a different search.' : 'No memories saved yet.'}</p>}
        <div className="memory-list">{entries.map(entry => <button key={entry.id} type="button" className={`memory-row${selected?.id === entry.id ? ' selected' : ''}`} aria-pressed={selected?.id === entry.id} disabled={mutating || closing} onClick={() => choose(entry)}>
          <strong>{entry.title}</strong><span>{entry.category}{entry.project ? ` · ${entry.project}` : ''}</span><span className={`memory-enabled${entry.enabled ? ' active' : ''}`}>{entry.enabled ? 'Enabled for answers' : 'Local only'}</span>
        </button>)}</div>
        {hasMore && next !== null && <Button quiet disabled={busy} onClick={() => void load('', next)}>Older memories</Button>}
      </section>
      <section className="memory-editor" aria-labelledby="memory-editor-title">
        {confirmClose && <div className="memory-confirm" role="alert"><p>You have unsaved changes. Discard them and close memory?</p><div className="memory-actions"><Button disabled={busy} onClick={() => void close(true)}>Discard changes and close</Button><Button quiet disabled={busy} onClick={() => { setConfirmClose(false); title.current?.focus(); }}>Keep editing</Button></div></div>}
        {pendingSelection && <div className="memory-confirm" role="alert"><p>You have unsaved changes. Discard them to open {pendingSelection === 'new' ? 'a new memory' : 'another memory'}?</p><div className="memory-actions"><Button onClick={() => choose(pendingSelection, true)} disabled={mutating || closing}>Discard changes</Button><Button quiet onClick={() => setPendingSelection(null)} disabled={mutating || closing}>Keep editing</Button></div></div>}
        {!editing && formError && <p className="memory-error" role="alert">{formError}</p>}
        {notice && <p className="memory-message" role="status">{notice}</p>}
        {!editing ? <div className="memory-editor-empty"><h2 id="memory-editor-title">Context you control</h2><p>Select a memory to review it, or add a note about a project, person or preference.</p><p>New memories stay local until you enable them for answers.</p></div> : <form onSubmit={save} className="memory-form">
          <div><h2 id="memory-editor-title">{selected ? 'Edit memory' : 'New memory'}</h2>{selected && <p className="memory-provenance">Manual entry · Created {date(selected.createdAt)}<br />Updated {date(selected.updatedAt)}</p>}</div>
          <div className="field"><label htmlFor="memory-entry-title">Title</label><input ref={title} id="memory-entry-title" className="input" value={draft.title} disabled={mutating || closing} onChange={event => update('title', event.target.value)} aria-describedby="memory-title-limit" /><small id="memory-title-limit">Up to 200 characters.</small></div>
          <div className="memory-field-pair"><div className="field"><label htmlFor="memory-category">Category</label><select id="memory-category" className="input" value={draft.category} disabled={mutating || closing} onChange={event => update('category', event.target.value as MemoryInput['category'])}>{categories.map(category => <option key={category} value={category}>{category[0].toUpperCase() + category.slice(1)}</option>)}</select></div><div className="field"><label htmlFor="memory-project">Project <span className="memory-optional">optional</span></label><input id="memory-project" className="input" value={draft.project ?? ''} disabled={mutating || closing} onChange={event => update('project', event.target.value || null)} /></div></div>
          <div className="field"><label htmlFor="memory-body">Memory text</label><textarea id="memory-body" className="input" value={draft.body} disabled={mutating || closing} onChange={event => update('body', event.target.value)} aria-describedby="memory-body-limit" /><small id="memory-body-limit">{bytes(draft.body).toLocaleString()} / 16,384 UTF-8 bytes</small></div>
          <label className="checkbox memory-enable"><input type="checkbox" checked={draft.enabled} disabled={mutating || closing} onChange={event => update('enabled', event.target.checked)} aria-describedby="memory-permission-help" />Enable for relevant answers</label>
          {formError && <p className="memory-error" role="alert">{formError}</p>}
          <div className="memory-form-actions">{selected && <Button quiet disabled={busy} onClick={() => setConfirmDelete(true)}>Delete memory</Button>}<Button primary type="submit" disabled={busy || !desktop || (!dirty && !!selected)}>{mutating ? 'Working…' : 'Save memory'}</Button></div>
          {confirmDelete && <div className="memory-confirm" role="alert"><p>Delete this local memory? This cannot be undone.</p><div className="memory-actions"><Button disabled={busy} onClick={() => void remove()}>Delete permanently</Button><Button quiet disabled={busy} onClick={() => setConfirmDelete(false)}>Keep memory</Button></div></div>}
        </form>}
      </section>
    </div>
  </section>;
}
