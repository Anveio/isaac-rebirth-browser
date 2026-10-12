# Start the Isaac browser development workspace

1. Work in the checked-out `isaac-rebirth-browser` repository.
2. Read the managed environment status and `/etc/codex/network-policy.json` before
   using the network. Preserve injected proxy/CA settings.
3. Run `bash cloud/start.sh`. Verify HTTP readiness on port 5173.
4. Use `.venv/bin/python` for asset/binary analysis and `npm run check` for browser
   verification. Playwright has Chromium installed for headless browser tests.
5. The fixture is an environment check. Continue actual port development in the
   repository, beginning with base Rebirth resources and a small playable room.

