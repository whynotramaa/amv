import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { beginSignIn, cancelSignIn, chatgptModels, deleteApiKey, desktop, loadConnections, providerModels, reauthorizeAccount, saveProviders, selectAccount, selectChatgptModel, setApiKey, signOutAccount, type AccountMetadata, type ConnectionState, type ModelInfo, type ProviderConfig } from '../bridge';
import { Button } from './primitives';

type Notice = { text: string; error: boolean };

export default function Connections({ onChanged }: { onChanged: () => void }) {
  const [state, setState] = useState<ConnectionState | null>(null);
  const [draft, setDraft] = useState<ProviderConfig[]>([]);
  const [busy, setBusy] = useState(false);
  const [signInAction, setSignInAction] = useState<'begin' | 'cancel' | null>(null);
  const [modelBusy, setModelBusy] = useState<string | null>(null);
  const [modelsByAccount, setModelsByAccount] = useState<Record<string, ModelInfo[]>>({});
  const [notice, setNotice] = useState<Notice | null>(null);
  const mounted = useRef(false);
  const reloadGeneration = useRef(0);
  const modelGeneration = useRef(0);
  const signInGeneration = useRef(0);
  const activeAccount = useRef<string | null>(null);

  function commit(next: ConnectionState, replaceDraft = false) {
    if (activeAccount.current !== next.activeAccount) {
      activeAccount.current = next.activeAccount;
      ++modelGeneration.current;
      setModelBusy(null);
      setModelsByAccount({});
    }
    setState(next);
    if (replaceDraft) setDraft(next.providers.map(provider => provider.config));
    onChanged();
  }

  async function reload(replaceDraft = false) {
    const generation = ++reloadGeneration.current;
    try {
      const next = await loadConnections();
      if (mounted.current && generation === reloadGeneration.current) { commit(next, replaceDraft); return true; }
    } catch {
      if (mounted.current && generation === reloadGeneration.current) setNotice({ text: 'Couldn’t refresh connections.', error: true });
    }
    return false;
  }

  useEffect(() => {
    mounted.current = true;
    void reload(true);
    const pending = desktop ? listen('connections-changed', () => { void reload(); }) : null;
    return () => {
      mounted.current = false;
      ++reloadGeneration.current;
      ++modelGeneration.current;
      ++signInGeneration.current;
      if (pending) void pending.then(unlisten => unlisten()).catch(() => {});
    };
  }, []);

  async function run(action: () => Promise<unknown>, success: string, replaceDraft = false) {
    setBusy(true); setNotice(null);
    try { await action(); const refreshed = await reload(replaceDraft); if (mounted.current && refreshed) setNotice({ text: success, error: false }); }
    catch (error) { if (mounted.current) setNotice({ text: error instanceof Error ? error.message : String(error), error: true }); }
    finally { if (mounted.current) setBusy(false); }
  }

  async function startSignIn() {
    if (!desktop || signInAction || state?.signingIn) return;
    const generation = ++signInGeneration.current;
    setNotice(null); setSignInAction('begin');
    setState(current => current ? { ...current, signingIn: true } : current);
    try { await beginSignIn(); if (mounted.current && generation === signInGeneration.current) await reload(); }
    catch (error) {
      if (mounted.current && generation === signInGeneration.current) {
        setState(current => current ? { ...current, signingIn: false } : current);
        await reload();
        if (generation === signInGeneration.current) setNotice({ text: error instanceof Error ? error.message : String(error), error: true });
      }
    }
    finally { if (mounted.current && generation === signInGeneration.current) setSignInAction(null); }
  }

  async function stopSignIn() {
    if (signInAction === 'cancel') return;
    const generation = ++signInGeneration.current;
    setNotice(null); setSignInAction('cancel');
    try { await cancelSignIn(); const refreshed = await reload(); if (mounted.current && refreshed && generation === signInGeneration.current) setNotice({ text: 'Sign-in cancelled.', error: false }); }
    catch (error) { if (mounted.current && generation === signInGeneration.current) setNotice({ text: error instanceof Error ? error.message : String(error), error: true }); }
    finally { if (mounted.current && generation === signInGeneration.current) setSignInAction(null); }
  }

  function update(config: ProviderConfig) { setDraft(old => old.map(provider => provider.provider === config.provider ? config : provider)); }

  return <div className="connection-sections" aria-busy={!state || busy}>
    <div className="section-intro"><h1>Connections</h1><p>Use ChatGPT, with API providers available when you choose.</p></div>
    <section className="connection-section" aria-labelledby="chatgpt-heading">
      <h2 id="chatgpt-heading">ChatGPT</h2>
      <p className="help">Sign in in your browser. Choose an account explicitly; Harness does not rotate accounts when limits are reached.</p>
      {state?.accounts.map(account => <AccountRow key={account.accountId} account={account} active={state.activeAccount === account.accountId} disabled={busy || Boolean(signInAction)} models={modelsByAccount[account.accountId] || []} modelBusy={modelBusy === account.accountId}
        onRun={run} onReload={reload} onFindModels={async () => {
          const generation = ++modelGeneration.current;
          setModelsByAccount(old => ({ ...old, [account.accountId]: [] }));
          setModelBusy(account.accountId); setNotice(null);
          try {
            const models = await chatgptModels(account.accountId);
            if (mounted.current && modelGeneration.current === generation && activeAccount.current === account.accountId) setModelsByAccount(old => ({ ...old, [account.accountId]: models }));
          } catch (error) { if (mounted.current && modelGeneration.current === generation) setNotice({ text: error instanceof Error ? error.message : String(error), error: true }); }
          finally { if (mounted.current && modelGeneration.current === generation) setModelBusy(null); }
        }} onSelectModel={async modelId => {
          await run(() => selectChatgptModel(account.accountId, modelId), 'ChatGPT model selected.');
        }} />)}
      {!state?.accounts.length && <p className="help">No account connected.</p>}
      <div className="actions"><Button disabled={!desktop || !state || busy || Boolean(signInAction) || state.signingIn} onClick={() => void startSignIn()}>
        {state?.signingIn ? 'Waiting for sign-in…' : state?.accounts.length ? 'Add account' : 'Sign in with ChatGPT'}
      </Button>{state?.signingIn && <Button quiet disabled={signInAction === 'cancel'} onClick={() => void stopSignIn()}>{signInAction === 'cancel' ? 'Cancelling…' : 'Cancel sign-in'}</Button>}</div>
    </section>
    <section className="connection-section" aria-labelledby="fallback-heading">
      <h2 id="fallback-heading">API fallback</h2>
      <p className="help">Order providers below. Each needs your permission before it can receive meeting text. Keys stay in the Windows credential vault.</p>
      {draft.map((config, index) => <Provider key={config.provider} config={config} keyConfigured={state?.providers.find(provider => provider.config.provider === config.provider)?.keyConfigured || false}
        urlSaved={state?.providers.find(provider => provider.config.provider === config.provider)?.config.baseUrl === config.baseUrl}
        disabled={busy || !desktop} onUpdate={update} onRun={run} onFirst={index ? () => setDraft(old => [...old].reverse()) : undefined} />)}
      <div className="form-actions"><Button primary disabled={!desktop || !state || busy} onClick={() => void run(() => saveProviders(draft), 'Provider settings saved.', true)}>Save provider settings</Button></div>
    </section>
    {notice && <p className={`notice${notice.error ? ' error' : ' success'}`} role={notice.error ? 'alert' : 'status'}>{notice.text}</p>}
    {!desktop && <p className="help">Sign-in and credential storage are available in the Windows application.</p>}
  </div>;
}

function accountLabel(accountId: string) { return accountId.slice(0, 6); }

function AccountRow({ account, active, disabled, models, modelBusy, onRun, onReload, onFindModels, onSelectModel }: {
  account: AccountMetadata; active: boolean; disabled: boolean; models: ModelInfo[]; modelBusy: boolean;
  onRun: (action: () => Promise<unknown>, success: string) => Promise<void>;
  onReload: () => Promise<boolean>;
  onFindModels: () => Promise<void>; onSelectModel: (modelId: string) => Promise<void>;
}) {
  const title = account.displayName || account.email || 'ChatGPT account';
  return <div className="account-row">
    <div className="account-details"><p>{title}</p>{account.displayName && account.email && <small className="help">{account.email}</small>}<small className="account-label">Account {accountLabel(account.accountId)} · {account.signedIn ? 'Signed in' : 'Signed out'}</small></div>
    <div className="actions">
      {account.signedIn ? <Button quiet disabled={disabled || active} onClick={() => void onRun(() => selectAccount(account.accountId), 'ChatGPT account selected.')}>{active ? 'Selected' : 'Use account'}</Button> : <Button quiet disabled={disabled} onClick={() => void onRun(() => reauthorizeAccount(account.accountId), 'Continue sign-in in your browser.')}>Sign in again</Button>}
      {account.signedIn && <details className="account-options"><summary>Account options</summary><div className="actions"><Button quiet disabled={disabled} onClick={() => void onRun(() => reauthorizeAccount(account.accountId), 'Continue sign-in in your browser.')}>Sign in again</Button><Button quiet disabled={disabled} onClick={() => void onRun(async () => {
        const remoteRevocationConfirmed = await signOutAccount(account.accountId);
        if (!remoteRevocationConfirmed) { await onReload(); throw new Error('Signed out locally; remote revocation was not confirmed. Disconnect Harness in ChatGPT Settings.'); }
      }, 'ChatGPT account signed out.')}>Sign out</Button></div></details>}
    </div>
    {active && account.signedIn && <div className="account-model-panel">
      <div className="summary-row"><div><strong>ChatGPT model</strong>{account.selectedModel && <small className="help">Saved selection: {account.selectedModel}</small>}</div><Button quiet disabled={disabled || modelBusy} onClick={() => void onFindModels()}>{modelBusy ? 'Finding models…' : 'Find models'}</Button></div>
      <select className="input" aria-label={`ChatGPT model for ${title}`} disabled={disabled || !models.length || modelBusy} value={models.some(model => model.id === account.selectedModel) ? account.selectedModel || '' : ''} onChange={event => void onSelectModel(event.target.value)}>
        <option value="" disabled>Choose a discovered model</option>{models.map(model => <option key={model.id} value={model.id}>{model.name}</option>)}
      </select>
      <small className="help">Model choices are refreshed only when you choose Find models.</small>
    </div>}
  </div>;
}

function Provider({ config, keyConfigured, urlSaved, disabled, onUpdate, onRun, onFirst }: {
  config: ProviderConfig; keyConfigured: boolean; urlSaved: boolean; disabled: boolean;
  onUpdate: (config: ProviderConfig) => void; onRun: (action: () => Promise<unknown>, success: string) => Promise<void>; onFirst?: () => void;
}) {
  const keyInput = useRef<HTMLInputElement>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  useEffect(() => { setModels([]); }, [config.baseUrl]);
  const name = config.provider === 'gemini' ? 'Gemini' : 'DeepSeek';
  const prefix = config.provider;
  function saveKey() {
    if (!keyInput.current) return;
    const key = keyInput.current.value;
    keyInput.current.value = '';
    void onRun(() => setApiKey(config.provider, key), `${name} key saved.`);
  }
  return <div className="provider-fields">
    <div className="summary-row"><h3>{name}</h3>{onFirst && <Button quiet disabled={disabled} onClick={onFirst}>Use first</Button>}</div>
    <div className="field"><label htmlFor={`${prefix}-url`}>Base URL</label><input id={`${prefix}-url`} className="input" type="url" value={config.baseUrl} maxLength={2048} spellCheck={false} disabled={disabled} aria-describedby={`${prefix}-url-help`} onChange={event => onUpdate({ ...config, baseUrl: event.target.value })} />
      <small id={`${prefix}-url-help`}>Save URL changes before adding a key. A saved key only works with its original base URL.</small></div>
    <div className="field"><label htmlFor={`${prefix}-key`}>API key</label><input ref={keyInput} id={`${prefix}-key`} className="input" type="password" autoComplete="off" maxLength={16384} disabled={disabled || !urlSaved} placeholder={keyConfigured && urlSaved ? 'Key saved · enter a replacement' : 'Enter API key'} aria-describedby={`${prefix}-key-help`} />
      <small id={`${prefix}-key-help`}>{keyConfigured && urlSaved ? 'A key is saved for this URL.' : 'No key saved for this URL.'}</small>
      <div className="actions"><Button disabled={disabled || !urlSaved} onClick={saveKey}>Save key</Button>{keyConfigured && <Button quiet disabled={disabled} onClick={() => void onRun(() => deleteApiKey(config.provider), `${name} key removed.`)}>Remove key</Button>}</div></div>
    <div className="field"><label htmlFor={`${prefix}-model`}>Model</label><select id={`${prefix}-model`} className="input" disabled={disabled || !models.length} value={config.model || ''} onChange={event => onUpdate({ ...config, model: event.target.value || null })}>
      <option value="">Choose a discovered model</option>{config.model && !models.some(model => model.id === config.model) && <option value={config.model}>{config.model}</option>}{models.map(model => <option key={model.id} value={model.id}>{model.name}</option>)}
    </select><Button quiet disabled={disabled || !keyConfigured || !urlSaved} onClick={() => void onRun(async () => { setModels(await providerModels(config.provider)); }, `${name} models loaded.`)}>Discover models</Button></div>
    <label className="checkbox"><input type="checkbox" checked={config.allowFallback} disabled={disabled} onChange={event => onUpdate({ ...config, allowFallback: event.target.checked })} />Allow {name} to receive text as a fallback</label>
  </div>;
}
