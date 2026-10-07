import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { DocumentEntry, DocumentChunk } from '../bridge';
import { checkDocument, closeDocuments, deleteDocument, desktop, importDocument, listDocuments, reindexDocument, searchDocuments, setDocumentEnabled } from '../bridge';
import { Button, Icon, IconButton } from './primitives';
import '../styles/documents.css';

const errorText = (error: unknown) => error instanceof Error ? error.message : String(error);
const bytes = (text: string) => new TextEncoder().encode(text).byteLength;
const date = (value: number) => new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value));

export default function DocumentsView() {
  const [documents, setDocuments] = useState<DocumentEntry[]>([]);
  const [next, setNext] = useState<number | null>(null);
  const [selected, setSelected] = useState<DocumentEntry | null>(null);
  const [sourceState, setSourceState] = useState<{ changed: boolean; missing: boolean } | null>(null);
  const [query, setQuery] = useState('');
  const [searched, setSearched] = useState('');
  const [matches, setMatches] = useState<DocumentChunk[]>([]);
  const [loading, setLoading] = useState(desktop);
  const [mutating, setMutating] = useState(false);
  const [closing, setClosing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [sourcePolicy, setSourcePolicy] = useState<'reference' | 'copy'>('reference');
  const [project, setProject] = useState('');
  const [enabled, setEnabled] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const mounted = useRef(true);
  const generation = useRef(0);
  const reading = useRef(false);
  const mutation = useRef(false);
  const closingRef = useRef(false);
  const busy = loading || mutating || closing;

  async function load(beforeId: number | null = null) {
    if (!desktop || mutation.current || closingRef.current || reading.current) return;
    const request = ++generation.current;
    reading.current = true; setLoading(true); setError(null);
    try {
      const page = await listDocuments(beforeId);
      if (mounted.current && request === generation.current) {
        setDocuments(page.documents); setNext(page.hasMore ? page.next : null); setMatches([]); setSearched('');
        setSelected(previous => previous ? page.documents.find(entry => entry.id === previous.id) ?? previous : null);
      }
    } catch (failure) {
      if (mounted.current && request === generation.current) setError(errorText(failure));
    } finally {
      if (mounted.current && request === generation.current) { reading.current = false; setLoading(false); }
    }
  }

  useEffect(() => {
    mounted.current = true;
    void load();
    return () => { mounted.current = false; generation.current += 1; reading.current = false; };
  }, []);

  async function close() {
    if (closingRef.current) return;
    if (mutation.current) { setNotice('Finish the current change before closing documents.'); return; }
    closingRef.current = true; setClosing(true); setError(null);
    try { await closeDocuments(); }
    catch (failure) { if (mounted.current) setError(errorText(failure)); }
    finally { closingRef.current = false; if (mounted.current) setClosing(false); }
  }

  useEffect(() => {
    if (!desktop) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen('documents-close-requested', () => { if (!disposed) void close(); }).then(stop => {
      if (disposed) stop(); else unlisten = stop;
    }).catch(failure => { if (!disposed && mounted.current) setError(errorText(failure)); });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  async function choose(entry: DocumentEntry) {
    if (mutation.current || reading.current || closingRef.current) return;
    const request = ++generation.current;
    reading.current = true; setLoading(true); setSelected(entry); setSourceState(null);
    setConfirmDelete(false); setSearched(''); setMatches([]); setError(null); setNotice(null);
    try {
      const state = await checkDocument(entry.id);
      if (mounted.current && request === generation.current) setSourceState(state);
    } catch (failure) {
      if (mounted.current && request === generation.current) setError(errorText(failure));
    } finally {
      if (mounted.current && request === generation.current) { reading.current = false; setLoading(false); }
    }
  }

  async function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!desktop || mutation.current || reading.current || closingRef.current) return;
    const text = query.trim();
    if (!text) { await load(); return; }
    if (bytes(text) > 512 || text.split(/\s+/).length > 16) {
      setError('Search is limited to 512 UTF-8 bytes and 16 terms.'); return;
    }
    const request = ++generation.current;
    reading.current = true; setLoading(true); setError(null); setNotice(null); setConfirmDelete(false);
    try {
      const results = await searchDocuments(text);
      if (mounted.current && request === generation.current) { setMatches(results); setSearched(text); }
    } catch (failure) {
      if (mounted.current && request === generation.current) setError(errorText(failure));
    } finally {
      if (mounted.current && request === generation.current) { reading.current = false; setLoading(false); }
    }
  }

  async function change(action: 'import' | 'reindex' | 'toggle' | 'delete') {
    if (!desktop || mutation.current || reading.current || closingRef.current || (action !== 'import' && !selected)) return;
    const projectName = project.trim() || null;
    if (action === 'import' && projectName && (Array.from(projectName).length > 100 || bytes(projectName) > 400)) {
      setError('Project is limited to 100 characters and 400 UTF-8 bytes.'); return;
    }
    mutation.current = true; setMutating(true); setError(null); setNotice(null);
    let refresh = false;
    try {
      if (action === 'delete') {
        const deleted = await deleteDocument(selected!.id);
        if (!mounted.current) return;
        setSelected(null); setSourceState(null); setConfirmDelete(false);
        setNotice(deleted.cleanupPending ? 'Local index deleted. Managed-copy cleanup will retry on the next document operation or restart. The original file was kept.' : 'Local index and any managed copy deleted. The original file was kept.'); refresh = true;
      } else {
        const entry = action === 'import' ? await importDocument(sourcePolicy, projectName, enabled)
          : action === 'reindex' ? await reindexDocument(selected!.id)
            : await setDocumentEnabled(selected!.id, !selected!.enabled);
        if (!mounted.current || !entry) return;
        setSelected(entry); setConfirmDelete(false); setSearched(''); setMatches([]);
        if (action !== 'toggle') setSourceState(null);
        setNotice(action === 'import' ? 'Document indexed locally.' : action === 'reindex' ? 'Local index updated.' : entry.enabled ? 'Enabled for relevant authorized answers.' : 'Document is now local only.');
        refresh = true;
      }
    } catch (failure) {
      if (mounted.current) setError(errorText(failure));
    } finally {
      mutation.current = false;
      if (mounted.current) setMutating(false);
    }
    if (refresh && mounted.current) await load();
  }

  return <section className="documents" aria-labelledby="documents-title" aria-busy={busy}>
    <header className="documents-heading"><div><h1 id="documents-title">Documents</h1><p>Index useful files locally. You choose what can inform an answer.</p></div>
      <IconButton icon="refresh" label="Reload documents" disabled={busy || !desktop} onClick={() => void load()} />
    </header>
    {!desktop && <p className="documents-message" role="status">Import and document management are available in the desktop app. This preview does not save files.</p>}
    <details className="documents-import" open={!documents.length && !selected}>
      <summary>Import a document</summary>
      <div className="documents-import-fields">
        <div className="field"><label htmlFor="document-policy">Source handling</label><select id="document-policy" className="input" value={sourcePolicy} disabled={busy || !desktop} onChange={event => setSourcePolicy(event.target.value as 'reference' | 'copy')} aria-describedby="document-policy-help"><option value="reference">Reference original file</option><option value="copy">Keep a local copy</option></select><small id="document-policy-help">A reference reads the original when you reindex. A copy keeps an additional file managed by Harness.</small></div>
        <div className="field"><label htmlFor="document-project">Project <span className="documents-optional">optional</span></label><input id="document-project" className="input" value={project} disabled={busy || !desktop} onChange={event => setProject(event.target.value)} /></div>
        <label className="checkbox documents-enable"><input type="checkbox" checked={enabled} disabled={busy || !desktop} onChange={event => setEnabled(event.target.checked)} aria-describedby="documents-permission-help" />Enable for relevant answers</label>
        <p className="documents-help">Text, Markdown and JSON files only. Up to 1 MiB per file and 256 KiB of extracted text. PDF and Word import are not available yet. Import does not send a remote request.</p>
        <Button primary disabled={busy || !desktop} onClick={() => void change('import')}>{mutating ? 'Working…' : 'Choose file'}</Button>
      </div>
    </details>
    <p className="documents-help" id="documents-permission-help">Only documents you enable may be included when relevant in an authorized answer, including consented fallback providers. New imports are local only by default.</p>
    {error && <p className="documents-error" role="alert">{error}</p>}
    {notice && <p className="documents-message" role="status">{notice}</p>}
    <div className="documents-workspace">
      <section className="documents-library" aria-label="Indexed documents">
        <form className="documents-search" onSubmit={search}><Icon name="search" /><input value={query} onChange={event => setQuery(event.target.value)} disabled={busy || !desktop} aria-label="Search local document text" placeholder="Search document text" /><button type="submit" disabled={busy || !desktop}>Search</button></form>
        <p className="documents-help">Search includes local-only documents.</p>
        {loading && <p className="documents-message" role="status">Loading documents…</p>}
        {!loading && !documents.length && <p className="documents-message">No documents indexed yet. Import a file to retain searchable context.</p>}
        <div className="documents-list">{documents.map(entry => <button key={entry.id} type="button" className={`documents-row${selected?.id === entry.id ? ' selected' : ''}`} aria-pressed={!searched && selected?.id === entry.id} disabled={busy || !desktop} onClick={() => void choose(entry)}><strong>{entry.title}</strong>{entry.project && <span>{entry.project}</span>}<span>{entry.enabled ? 'Enabled for answers' : 'Local only'} · {entry.chunkCount} chunks</span></button>)}</div>
        {next !== null && <Button quiet disabled={busy} onClick={() => void load(next)}>Older documents</Button>}
      </section>
      <section className="documents-detail" aria-labelledby="document-detail-title">
        {searched ? <><div className="documents-search-heading"><h2 id="document-detail-title">Local search results</h2><button type="button" className="text-link" disabled={busy} onClick={() => { setQuery(''); setSearched(''); setMatches([]); }}>Clear search</button></div><p className="documents-help">Matches for "{searched}". These are indexed source excerpts, not AI answers.</p>{!matches.length && <p className="documents-message">No matching text. Try a different search.</p>}<div className="documents-matches">{matches.map(chunk => <article key={chunk.id} className="documents-match"><h3>{chunk.title}</h3><p className="documents-help">Chunk {chunk.chunkIndex + 1} · Index version {chunk.indexingVersion}</p><p className="documents-excerpt">{chunk.text}</p><details className="documents-provenance"><summary>Source details</summary><dl><dt>Original path</dt><dd>{chunk.sourcePath}</dd><dt>Content hash</dt><dd className="documents-hash">{chunk.contentHash}</dd></dl></details></article>)}</div></> : selected ? <><div><h2 id="document-detail-title">{selected.title}</h2><p className="documents-help">{selected.enabled ? 'Enabled for relevant answers' : 'Local only'}{selected.project ? ` · ${selected.project}` : ''}</p></div>
          {sourceState && <p className="documents-message" role="status">{sourceState.missing ? 'The source file is missing. The saved index remains available.' : sourceState.changed ? 'The source has changed. Reindex when you want to replace the saved text.' : 'The source matches the saved index.'}</p>}
          <div className="documents-actions"><Button disabled={busy || !desktop} onClick={() => void change('toggle')} aria-describedby="documents-permission-help">{selected.enabled ? 'Keep local only' : 'Enable for answers'}</Button><Button quiet disabled={busy || !desktop || sourceState?.missing} onClick={() => void change('reindex')}>Reindex document</Button></div>
          <p className="documents-help">Reindexing replaces the saved text only after a successful import. It does not send a remote request.</p>
          <dl className="documents-metadata"><dt>Source handling</dt><dd>{selected.sourcePolicy === 'copy' ? 'Managed local copy' : 'Original file reference'}</dd><dt>Original path</dt><dd>{selected.sourcePath}</dd><dt>File modified</dt><dd>{date(selected.modifiedAt)}</dd><dt>Indexed</dt><dd>{date(selected.indexedAt)}</dd><dt>Index version</dt><dd>{selected.indexingVersion}</dd><dt>Indexed text</dt><dd>{selected.chunkCount} chunks · {selected.textBytes.toLocaleString()} bytes</dd><dt>Content hash</dt><dd className="documents-hash">{selected.contentHash}</dd></dl>
          <div className="documents-delete"><Button quiet disabled={busy || !desktop} onClick={() => setConfirmDelete(true)}>Delete local index</Button>{confirmDelete && <div className="documents-confirm" role="alert"><p>Delete the local index and any managed copy? The original file will not be deleted. This cannot be undone.</p><div className="documents-actions"><Button disabled={busy} onClick={() => void change('delete')}>Delete permanently</Button><Button quiet disabled={busy} onClick={() => setConfirmDelete(false)}>Keep document</Button></div></div>}</div>
        </> : <div className="documents-empty"><h2 id="document-detail-title">Source context you control</h2><p>Select a document to inspect its provenance and answer permission. Search to read its indexed excerpts.</p><p>Harness checks a selected source for changes. It reindexes only when you ask.</p></div>}
      </section>
    </div>
  </section>;
}
