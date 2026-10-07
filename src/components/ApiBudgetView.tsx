import { useEffect, useRef, useState } from 'react';
import { apiBudgets, saveApiBudget, desktop, type ApiBudgetProvider, type ApiBudgetInput, type ApiPrice } from '../bridge';
import { Button, Icon } from './primitives';
import '../styles/budgets.css';

// Money stays in integer microdollars; decimal input never passes through floating-point rounding.
export function parseBudgetMoney(value: string, maximum: number): number | null {
  const text = value.trim();
  if (!text) return null;
  if (!/^\d{1,10}(?:\.\d{1,6})?$/.test(text)) throw new Error('Enter a USD amount with up to six decimal places.');
  const [whole, fraction = ''] = text.split('.');
  const micros = BigInt(whole) * 1_000_000n + BigInt(fraction.padEnd(6, '0'));
  if (micros > BigInt(maximum)) throw new Error(`The USD amount must be at most ${formatMoney(maximum)}.`);
  return Number(micros);
}
const formatMoney = (value: number | null) => value === null ? '' : `${Math.floor(value / 1_000_000)}.${String(value % 1_000_000).padStart(6, '0')}`.replace(/\.?0+$/, '');
const money = (value: number | null) => value === null ? 'Unknown' : `$${formatMoney(value)}`;
const names = { gemini: 'Gemini', deepseek: 'DeepSeek' };
const reasons: Record<string, string> = {
  legacy_usage: 'Earlier usage cannot be attributed to this key. Enabled caps block sending until the daily reset.',
  pending: 'An attempt is pending. Enabled caps wait for its reported usage.',
  unknown_tokens: 'Reported token usage is incomplete. The token cap blocks sending until the daily reset.',
  unknown_costs: 'Estimated cost is incomplete. The cost cap blocks sending until the daily reset.',
  missing_price: 'Add model prices to use the daily cost cap.',
  token_cap: 'The daily token cap has been reached.',
  cost_cap: 'The daily estimated-cost cap has been reached.',
};

export function validApiBudgets(value: unknown): value is ApiBudgetProvider[] {
  const integer = (v: unknown, max = Number.MAX_SAFE_INTEGER) => Number.isSafeInteger(v) && (v as number) >= 0 && (v as number) <= max;
  const nullable = (v: unknown, max?: number) => v === null || integer(v, max);
  const price = (v: ApiPrice | null) => v === null || v && integer(v.inputMicrosPerMillion, 1e9) && integer(v.outputMicrosPerMillion, 1e9) && nullable(v.cachedMicrosPerMillion, 1e9) && (v.cachedMicrosPerMillion === null || v.cachedMicrosPerMillion <= v.inputMicrosPerMillion);
  return Array.isArray(value) && value.length === 2 && new Set(value.map(v => v?.provider)).size === 2 && value.every(v => {
    if (!v || !(v.provider === 'gemini' || v.provider === 'deepseek') || !(v.model === null || typeof v.model === 'string' && v.model.length > 0 && v.model.length <= 200 && !/[\x00-\x1f\x7f]/.test(v.model)) || !(v.keyId === null || typeof v.keyId === 'string' && /^[A-Za-z0-9_-]{43}$/.test(v.keyId))) return false;
    const b = v.budget;
    if (v.keyId === null) return b === null;
    return b && nullable(b.tokenCap, 1e12) && nullable(b.costCapMicros, 1e15) && price(b.price) && integer(b.todayTokens) && nullable(b.todayCostMicros) && integer(b.attempts) && integer(b.unknownTokens) && integer(b.unknownCosts) && integer(b.pending) && (b.reason === null || Object.hasOwn(reasons, b.reason)) && integer(b.resetAt, 8_640_000_000_000);
  });
}

type Draft = { tokens: string; cost: string; input: string; output: string; cached: string };
const draftOf = (row: ApiBudgetProvider): Draft => ({ tokens: row.budget?.tokenCap?.toString() ?? '', cost: formatMoney(row.budget?.costCapMicros ?? null), input: formatMoney(row.budget?.price?.inputMicrosPerMillion ?? null), output: formatMoney(row.budget?.price?.outputMicrosPerMillion ?? null), cached: formatMoney(row.budget?.price?.cachedMicrosPerMillion ?? null) });
const failureText = (failure: unknown) => failure instanceof Error ? failure.message : String(failure);

function ProviderBudget({ initial, disabled, onDirty, onBusy }: { initial: ApiBudgetProvider; disabled: boolean; onDirty: (dirty: boolean) => void; onBusy: (busy: boolean) => void }) {
  const [row, setRow] = useState(initial);
  const [draft, setDraft] = useState(() => draftOf(initial));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const generation = useRef(0);
  const baseline = draftOf(row);
  const dirty = (Object.keys(draft) as (keyof Draft)[]).some(k => draft[k] !== baseline[k]);
  useEffect(() => { onDirty(dirty); }, [dirty, onDirty]);
  useEffect(() => () => { generation.current++; }, []);
  const available = desktop && row.keyId !== null && row.budget !== null;
  const prefix = `budget-${row.provider}`;

  async function save() {
    if (!available || !row.keyId || busy || disabled) return;
    setError(null); setSaved(false);
    let input: ApiBudgetInput;
    try {
      const tokenText = draft.tokens.trim();
      if (tokenText && (!/^\d{1,13}$/.test(tokenText) || Number(tokenText) > 1e12)) throw new Error('Daily tokens must be a whole number between 0 and 1,000,000,000,000.');
      const rates = [draft.input, draft.output, draft.cached].map(v => parseBudgetMoney(v, 1e9));
      if (rates.some(v => v !== null) && (rates[0] === null || rates[1] === null)) throw new Error('Enter both input and output prices, or leave all model prices blank.');
      if (rates[2] !== null && rates[0] !== null && rates[2] > rates[0]) throw new Error('Cached input price cannot exceed the regular input price.');
      if (rates.some(v => v !== null) && !row.model) throw new Error('Select a model in Connections before setting its prices.');
      input = { provider: row.provider, keyId: row.keyId, model: row.model, tokenCap: tokenText ? Number(tokenText) : null, costCapMicros: parseBudgetMoney(draft.cost, 1e15), price: rates[0] !== null && rates[1] !== null ? { inputMicrosPerMillion: rates[0], outputMicrosPerMillion: rates[1], cachedMicrosPerMillion: rates[2] } : null };
    } catch (failure) { setError(failureText(failure)); return; }
    const current = ++generation.current;
    setBusy(true); onBusy(true);
    try {
      await saveApiBudget(input);
      if (current !== generation.current) return;
      const rows = await apiBudgets();
      if (current !== generation.current) return;
      if (!validApiBudgets(rows)) throw new Error('The budget record is invalid. Your edits are retained; refresh to retry.');
      const updated = rows.find(v => v.provider === row.provider);
      if (!updated || updated.keyId !== row.keyId || updated.model !== row.model) throw new Error('The key or model changed. Refresh before editing its budget.');
      setRow(updated); setDraft(draftOf(updated)); setSaved(true);
    } catch (failure) { if (current === generation.current) setError(failureText(failure)); }
    finally { if (current === generation.current) { setBusy(false); onBusy(false); } }
  }
  const fields: [keyof Draft, string, string][] = [
    ['tokens', 'Daily token cap', 'Blank means no token cap; 0 blocks new attempts. Input and output tokens count together.'],
    ['cost', 'Daily estimated-cost cap (USD)', 'Blank means no cost cap; 0 blocks new attempts.'],
    ['input', 'Input price (USD / million tokens)', 'Use the price for the selected model. Leave all prices blank if unknown.'],
    ['output', 'Output price (USD / million tokens)', 'Input and output prices must be entered together.'],
    ['cached', 'Cached input price (USD / million tokens)', 'Optional. Missing cached counts or price use the regular input price.'],
  ];
  return <details className="budget-provider">
    <summary><span>{names[row.provider]}</span><span className="help">{row.keyId ? `Key ${row.keyId.slice(0, 8)}…${row.keyId.slice(-4)}` : 'No API key saved'} · {row.model ?? 'No model selected'}</span></summary>
    {!available && <p className="help">Save this provider’s API key in Connections to configure its budget.</p>}
    {row.budget && <>
      <dl className="usage-values"><dt>Today’s reported tokens</dt><dd>{row.budget.todayTokens.toLocaleString()}</dd><dt>Today’s estimated cost</dt><dd>{money(row.budget.todayCostMicros)}</dd><dt>Attempts / pending</dt><dd>{row.budget.attempts} / {row.budget.pending}</dd><dt>Unknown tokens / costs</dt><dd>{row.budget.unknownTokens} / {row.budget.unknownCosts}</dd></dl>
      {row.budget.reason && <p role="status" className="help">{reasons[row.budget.reason]}</p>}
      <p className="help">Daily reset: {new Date(row.budget.resetAt * 1000).toLocaleString()}. {row.budget.unknownTokens > 0 || row.budget.unknownCosts > 0 ? 'Reported totals may be partial.' : ''}</p>
    </>}
    {available && <form onSubmit={event => { event.preventDefault(); void save(); }} aria-busy={busy}>
      <fieldset disabled={!available || busy || disabled} className="budget-fields"><legend className="field-label">{names[row.provider]} limits and model prices</legend>
        {fields.map(([key, label, help]) => <div className="field" key={key}><label htmlFor={`${prefix}-${key}`}>{label}</label><input id={`${prefix}-${key}`} className="input" type="text" inputMode={key === 'tokens' ? 'numeric' : 'decimal'} autoComplete="off" maxLength={24} value={draft[key]} disabled={key === 'input' || key === 'output' || key === 'cached' ? !row.model : undefined} aria-describedby={`${prefix}-${key}-help`} onChange={event => { setDraft({ ...draft, [key]: event.target.value }); setSaved(false); }} /><small id={`${prefix}-${key}-help`}>{help}</small></div>)}
      </fieldset>
      {!row.model && available && <p className="help">Daily token limits can be saved now. Select a model in Connections to set prices.</p>}
      {error && <p role="alert" className="notice error">{error}</p>}
      {saved && <p role="status" className="help">Budget saved.</p>}
      <Button type="submit" disabled={!available || busy || disabled || !dirty}>{busy ? 'Saving…' : 'Save budget'}</Button>
    </form>}
  </details>;
}

export default function ApiBudgetView() {
  const [rows, setRows] = useState<ApiBudgetProvider[] | null>(null);
  const [busy, setBusy] = useState(desktop);
  const [error, setError] = useState<string | null>(null);
  const [confirmRefresh, setConfirmRefresh] = useState(false);
  const [dirty, setDirty] = useState<Record<string, boolean>>({});
  const [saving, setSaving] = useState<Record<string, boolean>>({});
  const [epoch, setEpoch] = useState(0);
  const generation = useRef(0);
  const hasEdits = Object.values(dirty).some(Boolean);
  const hasSave = Object.values(saving).some(Boolean);
  async function load() {
    if (!desktop) return;
    const current = ++generation.current;
    setBusy(true); setError(null); setConfirmRefresh(false);
    try {
      const value = await apiBudgets();
      if (current !== generation.current) return;
      if (!validApiBudgets(value)) throw new Error('The local budget record is invalid. Retry the lookup; existing edits are retained.');
      setRows(value); setEpoch(v => v + 1); setDirty({}); setSaving({});
    } catch (failure) { if (current === generation.current) setError(failureText(failure)); }
    finally { if (current === generation.current) setBusy(false); }
  }
  useEffect(() => { void load(); return () => { generation.current++; }; }, []);
  return <section className="api-budgets" aria-label="API budgets" aria-busy={busy}>
    <div className="usage-heading"><h3>API budgets</h3><Button quiet disabled={!desktop || busy || hasSave} onClick={() => hasEdits ? setConfirmRefresh(true) : void load()}><Icon name="refresh" />Refresh budgets</Button></div>
    <p className="help">Optional daily limits per API key. These check local usage before sending; an ongoing request can exceed a cap. They are not provider billing limits.</p>
    {!desktop && <p role="status" className="help">Budget settings are available in the desktop app.</p>}
    {busy && <p role="status" className="help">Loading API budgets…</p>}
    {confirmRefresh && <div className="usage-actions"><p className="help">Refresh will discard your unsaved budget edits.</p><Button quiet onClick={() => void load()}>Discard edits and refresh</Button><Button quiet onClick={() => setConfirmRefresh(false)}>Keep editing</Button></div>}
    {error && <div className="usage-actions"><p role="alert" className="notice error">{error}</p><Button quiet disabled={busy || hasSave} onClick={() => hasEdits ? setConfirmRefresh(true) : void load()}>Retry budget lookup</Button></div>}
    {rows?.map(row => <ProviderBudget key={`${epoch}:${row.provider}`} initial={row} disabled={busy} onDirty={value => setDirty(previous => previous[row.provider] === value ? previous : { ...previous, [row.provider]: value })} onBusy={value => setSaving(previous => ({ ...previous, [row.provider]: value }))} />)}
    <details className="budget-explanation"><summary>How estimates and limits work</summary><p className="help">Prices are your USD estimates per million tokens, captured when each attempt starts. Unknown prices stay unknown. Missing reported usage blocks the corresponding enabled cap until the local daily reset; a pending attempt blocks capped keys until it finishes.</p><p className="help">Caps follow the key and endpoint. Deleting a meeting does not reset them. Completed anonymous counters remain for eight local calendar days; live pending attempts remain until resolved; no meeting text is retained in these counters.</p></details>
  </section>;
}

export function budgetParserSelfCheck() {
  for (const [text, expected] of [['', null], ['0', 0], ['0.000001', 1], ['1.234567', 1234567], ['1000000000', 1e15]] as const) if (parseBudgetMoney(text, 1e15) !== expected) throw new Error('Budget money parsing regression.');
  for (const text of ['-1', '1e3', '0.0000001', '1,000', 'Infinity', '1.']) { let rejected = false; try { parseBudgetMoney(text, 1e15); } catch { rejected = true; } if (!rejected) throw new Error('Invalid budget money accepted.'); }
  let rejected = false; try { parseBudgetMoney('1000.000001', 1e9); } catch { rejected = true; } if (!rejected) throw new Error('Budget price bound regression.');
  if (formatMoney(0) !== '0' || formatMoney(1000000) !== '1' || formatMoney(1) !== '0.000001') throw new Error('Budget money display regression.');
  return true;
}
