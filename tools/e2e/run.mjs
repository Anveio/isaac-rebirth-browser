import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
export const LIVE_SCRIPTS = ['test:e2e:paired', 'test:e2e:floor', 'test:e2e:weapons', 'test:e2e:boot-mt', 'test:e2e:traversal', 'test:e2e:pickups', 'test:e2e:audio', 'test:e2e:doors', 'test:e2e:regressions', 'test:e2e:tear-lifecycle', 'test:e2e:rock-blast'];
const { values } = parseArgs({ options: { output: { type: 'string' }, list: { type: 'boolean' }, suite: { type: 'string' } } });
if (values.suite && !LIVE_SCRIPTS.includes(values.suite)) throw new Error(`Unknown live game suite: ${values.suite}`);
const selectedScripts = values.suite ? [values.suite] : LIVE_SCRIPTS;

if (values.list) {
  console.log(JSON.stringify(LIVE_SCRIPTS));
} else {
  const output = resolve(values.output || join(ROOT, '.cache/e2e', `suite-${Date.now()}-${process.pid}`));
  mkdirSync(output, { recursive: true });
  const reportPath = join(output, 'report.json');
  const report = {
    schema: 1, status: 'running', startedAt: new Date().toISOString(),
    runnerSha256: createHash('sha256').update(readFileSync(fileURLToPath(import.meta.url))).digest('hex'),
    scripts: selectedScripts, suites: [],
  };
  const save = () => writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n');
  save();
  for (const script of selectedScripts) {
    console.log(`Running live game suite: ${script}`);
    const startedAt = new Date().toISOString();
    const result = await new Promise(done => {
      const child = spawn('npm', ['run', script], { cwd: ROOT, stdio: 'inherit' });
      child.once('error', error => done({ exitCode: null, signal: null, error: error.message, status: 'fail' }));
      child.once('close', (exitCode, signal) => done({ exitCode, signal, status: exitCode === 0 && !signal ? 'pass' : 'fail' }));
    });
    report.suites.push({ script, startedAt, finishedAt: new Date().toISOString(), ...result });
    save();
    console.log(`${result.status}: ${script} (exit ${result.exitCode}${result.signal ? `, signal ${result.signal}` : ''})`);
  }
  report.status = report.suites.length === selectedScripts.length && report.suites.every(suite => suite.status === 'pass') ? 'pass' : 'fail';
  report.finishedAt = new Date().toISOString();
  save();
  console.log(JSON.stringify({ status: report.status, report: reportPath, suites: report.suites }));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
