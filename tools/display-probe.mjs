import { chromium } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
process.env.DISPLAY ??= ':99';
fs.mkdirSync('.cache/display-probe', { recursive: true });
const browser = await chromium.launch({ headless: false, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage({ viewport: { width: 1000, height: 600 } });
  await page.setContent('<title>Isaac Display Probe</title><body style="background:#17324a;color:white;font:48px sans-serif"><h1>DISPLAY PROBE</h1><p>This is not the original game.</p><p id="result">Waiting for keyboard input</p><script>document.addEventListener("keydown", e => { document.querySelector("#result").textContent = "Received: " + e.key; document.body.style.background = "#275e32"; });</script></body>');
  await page.waitForTimeout(300);
  const window = execFileSync('xdotool', ['search', '--onlyvisible', '--name', 'Isaac Display Probe'], { encoding: 'utf8' }).trim().split('\n').at(-1);
  execFileSync('xdotool', ['windowfocus', '--sync', window]);
  execFileSync('import', ['-window', 'root', '.cache/display-probe/before.png']);
  execFileSync('xdotool', ['key', '--clearmodifiers', 'Right']);
  await page.waitForFunction(() => document.querySelector('#result').textContent === 'Received: ArrowRight');
  execFileSync('import', ['-window', 'root', '.cache/display-probe/after.png']);
  if (fs.readFileSync('.cache/display-probe/before.png').equals(fs.readFileSync('.cache/display-probe/after.png'))) throw Error('No visible screenshot change');
  console.log('Display probe passed: captured X display before/after xdotool Right; page received ArrowRight. This does not verify original gameplay.');
} finally { await browser.close(); }
