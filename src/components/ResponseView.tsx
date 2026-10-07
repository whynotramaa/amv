import { isValidElement, useEffect, useRef, useState, type ReactNode } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { createHighlighterCore, type ThemedToken } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import { openExternal, type ResponseState } from '../bridge';
import { Button } from './primitives';

const grammars = {
  javascript: () => import('@shikijs/langs/javascript'),
  typescript: () => import('@shikijs/langs/typescript'),
  python: () => import('@shikijs/langs/python'),
  json: () => import('@shikijs/langs/json'),
  rust: () => import('@shikijs/langs/rust'),
  sql: () => import('@shikijs/langs/sql'),
  bash: () => import('@shikijs/langs/bash'),
};
const aliases: Record<string, keyof typeof grammars> = { js: 'javascript', ts: 'typescript', py: 'python', rs: 'rust', sh: 'bash', shell: 'bash' };
let highlighter: ReturnType<typeof createHighlighterCore> | undefined;
const tokenClasses: Record<string, string> = {};
function codeHighlighter() {
  // ponytail: one cached JS engine, at most seven grammars. Dispose idle grammars if measured memory warrants it.
  const css = getComputedStyle(document.documentElement);
  const color = (name: string) => css.getPropertyValue(name).trim();
  for (const [token, className] of [['--text-primary', 'code-primary'], ['--text-tertiary', 'code-comment'], ['--success', 'code-literal'], ['--accent', 'code-keyword']]) {
    tokenClasses[color(token).toLowerCase()] = className;
  }
  return highlighter ??= createHighlighterCore({
    engine: createJavaScriptRegexEngine({ forgiving: true }), langs: [],
    themes: [{ name: 'harness', type: 'dark', colors: { 'editor.background': color('--background'), 'editor.foreground': color('--text-primary') },
      tokenColors: [
        { scope: ['comment'], settings: { foreground: color('--text-tertiary') } },
        { scope: ['string', 'constant.numeric', 'constant.language'], settings: { foreground: color('--success') } },
        { scope: ['keyword', 'entity.name'], settings: { foreground: color('--accent') } },
      ],
    }],
  });
}

function safeHref(href?: string) {
  if (!href || href.length > 2048 || /[\u0000-\u001f\u007f]/.test(href)) return undefined;
  if (href.startsWith('#')) return href;
  try {
    const url = new URL(href);
    return ['https:', 'http:'].includes(url.protocol) && !url.username && !url.password ? url.href : undefined;
  } catch { return undefined; }
}

function CodeBlock({ children }: { children?: ReactNode }) {
  const code = isValidElement<{ children?: ReactNode; className?: string }>(children) ? children.props : null;
  const text = String(code?.children ?? '').replace(/\n$/, '');
  const language = code?.className?.match(/\blanguage-([\w-]+)/)?.[1];
  const [copyState, setCopyState] = useState('Copy code');
  const version = useRef(0);
  useEffect(() => { version.current++; setCopyState('Copy code'); return () => { version.current++; }; }, [text]);
  const [highlighted, setHighlighted] = useState<{ text: string; tokens: ThemedToken[][] } | null>(null);
  useEffect(() => {
    let live = true;
    const name = aliases[language || ''] || language;
    const grammar = name && Object.hasOwn(grammars, name) ? grammars[name as keyof typeof grammars] : null;
    // Large/unknown blocks remain plain text; unchanged finished blocks retain their tokenization.
    if (!grammar || text.length > 16 * 1024 || new TextEncoder().encode(text).length > 16 * 1024) { setHighlighted(null); return; }
    void codeHighlighter().then(async engine => {
      if (!live) return;
      await engine.loadLanguage(await grammar());
      if (!live) return;
      const tokens = engine.codeToTokens(text, { lang: name!, theme: 'harness' }).tokens;
      if (live) setHighlighted({ text, tokens });
    }).catch(() => { if (live) setHighlighted(null); });
    return () => { live = false; };
  }, [text, language]);
  async function copy() {
    const current = version.current;
    try {
      if (!navigator.clipboard) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(text);
      if (version.current === current) setCopyState('Copied');
    } catch { if (version.current === current) setCopyState('Copy failed'); }
  }
  return <div className="response-code"><div className="response-code-heading"><span>{language || 'Code'}</span><Button quiet onClick={() => void copy()}>{copyState}</Button></div><pre><code>{highlighted?.text === text ? highlighted.tokens.map((line, index) => <span key={index}>{line.map((token, part) => <span key={part} className={tokenClasses[token.color?.toLowerCase() || ''] || 'code-primary'}>{token.content}</span>)}{index < highlighted.tokens.length - 1 ? '\n' : ''}</span>) : text}</code></pre></div>;
}

export default function ResponseView({ response, onRetry }: { response: ResponseState; onRetry?: () => void }) {
  const [linkError, setLinkError] = useState<string | null>(null);
  const busy = response.status === 'preparing' || response.status === 'streaming';
  const failed = response.status === 'error' || response.status === 'cancelled';
  const heading = response.status === 'preparing' ? 'Preparing response' : response.status === 'streaming' ? 'Responding' : failed ? (response.status === 'cancelled' ? 'Response stopped' : 'Response failed') : response.status === 'partial' ? 'Partial response' : 'Response';
  return <section className={`response response-${response.status}`} aria-label="Meeting response" aria-busy={busy}>
    {response.status !== 'completed' && <div className="response-heading"><span role="status">{heading}</span></div>}
    {response.answer ? <div className="response-markdown"><ReactMarkdown skipHtml remarkPlugins={[remarkGfm]} components={{
      a: ({ href, children }) => {
        const safe = safeHref(href);
        return safe ? <a href={safe} rel="noopener noreferrer" onClick={safe.startsWith('#') ? undefined : event => {
          event.preventDefault(); setLinkError(null);
          void openExternal(safe).catch(() => setLinkError('Could not open the link. Try again.'));
        }}>{children}</a> : <span>{children}</span>;
      },
      img: ({ alt }) => <span className="response-note">{alt ? `Image: ${alt}` : 'Image omitted'}</span>,
      pre: CodeBlock,
      table: ({ children }) => <div className="response-table" role="region" aria-label="Response table" tabIndex={0}><table>{children}</table></div>,
    }}>{response.answer}</ReactMarkdown></div> : <p className="help">{busy ? 'The answer will appear here.' : response.error || 'No answer was returned.'}</p>}
    {response.answer && response.error && <p className="notice error" role="alert">{response.error}</p>}
    {linkError && <p className="notice error" role="alert">{linkError}</p>}
    {failed && onRetry && <Button quiet onClick={onRetry}>Retry response</Button>}
  </section>;
}
