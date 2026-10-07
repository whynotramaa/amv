import { lazy, Suspense, StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';

const MemoryView = lazy(() => import('./components/MemoryView'));
const DocumentsView = lazy(() => import('./components/DocumentsView'));
const documentsWindow = new URLSearchParams(location.search).get('view') === 'documents';
const memoryWindow = new URLSearchParams(location.search).get('view') === 'memory';
createRoot(document.getElementById('root')!).render(<StrictMode>{memoryWindow ? <main className="memory-window"><Suspense fallback={<p role="status">Loading local memory…</p>}><MemoryView /></Suspense></main> : documentsWindow ? <main className="memory-window"><Suspense fallback={<p role="status">Loading local documents…</p>}><DocumentsView /></Suspense></main> : <App />}</StrictMode>);
