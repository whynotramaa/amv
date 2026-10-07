import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';

const browser = await chromium.launch({ executablePath: process.env.HARNESS_BROWSER_EXECUTABLE || undefined, headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
const errors = [];
page.on('pageerror', error => errors.push(String(error)));
await page.addInitScript(() => {
  window.isTauri = true; window.calls = []; window.cancelPicker = true; window.holdImport = false; window.failReindex = true;
  window.documents = [{ id: 21, title: 'Approved revenue', sourcePath: 'C:\\Sales\\revenue.md', sourcePolicy: 'reference', contentHash: 'a'.repeat(64), modifiedAt: 1700000000000, indexedAt: 1700000001000, indexingVersion: 1, project: 'Acme', enabled: false, chunkCount: 2, textBytes: 120 }];
  window.fixtureInvoke = async (command, args) => {
    window.calls.push({ command, args });
    if (command === 'close_documents') { window.closedDocuments = true; return; }
    if (command === 'list_documents') return { documents: window.documents, hasMore: false, next: null };
    if (command === 'check_document') return { changed: true, missing: false };
    if (command === 'import_document') {
      if (window.holdImport) await new Promise(resolve => { window.releaseImport = resolve; });
      if (window.cancelPicker) return null;
      const entry = { ...window.documents[0], id: 22, title: 'Sales notes', sourcePath: 'C:\\Sales\\notes.txt', sourcePolicy: args.sourcePolicy, project: args.project, enabled: args.enabled };
      window.documents = [entry, ...window.documents.filter(row => row.id !== 22)]; return entry;
    }
    if (command === 'set_document_enabled') {
      const entry = window.documents.find(row => row.id === args.id); entry.enabled = args.enabled; return entry;
    }
    if (command === 'reindex_document') {
      if (window.failReindex) throw new Error('Source file cannot be read. The existing index was retained.');
      const entry = window.documents.find(row => row.id === args.id); entry.indexingVersion += 1; entry.contentHash = 'b'.repeat(64); return entry;
    }
    if (command === 'search_documents') return [{ id: 31, documentId: 21, title: 'Approved revenue', sourcePath: 'C:\\Sales\\revenue.md', contentHash: 'a'.repeat(64), indexingVersion: 1, chunkIndex: 0, text: 'Approved revenue grew 12%. <script>window.badSource=true</script>' }];
    if (command === 'delete_document') {
      window.documents = window.documents.filter(row => row.id !== args.id); return { cleanupPending: true };
    }
    throw new Error('Unexpected documents fixture command ' + command);
  };
});
await page.route('**/src/main.tsx*', async route => {
  const response = await route.fetch();
  await route.fulfill({ response, body: "import { mockIPC } from '/node_modules/@tauri-apps/api/mocks.js'; mockIPC(window.fixtureInvoke,{shouldMockEvents:true});\n" + await response.text() });
});
try {
  await page.goto((process.env.HARNESS_PREVIEW_URL || 'http://127.0.0.1:1420') + '/?view=documents');
  await page.getByRole('button', { name: /Approved revenue/ }).waitFor();
  await page.getByText('Import a document', { exact: true }).click();
  const enabled = page.getByRole('checkbox', { name: 'Enable for relevant answers', exact: true });
  assert.equal(await enabled.isChecked(), false);
  assert.equal(await page.getByLabel('Source handling', { exact: true }).inputValue(), 'reference');
  await page.evaluate(() => document.querySelector('.memory-window').scrollTo(0, 0));
  await page.screenshot({ path: fileURLToPath(new URL('../.impeccable/review/documents-import-fixture.png', import.meta.url)), fullPage: true });
  await page.getByRole('button', { name: 'Choose file', exact: true }).click();
  await page.waitForFunction(() => window.calls.some(call => call.command === 'import_document'));
  await page.getByRole('button', { name: 'Choose file', exact: true }).waitFor({ state: 'visible' });
  assert.equal(await page.locator('.documents-row').count(), 1);
  assert.deepEqual(await page.evaluate(() => window.calls.find(call => call.command === 'import_document').args), { sourcePolicy: 'reference', project: null, enabled: false });
  await page.evaluate(() => { window.cancelPicker = false; window.holdImport = true; });
  await page.getByRole('button', { name: 'Choose file', exact: true }).click();
  await page.waitForFunction(() => typeof window.releaseImport === 'function');
  assert.equal(await page.getByRole('button', { name: /Approved revenue/ }).isDisabled(), true);
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('plugin:event|emit', { event: 'documents-close-requested', payload: null }));
  await page.getByText('Finish the current change before closing documents.', { exact: true }).waitFor();
  assert.equal(await page.evaluate(() => !!window.closedDocuments), false);
  await page.evaluate(() => window.releaseImport());
  await page.getByText('Document indexed locally.', { exact: true }).waitFor();
  await page.getByRole('button', { name: 'Enable for answers', exact: true }).click();
  await page.getByText('Enabled for relevant authorized answers.', { exact: true }).waitFor();
  assert.deepEqual(await page.evaluate(() => window.calls.find(call => call.command === 'set_document_enabled').args), { id: 22, enabled: true });
  await page.getByRole('button', { name: /Approved revenue/ }).click();
  await page.getByText('The source has changed.', { exact: false }).waitFor();
  assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'reindex_document').length), 0);
  await page.getByRole('button', { name: 'Reindex document', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: 'The existing index was retained.' }).waitFor();
  assert.equal(await page.locator('.documents-hash').textContent(), 'a'.repeat(64));
  assert.equal(await page.getByRole('heading', { name: 'Approved revenue', exact: true }).count(), 1);
  await page.evaluate(() => window.failReindex = false);
  await page.getByRole('button', { name: 'Reindex document', exact: true }).click();
  await page.getByText('Local index updated.', { exact: true }).waitFor();
  assert.equal(await page.locator('.documents-hash').textContent(), 'b'.repeat(64));
  const search = page.getByRole('textbox', { name: 'Search local document text', exact: true });
  await search.fill('revenue');
  assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'search_documents').length), 0);
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('heading', { name: 'Local search results', exact: true }).waitFor();
  assert.match(await page.locator('.documents-excerpt').textContent(), /<script>/);
  assert.equal(await page.evaluate(() => !!window.badSource), false);
  await page.getByText('Source details', { exact: true }).click();
  assert.equal(await page.locator('.documents-provenance .documents-hash').textContent(), 'a'.repeat(64));
  await page.locator('.documents-import > summary').click();
  await page.evaluate(() => document.querySelector('.memory-window').scrollTo(0, 0));
  await page.mouse.move(0, 0);
  await page.waitForTimeout(220);
  await page.screenshot({ path: fileURLToPath(new URL('../.impeccable/review/documents-search-fixture.png', import.meta.url)), fullPage: true });
  await page.getByRole('button', { name: 'Clear search', exact: true }).click();
  await search.fill('x'.repeat(513));
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: '512 UTF-8 bytes' }).waitFor();
  assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'search_documents').length), 1);
  await search.fill('');
  await page.getByRole('button', { name: /Sales notes/ }).click();
  await page.getByText('The source has changed.', { exact: false }).waitFor();
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
  assert.equal(await page.evaluate(() => [...document.querySelectorAll('*')].some(element => getComputedStyle(element).boxShadow !== 'none')), false);
  await page.evaluate(() => document.querySelector('.memory-window').scrollTo(0, 0));
  await page.mouse.move(0, 0);
  await page.waitForTimeout(220);
  await page.screenshot({ path: fileURLToPath(new URL('../.impeccable/review/documents-fixture.png', import.meta.url)), fullPage: true });
  await page.setViewportSize({ width: 420, height: 600 });
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
  await page.evaluate(() => document.querySelector('.memory-window').scrollTo(0, 0));
  await page.screenshot({ path: fileURLToPath(new URL('../.impeccable/review/documents-narrow-fixture.png', import.meta.url)), fullPage: true });
  await page.getByRole('button', { name: 'Delete local index', exact: true }).click();
  assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'delete_document').length), 0);
  await page.getByRole('button', { name: 'Delete permanently', exact: true }).click();
  await page.getByText('Managed-copy cleanup will retry', { exact: false }).waitFor();
  assert.equal(await page.getByRole('button', { name: /Sales notes/ }).count(), 0);
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('plugin:event|emit', { event: 'documents-close-requested', payload: null }));
  await page.waitForFunction(() => window.closedDocuments === true);
  assert.deepEqual(errors, []);
  console.log('PASS documents UI fixture: default reference/local-only, picker cancellation/import, busy-close guard, enable, source changes, reindex failure/retry, explicit plain-text search, bounds, provenance, deletion cleanup receipt, no shadows/narrow overflow. Native file/window acceptance is separate.');
} finally { await browser.close(); }
