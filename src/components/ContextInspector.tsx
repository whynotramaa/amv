import { useEffect, useRef, useState } from 'react';
import { requestContext, type ContextSnapshot } from '../bridge';
import { Button } from './primitives';
import '../styles/context.css';

function validSnapshot(value: ContextSnapshot, requestId: number) {
  if (!value || typeof value !== 'object') return false;
  const { request, metadata } = value;
  const number = (input: unknown) => Number.isSafeInteger(input) && (input as number) >= 0;
  const text = (input: unknown) => typeof input === 'string';
  return number(value.id) && value.id > 0 && value.requestId === requestId && text(value.provider) && text(value.model) && (value.accountId == null || (text(value.accountId) && value.accountId.length === 43)) && (value.credentialId == null || (text(value.credentialId) && /^[A-Za-z0-9_-]{43}$/.test(value.credentialId))) && number(value.createdAt) && typeof value.upstreamOmitted === 'boolean'
    && request && text(request.model) && (request.instructions === null || text(request.instructions))
    && Array.isArray(request.messages) && request.messages.length <= 4096 && request.messages.every(message => message && text(message.role) && text(message.content))
    && metadata && text(metadata.model) && number(metadata.estimatedInputTokens) && number(metadata.inputTokenBudget) && number(metadata.responseReserveTokens)
    && Array.isArray(metadata.omissions) && metadata.omissions.length <= 4096 && metadata.omissions.every(item => item && text(item.kind) && number(item.count))
    && Array.isArray(metadata.truncations) && metadata.truncations.length <= 4096 && metadata.truncations.every(item => item && text(item.kind) && (item.id === undefined || text(item.id)) && (item.role === undefined || text(item.role)) && (item.source === undefined || text(item.source)) && (item.startMs === undefined || number(item.startMs)))
    && Array.isArray(metadata.excludedIds) && metadata.excludedIds.length <= 4096 && metadata.excludedIds.every(text);
}

export default function ContextInspector({ requestId }: { requestId: number }) {
  const [snapshot, setSnapshot] = useState<ContextSnapshot | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const generation = useRef(0);
  const activeRequest = useRef(requestId);
  activeRequest.current = requestId;

  async function load(beforeId: number | null = null) {
    const current = ++generation.current;
    const id = requestId;
    setBusy(true); setError(null);
    try {
      const value = await requestContext(id, beforeId);
      if (current !== generation.current || id !== activeRequest.current) return;
      if (value && !validSnapshot(value, id)) throw new Error('The saved context record is invalid.');
      if (beforeId !== null && value === null) { setError('No older prepared attempt is saved.'); return; }
      setSnapshot(value);
    } catch (failure) {
      if (current === generation.current && id === activeRequest.current) setError(failure instanceof Error ? failure.message : String(failure));
    } finally {
      if (current === generation.current && id === activeRequest.current) setBusy(false);
    }
  }
  useEffect(() => {
    setSnapshot(null); void load();
    return () => { generation.current++; };
  }, [requestId]);

  return <section className="context-inspector" aria-label="Prepared request context" aria-busy={busy}>
    <p className="help">Saved locally before this provider attempt. This records prepared context, not confirmation that the server received it. Provider-specific formatting may differ.</p>
    {busy && <p role="status" className="help">Loading saved context…</p>}
    {error && <p role="alert" className="notice error">{error}</p>}
    {!busy && !error && !snapshot && <p className="help">No prepared context is saved for this request. It may predate this feature or have stopped before preparation.</p>}
    {snapshot && <>
      <div className="context-heading"><span>{snapshot.provider} · {snapshot.model}</span><span className="help">{new Date(snapshot.createdAt * 1000).toLocaleString()}</span></div>
      <p className="help">{snapshot.accountId ? `ChatGPT account ${snapshot.accountId.slice(0, 8)}` : snapshot.provider === 'chatgpt' ? 'Account attribution was not recorded for this older attempt.' : snapshot.credentialId ? `API key ${snapshot.credentialId.slice(0,12)}…${snapshot.credentialId.slice(-4)}` : 'API key attribution was not recorded for this older attempt.'}</p>
      <dl className="context-budget"><dt>Estimated input</dt><dd>{snapshot.metadata.estimatedInputTokens} tokens</dd><dt>Input budget</dt><dd>{snapshot.metadata.inputTokenBudget} tokens</dd><dt>Response reserve</dt><dd>{snapshot.metadata.responseReserveTokens} tokens</dd></dl>
      <p className="help">Token counts are conservative byte estimates. Budgets are application limits, not verified model limits.</p>
      {snapshot.upstreamOmitted && <p className="help">Some history or recent transcript was excluded before compilation.</p>}
      {!!snapshot.metadata.omissions.length && <div><h3>Omitted context</h3><ul>{snapshot.metadata.omissions.map((item, index) => <li key={index}>{item.kind}: {item.count}</li>)}</ul></div>}
      {!!snapshot.metadata.truncations.length && <div><h3>Records excluded by the budget</h3><p className="help">These records were excluded whole. Their text was not shortened.</p><ul>{snapshot.metadata.truncations.map((item, index) => <li key={index}>{item.kind}{item.id ? ` · ${item.id}` : item.role ? ` · ${item.role}` : ''}</li>)}</ul></div>}
      {!!snapshot.metadata.excludedIds.length && <details><summary>Excluded source IDs</summary><p className="context-text">{snapshot.metadata.excludedIds.join('\n')}</p></details>}
      {snapshot.request.instructions && <details><summary>Application instructions</summary><p className="context-text">{snapshot.request.instructions}</p></details>}
      <div className="context-messages">{snapshot.request.messages.map((message, index) => <details key={`${snapshot.id}:${index}`} open={index === snapshot.request.messages.length - 1}><summary>{message.role === 'developer' ? 'Source context and provenance' : message.role === 'assistant' ? 'Previous answer' : index === snapshot.request.messages.length - 1 ? 'New user message' : 'Previous user message'}</summary><p className="context-text">{message.content}</p></details>)}</div>
      <div className="context-actions"><Button quiet disabled={busy} onClick={() => void load(snapshot.id)}>Older attempt</Button><Button quiet disabled={busy} onClick={() => void load()}>Latest attempt</Button></div>
    </>}
    {!snapshot && error && <Button quiet disabled={busy} onClick={() => void load()}>Retry context lookup</Button>}
  </section>;
}
