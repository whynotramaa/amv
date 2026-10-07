import { lazy, Suspense, useEffect, useRef, useState } from 'react';
import { desktop, usageReport, type UsageReport } from '../bridge';
import { Button, Icon } from './primitives';
import '../styles/usage.css';

const ApiBudgetView = lazy(() => import('./ApiBudgetView'));
const money = (value:number|null|undefined) => value == null ? 'Unknown' : `USD ${(value/1_000_000).toFixed(6)}`;
const count = (value: number | null) => value === null ? 'Unknown' : value.toLocaleString();
const timestamp = (value: number) => new Date(value * 1000).toLocaleString();
const account = (provider: string, id: string | null, key?:string|null) => key ? `API key ${key.slice(0,12)}…${key.slice(-4)}` : id
  ? `Account ${id.length > 20 ? `${id.slice(0, 12)}…${id.slice(-4)}` : id}`
  : provider === 'gemini' || provider === 'deepseek' ? 'API key · account not recorded' : 'Account unknown';
const providerName = (provider: string) => ({ chatgpt: 'ChatGPT', deepseek: 'DeepSeek', gemini: 'Gemini' }[provider] ?? provider);
const statusName = (status: string) => ({ in_flight: 'Pending', ok: 'Completed', error: 'Failed', limited: 'Limited', fell_back: 'Fallback selected', partial: 'Partial', cancelled: 'Cancelled', interrupted: 'Interrupted' }[status] ?? status);

function validReport(value: UsageReport) {
  const n = (v: unknown) => Number.isSafeInteger(v) && (v as number) >= 0;
  const optional = (v: unknown) => v === null || n(v);
  const date = (v: unknown) => n(v) && (v as number) <= 8_640_000_000_000;
  const text = (v: unknown) => typeof v === 'string';
  const totals = (v: UsageReport['groups'][number]['today']) => v && n(v.attempts) && optional(v.inputTokens) && optional(v.outputTokens) && optional(v.cachedTokens) && n(v.unknownTokenAttempts) && n(v.limitHits) && n(v.inFlight) && (v.estimatedCostMicros === undefined || optional(v.estimatedCostMicros)) && (v.unknownCostAttempts === undefined || n(v.unknownCostAttempts));
  return value && date(value.asOf) && date(value.todayStart) && date(value.weekStart) && typeof value.groupsTruncated === 'boolean' && optional(value.next)
    && Array.isArray(value.groups) && value.groups.length <= 100 && value.groups.every(g => g && text(g.provider) && (g.accountId === null || text(g.accountId)) && (g.credentialId == null || (text(g.credentialId) && g.credentialId.length===43)) && totals(g.today) && totals(g.week))
    && Array.isArray(value.attempts) && value.attempts.length <= 20 && value.attempts.every(a => a && n(a.id) && n(a.requestId) && n(a.meetingId) && text(a.provider) && (a.accountId === null || text(a.accountId)) && text(a.model) && text(a.kind) && text(a.status) && date(a.createdAt) && optional(a.inputTokens) && optional(a.outputTokens) && optional(a.cachedTokens) && optional(a.firstTokenMs) && optional(a.totalMs) && (a.errorCode === null || text(a.errorCode)) && typeof a.limitHit === 'boolean' && (a.credentialId == null || (text(a.credentialId) && a.credentialId.length===43)) && (a.costMicros === undefined || optional(a.costMicros)));
}

export default function UsageView() {
  const [showBudgets, setShowBudgets] = useState(false);
  const [report, setReport] = useState<UsageReport | null>(null);
  const [busy, setBusy] = useState(desktop);
  const [error, setError] = useState<string | null>(null);
  const [cursor, setCursor] = useState<number | null>(null);
  const generation = useRef(0);
  const lastRead = useRef<number | null>(null);

  async function load(beforeId: number | null = null) {
    if (!desktop) return;
    const current = ++generation.current;
    lastRead.current = beforeId;
    setBusy(true); setError(null);
    try {
      const value = await usageReport(beforeId);
      if (current !== generation.current) return;
      if (!validReport(value)) throw new Error('The local usage record is invalid. Retry or restore the database from a backup.');
      setReport(value); setCursor(beforeId);
    } catch (failure) {
      if (current === generation.current) setError(failure instanceof Error ? failure.message : String(failure));
    } finally {
      if (current === generation.current) setBusy(false);
    }
  }
  useEffect(() => { void load(); return () => { generation.current++; }; }, []);

  return <section className="usage" aria-label="Local usage" aria-busy={busy}>
    <div className="usage-heading"><h2>Usage</h2><Button quiet disabled={!desktop || busy} onClick={() => void load(cursor)}><Icon name="refresh" />Refresh</Button></div>
    <p className="help">Local provider attempts and reported tokens. Plan allowance is unavailable. API costs use your saved prices and reported tokens; they are estimates. Missing token counts stay unknown.</p>
    {!showBudgets && <Button quiet onClick={() => setShowBudgets(true)}>API budgets</Button>}
    {showBudgets && <Suspense fallback={<p className="help" role="status">Loading API budgets…</p>}><ApiBudgetView /></Suspense>}
    {!desktop && <p role="status" className="help">Usage records are available in the desktop app. This browser preview has no native records.</p>}
    {busy && <p role="status" className="help">Loading local usage…</p>}
    {error && <div className="usage-actions"><p role="alert" className="notice error">{error}</p><Button quiet disabled={busy} onClick={() => void load(lastRead.current)}>Retry usage lookup</Button></div>}
    {report && <>
      <p className="help">Updated {timestamp(report.asOf)}. Today starts {timestamp(report.todayStart)}; seven local calendar days start {timestamp(report.weekStart)}.</p>
      {!report.groups.length && <p className="help">No provider attempts have been recorded in these periods.</p>}
      {report.groups.map(group => <section className="usage-group" key={`${group.provider}:${group.accountId ?? ''}:${group.credentialId ?? ''}`}>
        <h3>{providerName(group.provider)}</h3><p className="help">{account(group.provider, group.accountId, group.credentialId)}</p>
        {([['Today', group.today], ['Seven local days', group.week]] as const).map(([label, totals]) => <details key={label} className="usage-period">
          <summary><span>{label}</span><span className="usage-measurement">{count(totals.attempts)} attempts · {count(totals.limitHits)} limit hits</span></summary>
          <dl className="usage-values"><dt>Reported input tokens</dt><dd>{count(totals.inputTokens)}</dd><dt>Reported output tokens</dt><dd>{count(totals.outputTokens)}</dd><dt>Reported cached tokens</dt><dd>{count(totals.cachedTokens)}</dd><dt>Attempts missing input/output counts</dt><dd>{count(totals.unknownTokenAttempts)}</dd>{group.provider !== 'chatgpt' && <><dt>Estimated API cost</dt><dd>{money(totals.estimatedCostMicros)}{(totals.unknownCostAttempts ?? 0)>0?' · Partial':''}</dd><dt>Attempts with unknown cost</dt><dd>{totals.unknownCostAttempts === undefined ? 'Unknown' : count(totals.unknownCostAttempts)}</dd></>}<dt>Pending attempts</dt><dd>{count(totals.inFlight)}</dd></dl>
          {totals.unknownTokenAttempts > 0 && <p className="help">Reported totals may be partial. Unknown counts are not treated as zero.</p>}
        </details>)}
      </section>)}
      {report.groupsTruncated && <p className="help">Only a bounded selection of provider/account summaries is shown.</p>}
      <section className="usage-attempts" aria-label="Recorded provider attempts">
        <h3>{cursor === null ? 'Recent attempts' : 'Older attempts'}</h3>
        {!report.attempts.length && <p className="help">No attempts on this page. Older versions may not have recorded usage.</p>}
        {report.attempts.map(attempt => <details className="usage-attempt" key={attempt.id}>
          <summary><span>{providerName(attempt.provider)} · {statusName(attempt.status)}{attempt.limitHit ? ' · Limit hit' : ''}</span><time className="help" dateTime={new Date(attempt.createdAt * 1000).toISOString()}>{timestamp(attempt.createdAt)}</time></summary>
          <p className="help">{account(attempt.provider, attempt.accountId, attempt.credentialId)} · {attempt.model}</p>
          {attempt.status === 'interrupted' && <p className="help">The attempt ended without a recorded final result.</p>}
          {attempt.status === 'in_flight' && <p className="help">No final result has been recorded yet. Refresh to check again.</p>}
          <dl className="usage-values">{attempt.provider !== 'chatgpt' && <><dt>Estimated API cost</dt><dd>{money(attempt.costMicros)}</dd></>}<dt>Input tokens</dt><dd>{count(attempt.inputTokens)}</dd><dt>Output tokens</dt><dd>{count(attempt.outputTokens)}</dd><dt>Cached tokens</dt><dd>{count(attempt.cachedTokens)}</dd><dt>First token</dt><dd>{attempt.firstTokenMs === null ? 'Unknown' : `${count(attempt.firstTokenMs)} ms`}</dd><dt>Total duration</dt><dd>{attempt.totalMs === null ? 'Unknown' : `${count(attempt.totalMs)} ms`}</dd><dt>Request / meeting</dt><dd>{attempt.requestId} / {attempt.meetingId}</dd><dt>Request type</dt><dd>{attempt.kind}</dd>{attempt.errorCode && <><dt>Diagnostic code</dt><dd>{attempt.errorCode}</dd></>}</dl>
        </details>)}
        <div className="usage-actions"><Button quiet disabled={busy || report.next === null} onClick={() => void load(report.next)}>Older attempts</Button>{cursor !== null && <Button quiet disabled={busy} onClick={() => void load()}>Latest attempts</Button>}</div>
      </section>
    </>}
  </section>;
}
