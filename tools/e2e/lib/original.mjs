// The pinned base Rebirth original binary that every live suite pairs against.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('../../..', import.meta.url)));

export const ORIGINAL_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';

export async function assertOriginalBinary() {
  const bytes = await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'));
  assert.equal(createHash('sha256').update(bytes).digest('hex'), ORIGINAL_SHA,
    'Original depot binary differs from the pinned base Rebirth image; rerun cloud/install.sh');
}
