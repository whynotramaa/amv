// Build-time acquisition only. The installed application never downloads models.
import { createHash, randomUUID } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { readFile, rename, rm } from 'node:fs/promises';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const directory = new URL('../src-tauri/models/', import.meta.url);
const manifest = JSON.parse(await readFile(new URL('manifest.json', directory), 'utf8'));
if (!/^ggml-[a-z0-9._-]+\.bin$/.test(manifest.filename)
    || !/^[a-f0-9]{64}$/.test(manifest.sha256)
    || !Number.isSafeInteger(manifest.bytes) || manifest.bytes < 1
    || new URL(manifest.url).protocol !== 'https:') throw new Error('Invalid model manifest');
const target = fileURLToPath(new URL(manifest.filename, directory));

async function verified(path) {
  const hash = createHash('sha256');
  let bytes = 0;
  try {
    for await (const chunk of createReadStream(path)) {
      bytes += chunk.length;
      if (bytes > manifest.bytes) return false;
      hash.update(chunk);
    }
    return bytes === manifest.bytes && hash.digest('hex') === manifest.sha256;
  } catch (error) {
    if (error.code === 'ENOENT') return false;
    throw error;
  }
}

if (!await verified(target)) {
  const temporary = `${target}.${randomUUID()}.part`;
  try {
    const response = await fetch(manifest.url, { signal: AbortSignal.timeout(120_000) });
    if (!response.ok || !response.body) throw new Error(`Model download failed (${response.status})`);
    let bytes = 0;
    await pipeline(Readable.fromWeb(response.body), new Transform({
      transform(chunk, _, callback) {
        bytes += chunk.length;
        callback(bytes > manifest.bytes ? new Error('Model exceeds pinned size') : null, chunk);
      },
    }), createWriteStream(temporary, { flags: 'wx' }));
    if (!await verified(temporary)) throw new Error('Model checksum mismatch');
    await rename(temporary, target);
  } finally {
    await rm(temporary, { force: true });
  }
}
console.log(`Verified local model: ${manifest.filename} (${manifest.bytes} bytes)`);
