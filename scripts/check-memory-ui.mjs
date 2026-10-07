import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
const browser = await chromium.launch({executablePath:process.env.HARNESS_BROWSER_EXECUTABLE || undefined,headless:true,args:['--no-sandbox']});
const page = await browser.newPage({viewport:{width:900,height:700}});
const errors=[];
page.on('pageerror', error=>errors.push(String(error)));
await page.addInitScript(()=>{
 window.isTauri=true; window.calls=[]; window.failSave=true; window.failDelete=true;
 window.memories=[{id:21,title:'Acme revenue',body:'Revenue grew 12%, from the approved annual report.',category:'project',project:'Acme',enabled:false,createdAt:1700000000000,updatedAt:1700000000000,source:'manual'}];
 window.fixtureInvoke=async(command,args)=>{
  window.calls.push({command,args});
  if(command==='close_memory'){window.closedMemory=true;return;}
  if(command==='list_memories')return {entries:window.memories,hasMore:false,next:null};
  if(command==='search_memories')return window.memories.filter(entry=>entry.title.toLowerCase().includes(args.query.toLowerCase()));
  if(command==='save_memory'){
   if(window.failSave)throw new Error('Disk is full.');
   const entry={...args.input,id:args.id??22,createdAt:1700000000000,updatedAt:1700000001000,source:'manual'};
   window.memories=[entry,...window.memories.filter(row=>row.id!==entry.id)]; return entry;
  }
  if(command==='delete_memory'){
   if(window.failDelete)throw new Error('Database is busy.');
   window.memories=window.memories.filter(row=>row.id!==args.id);return;
  }
  throw new Error('Unexpected memory fixture command '+command);
 };
});
await page.route('**/src/main.tsx*',async route=>{
 const response=await route.fetch();
 await route.fulfill({response,body:"import { mockIPC } from '/node_modules/@tauri-apps/api/mocks.js'; mockIPC(window.fixtureInvoke,{shouldMockEvents:true});\n"+await response.text()});
});
try {
 await page.goto((process.env.HARNESS_PREVIEW_URL||'http://127.0.0.1:1420')+'/?view=memory');
 await page.getByRole('button',{name:'Add memory',exact:true}).click();
 const enabled=page.getByRole('checkbox',{name:'Enable for relevant answers',exact:true});
 assert.equal(await enabled.isChecked(),false);
 await page.getByLabel('Title',{exact:true}).fill('Trace latency');
 await page.getByLabel('Memory text',{exact:true}).fill('Approved latency: 35 ms. Source: internal measurement.');
 await page.getByRole('button',{name:'Save memory',exact:true}).click();
 await page.getByRole('alert').filter({hasText:'Disk is full.'}).waitFor();
 assert.equal(await page.getByLabel('Title',{exact:true}).inputValue(),'Trace latency');
 await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'memory-close-requested',payload:null}));
 await page.getByRole('button',{name:'Discard changes and close',exact:true}).waitFor();
 await page.getByRole('button',{name:'Keep editing',exact:true}).click();
 assert.equal(await page.getByLabel('Title',{exact:true}).inputValue(),'Trace latency');
 assert.equal(await page.evaluate(()=>!!window.closedMemory),false);
 await page.getByRole('button',{name:/Acme revenue/}).click();
 await page.getByText('You have unsaved changes.',{exact:false}).waitFor();
 await page.getByRole('button',{name:'Keep editing',exact:true}).click();
 assert.equal(await page.getByLabel('Title',{exact:true}).inputValue(),'Trace latency');
 await page.evaluate(()=>window.failSave=false);
 await enabled.check();
 await page.getByRole('button',{name:'Save memory',exact:true}).click();
 await page.getByText('Saved locally.',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(call=>call.command==='save_memory').at(-1).args.input.enabled),true);
 await page.getByRole('button',{name:/Acme revenue/}).click();
 await page.getByRole('button',{name:'Delete memory',exact:true}).click();
 await page.getByRole('button',{name:'Delete permanently',exact:true}).click();
 await page.getByRole('alert').filter({hasText:'Database is busy.'}).waitFor();
 assert.equal(await page.getByLabel('Title',{exact:true}).inputValue(),'Acme revenue');
 await page.evaluate(()=>window.failDelete=false);
 await page.getByRole('button',{name:'Delete permanently',exact:true}).click();
 await page.getByText('Memory deleted.',{exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:/Acme revenue/}).count(),0);
 await page.getByLabel('Search memories',{exact:true}).fill('Trace');
 await page.getByRole('button',{name:'Search',exact:true}).click();
 await page.getByRole('button',{name:'Clear search',exact:true}).waitFor();
 assert.equal(await page.locator('.memory-row').count(),1);
 await page.getByRole('button',{name:/Trace latency/}).click();
 assert.equal(await enabled.isChecked(),true);
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 assert.equal(await page.evaluate(()=>[...document.querySelectorAll('*')].some(element=>getComputedStyle(element).boxShadow!=='none')),false);
 await page.screenshot({path:fileURLToPath(new URL('../.impeccable/review/memory-fixture.png',import.meta.url)),fullPage:true});
 await page.setViewportSize({width:420,height:600});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 await page.screenshot({path:fileURLToPath(new URL('../.impeccable/review/memory-narrow-fixture.png',import.meta.url)),fullPage:true});
 await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'memory-close-requested',payload:null}));
 await page.waitForFunction(()=>window.closedMemory===true);
 assert.deepEqual(errors,[]);
 console.log('PASS memory UI fixture: default-off permission, failed-save draft retention, discard guard, enable, provenance, delete failure/retry, explicit search, no shadows/narrow overflow. Native storage/window acceptance is separate.');
} finally {await browser.close();}
