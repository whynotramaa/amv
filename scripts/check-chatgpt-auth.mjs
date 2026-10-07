import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const path = process.argv[2] || 'src-tauri/target/x86_64-pc-windows-msvc/release/harness.exe';
const binary = readFileSync(path);
assert.equal(binary.subarray(0, 2).toString(), 'MZ', 'Expected the Windows executable');
for (const marker of [
  'dynamic_agent_client',
  'https://auth.openai.com/api/accounts/authorize',
  'https://auth.openai.com/api/accounts/oauth/token',
  'https://api.openai.com/v1',
  'ext_agent_host_id',
  'agent_name_hint',
  'chatgpt.tokens.use.direct',
]) assert(binary.includes(Buffer.from(marker)), `Missing ChatGPT integration marker: ${marker}`);
for (const marker of [
  'app_EMoamEEZ73f0CkXaXp7hrann',
  'https://auth.openai.com/oauth/authorize',
  'https://chatgpt.com/backend-api/codex',
  'codex_cli_simplified_flow',
  'codex_cli_rs',
]) assert(!binary.includes(Buffer.from(marker)), `Obsolete Codex integration in release: ${marker}`);
console.log('ChatGPT release markers passed. Browser consent and inference still need a live account check.');
