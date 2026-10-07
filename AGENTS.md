# ChatGPT integration

Read [PROVIDER_PROTOCOL.md](PROVIDER_PROTOCOL.md) before changing authentication,
model discovery, inference endpoints, or release packaging. Harness uses the
official Sign in with ChatGPT flow for a locally running personal app.

- New connections use `dynamic_agent_client`. Returning connections use their
  saved issued client ID. Preserve the installation host ID and account bindings.
- Do not borrow Codex's `app_EMoamEEZ73f0CkXaXp7hrann` client, originator, fixed
  callback port, backend endpoints, or authentication files to repair sign-in.
- Follow the current official documentation when the contract changes. Do not
  switch authentication protocols based on an unexplained browser error.
- Run the auth tests and `node scripts/check-chatgpt-auth.mjs <harness.exe>`
  before distributing a Windows build. `scripts/package.ps1` enforces the binary
  check even with `-SkipChecks`. Preserve existing installers before frontend
  builds, which clear `dist`.
- Reaching the browser login page verifies only initial request acceptance.
  Verify consent, callback, restart, model discovery, and inference with an
  eligible account before claiming the integration works end to end.
