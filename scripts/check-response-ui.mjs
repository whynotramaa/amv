import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { readFile } from 'node:fs/promises';
const previewUrl = process.env.HARNESS_PREVIEW_URL || 'http://127.0.0.1:1420';
const reviewDir = fileURLToPath(new URL('../.impeccable/review/', import.meta.url));
const browser = await chromium.launch({executablePath:process.env.HARNESS_BROWSER_EXECUTABLE || undefined,headless:true,args:['--no-sandbox']});
const page = await browser.newPage({viewport:{width:520,height:600}});
const errors=[]; const imageRequests=[]; const policyErrors=[];
page.on('console', message=>{if(/Content Security Policy|violates.*directive/i.test(message.text())) policyErrors.push(message.text());});
page.on('pageerror', e=>{ errors.push(String(e)); console.error(String(e)); });
page.on('request', request=>{if(request.url().includes('tracking.invalid')) imageRequests.push(request.url());});
await page.addInitScript(()=>{
 window.isTauri=true;
 window.calls=[]; window.opened=[];
 const settings={sendMode:'automatic',responseMode:'summary',customInstruction:'',overlayShortcut:'Ctrl+Space',sendShortcut:'Ctrl+Shift+Enter',launchOnLogin:false};
 window.fixtureMeeting={status:'active',meetingId:7,title:'Acme discovery',startedAt:Date.now(),transcript:[{id:101,source:'system',startMs:0,text:'Revenue increased 12% year over year.'}],error:null};
 window.fixtureResponse=null;
 window.fixtureInvoke=async(cmd,args)=>{
  window.calls.push({cmd,args});
  if(cmd==='get_app_state')return {settings,audioAvailable:true,chatgptConnected:true,inferenceAvailable:true,shortcutError:null};
  if(cmd==='get_meeting_state')return window.fixtureMeeting;
  if(cmd==='get_response_state')return window.fixtureResponse;
  if(cmd==='audio_devices')return [];
  if(cmd==='start_chat') { return await new Promise(resolve=>{window.resolveChat=resolve;}); }
  if(cmd==='ask_meeting'){
   if(window.failNextAsk) {window.failNextAsk=false;return await new Promise((resolve,reject)=>{window.rejectAsk=reject;});}
   const initial={requestId:11,attemptId:1,meetingId:7,status:'preparing',answer:'',provider:null,model:null,contextOmitted:false,error:null,usage:null,userText:args.text};
   window.fixtureResponse={...initial,status:'completed',provider:'chatgpt',model:'test-model',answer:'**Revenue:** grew 12%. Inline `USD`.\n\n| Metric | Change |\n| --- | --- |\n| Revenue | 12% |\n\n- [x] Confirmed\n\n```python\nprint("<script>unsafe()</script>")\n```\n\n[Report](https://example.com/report) [Unsafe](javascript:alert(1)) ![Tracking](https://tracking.invalid/pixel)\n\n<img src="https://tracking.invalid/raw">',usage:{inputTokens:20,outputTokens:10,totalTokens:null},contextOmitted:true};
   await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'response-state',payload:window.fixtureResponse});
   await new Promise(resolve=>setTimeout(resolve,100));
   return initial; // Older IPC result must not undo completed event.
  }
  if(cmd==='send_meeting_speech') {
   window.retryAttempt=(window.retryAttempt??1)+1;
   window.fixtureResponse={requestId:12,attemptId:window.retryAttempt,meetingId:window.fixtureMeeting.meetingId,status:window.retryAttempt===2?'streaming':'completed',answer:window.retryAttempt===2?'':'Recovered request',provider:'chatgpt',model:'test-model',contextOmitted:false,error:null,usage:null,userText:'saved question'};
   return window.fixtureResponse;
  }
  if(cmd==='saved_meetings')return {meetings:[{id:7,title:'Recovered meeting',startedAt:Date.now()-100000,endedAt:Date.now()-1000,status:'interrupted'}],hasMore:false};
  if(cmd==='saved_transcript')return {segments:[],next:null};
  if(cmd==='restore_saved_meeting'){window.fixtureMeeting={status:'idle',meetingId:7,title:'Recovered meeting',startedAt:Date.now()-100000,transcript:[],error:null}; return window.fixtureMeeting;}

  if(cmd==='open_external'){window.opened.push(args.url);return;}
  if(cmd==='stop_meeting'){window.fixtureMeeting={...window.fixtureMeeting,status:'idle'};return window.fixtureMeeting;}
  if(cmd==='start_meeting'){
   window.fixtureMeeting={status:'active',meetingId:8,title:args.title,startedAt:Date.now(),transcript:[],error:null};
   await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'meeting-state',payload:window.fixtureMeeting});
   return {...window.fixtureMeeting,status:'starting'};
  }
  if(cmd==='cancel_response') {
   window.fixtureResponse={...window.fixtureResponse,status:'cancelled',error:null};
   await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'response-state',payload:window.fixtureResponse}); return;
  }

  throw new Error('Unexpected fixture command '+cmd);
 };
 Object.defineProperty(navigator,'clipboard',{value:{writeText:async text=>{window.copied=text;}}});
});
const mockPrefix = "import { mockIPC } from '/node_modules/@tauri-apps/api/mocks.js'; mockIPC(window.fixtureInvoke,{shouldMockEvents:true});\n";
if (process.env.HARNESS_PRODUCTION_FIXTURE) {
 const config = JSON.parse(await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
 const index = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
 const entry = index.match(/src="([^"]+\.js)"/)[1];
 await page.route(previewUrl + '/**', async route => {
  const path = new URL(route.request().url()).pathname;
  const file = path === '/' ? new URL('../dist/index.html', import.meta.url) : path === '/node_modules/@tauri-apps/api/mocks.js' ? new URL('../node_modules/@tauri-apps/api/mocks.js', import.meta.url) : new URL('../dist' + path, import.meta.url);
  const body = await readFile(file);
  const contentType = path === '/' ? 'text/html' : path.endsWith('.js') ? 'text/javascript' : path.endsWith('.css') ? 'text/css' : path.endsWith('.woff2') ? 'font/woff2' : 'application/octet-stream';
  await route.fulfill({body: path === entry ? mockPrefix + body.toString() : body, contentType, headers: {'Content-Security-Policy':config.app.security.csp}});
 });
} else await page.route('**/src/main.tsx*',async route=>{
 const response=await route.fetch();
 await route.fulfill({response,body:mockPrefix+await response.text()});
});
await page.goto(previewUrl);
await page.locator('.meeting-context > summary').filter({hasText:'Acme discovery'}).waitFor();
await page.getByLabel('Include microphone context',{exact:true}).check();
await page.getByLabel('Message',{exact:true}).fill('What changed?');
await page.getByLabel('Message',{exact:true}).press('Enter');
await page.getByText('Some earlier meeting context was omitted.').waitFor();
assert.equal(await page.getByText('Preparing response',{exact:true}).count(),0);
assert.equal(await page.locator('.response-markdown table').count(),1);
assert.equal(await page.locator('.response-markdown img').count(),0);
assert.equal(await page.locator('.response-markdown > p code').innerText(),'USD');
await page.waitForFunction(()=>document.querySelector('.response-code code span span'));
assert(await page.locator('.response-code code span span').count()>0);
assert.equal(await page.locator('.response-code [style]').count(),0);
assert(await page.locator('.response-code .code-literal').count()>0);
assert(await page.locator('.response-code .code-literal').first().evaluate(element=>getComputedStyle(element).color !== getComputedStyle(element.closest('code')).color));
assert.equal(await page.locator('a[href^="javascript:"]').count(),0);
assert.equal(await page.evaluate(()=>window.calls.find(c=>c.cmd==='ask_meeting').args.includeMicrophone),true);
await page.getByRole('button',{name:'Copy code',exact:true}).click();
await page.getByRole('button',{name:'Copied',exact:true}).waitFor();
assert.equal(await page.evaluate(()=>window.copied),'print("<script>unsafe()</script>")');
await page.getByRole('link',{name:'Report',exact:true}).click();
assert.deepEqual(await page.evaluate(()=>window.opened),['https://example.com/report']);
assert.equal(imageRequests.length,0);
await page.evaluate(()=>document.querySelector('.view').scrollTo(0,0));
await page.screenshot({path:`${reviewDir}/response-fixture.png`});
await page.setViewportSize({width:420,height:360});
assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
await page.screenshot({path:`${reviewDir}/response-narrow-fixture.png`});
await page.evaluate(async()=>{
 window.failedAttempt={requestId:12,attemptId:1,meetingId:7,status:'error',answer:'',provider:'chatgpt',model:'test-model',contextOmitted:false,error:'Offline',usage:null,userText:'saved question'};
 await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'response-state',payload:window.failedAttempt});
});
await page.getByRole('button',{name:'Retry response',exact:true}).click();
await page.getByText('Responding',{exact:true}).waitFor();
await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'response-state',payload:window.failedAttempt}));
assert.equal(await page.getByText('Responding',{exact:true}).count(),1);
await page.getByRole('button',{name:'Stop',exact:true}).click();
await page.getByText('Response stopped',{exact:true}).waitFor();
await page.getByRole('button',{name:'Retry response',exact:true}).click();
await page.getByText('Recovered request',{exact:true}).waitFor();
await page.getByRole('button',{name:'Stop listening',exact:true}).click();
assert.equal(await page.getByRole('button',{name:'Send speech',exact:true}).isVisible(),true);
await page.getByRole('button',{name:'Start listening',exact:true}).click();
const automatic = page.getByRole('checkbox',{name:'Allow automatic sending of new system speech during this meeting',exact:true});
assert.equal(await automatic.isChecked(),false);
await page.getByLabel('Meeting title',{exact:true}).fill('Next meeting');
await automatic.check();
await page.getByRole('checkbox',{name:'I have permission to capture audio for this meeting.',exact:true}).check();
await page.getByRole('button',{name:'Start local meeting',exact:true}).click();
await page.locator('.meeting-context > summary').filter({hasText:'Next meeting'}).waitFor();
assert.equal(await page.evaluate(()=>window.calls.find(c=>c.cmd==='start_meeting').args.remoteConsent),true);
assert.equal(await page.locator('.response').count(),0);
await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'response-state',payload:window.fixtureResponse}));
assert.equal(await page.locator('.response').count(),0);
await page.getByRole('button',{name:'Stop listening',exact:true}).click();
await page.getByRole('button',{name:'Settings',exact:true}).click();
await page.getByRole('button',{name:'Saved meetings',exact:true}).click();
await page.getByRole('button',{name:/Recovered meeting/}).click();
await page.getByRole('button',{name:'Open in assistant',exact:true}).click();
await page.locator('.meeting-context > summary').filter({hasText:'Recovered meeting'}).waitFor();
assert.equal(await page.evaluate(()=>window.calls.find(c=>c.cmd==='restore_saved_meeting').args.meetingId),7);
assert.equal(await page.getByText('Some earlier meeting context was omitted.').count(),0);
assert.equal(await page.getByRole('button',{name:'Send speech',exact:true}).isVisible(),true);
assert.equal(await page.getByRole('combobox',{name:'Output mode for next meeting',exact:true}).isVisible(),true);
await page.getByText('Output mode changes apply to the next meeting or new chat.',{exact:true}).waitFor();
// A failed request from another meeting must not restore its text or error.
await page.evaluate(()=>{window.failNextAsk=true;});
await page.getByLabel('Message',{exact:true}).fill('Old meeting question');
await page.getByLabel('Message',{exact:true}).press('Enter');
await page.waitForFunction(()=>typeof window.rejectAsk==='function');
await page.evaluate(async()=>{
 window.fixtureMeeting={status:'active',meetingId:9,title:'New live context',startedAt:Date.now(),transcript:[],error:null};
 await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'meeting-state',payload:window.fixtureMeeting});
});
await page.locator('.meeting-context > summary').filter({hasText:'New live context'}).waitFor();
await page.evaluate(()=>window.rejectAsk(new Error('Obsolete request failed.')));
await page.waitForFunction(()=>!document.querySelector('textarea[aria-label="Message"]').disabled);
assert.equal(await page.getByLabel('Message',{exact:true}).inputValue(),'');
assert.equal(await page.getByText('Error: Obsolete request failed.',{exact:true}).count(),0);
assert.equal(await page.getByText('Old meeting question',{exact:true}).count(),0);
// Switching an unsent draft also clears it, and a late chat creation cannot
// replace a meeting selected while that command was pending.
await page.getByLabel('Message',{exact:true}).fill('Unsent previous-context draft');
await page.evaluate(async()=>{
 window.fixtureMeeting={status:'idle',meetingId:null,title:null,startedAt:null,transcript:[],error:null};
 await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'meeting-state',payload:window.fixtureMeeting});
});
await page.waitForFunction(()=>!document.querySelector('.meeting-context'));
assert.equal(await page.getByLabel('Message',{exact:true}).inputValue(),'');
const asksBeforeLateChat=await page.evaluate(()=>window.calls.filter(call=>call.cmd==='ask_meeting').length);
await page.getByLabel('Message',{exact:true}).fill('Question for pending chat');
await page.getByLabel('Message',{exact:true}).press('Enter');
await page.waitForFunction(()=>typeof window.resolveChat==='function');
await page.evaluate(async()=>{
 window.fixtureMeeting={status:'active',meetingId:10,title:'Selected while chat starts',startedAt:Date.now(),transcript:[],error:null};
 await window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'meeting-state',payload:window.fixtureMeeting});
});
await page.locator('.meeting-context > summary').filter({hasText:'Selected while chat starts'}).waitFor();
await page.evaluate(()=>window.resolveChat({status:'idle',meetingId:11,title:'Obsolete new chat',startedAt:Date.now(),transcript:[],error:null}));
await page.waitForFunction(()=>!document.querySelector('textarea[aria-label="Message"]').disabled);
assert.equal(await page.locator('.meeting-context > summary').filter({hasText:'Selected while chat starts'}).count(),1);
assert.equal(await page.getByText('Obsolete new chat',{exact:true}).count(),0);
assert.equal(await page.getByLabel('Message',{exact:true}).inputValue(),'');
assert.equal(await page.evaluate(()=>window.calls.filter(call=>call.cmd==='ask_meeting').length),asksBeforeLateChat);

assert.deepEqual(errors,[]);
assert.deepEqual(policyErrors,[]);
console.log('Response fixture passed: stale replies/events, question MIC opt-in, GFM/code/copy, safe links/images, narrow overflow, fresh meeting consent, stopped/saved speech sending, next-meeting mode scope, late question/chat context guards. UI-only evidence.');
await browser.close();
