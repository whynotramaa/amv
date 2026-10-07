import { createHash, randomUUID } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { mkdir, readFile, rename, rm } from 'node:fs/promises';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const models = new URL('../src-tauri/models/', import.meta.url);
const manifest = JSON.parse(await readFile(new URL('manifest.json', models), 'utf8'));
if (!/^[a-z0-9._-]+$/.test(manifest.directory) || new URL(manifest.baseUrl).protocol !== 'https:') throw new Error('Invalid model manifest');
const directory = new URL(`${manifest.directory}/`, models);
await mkdir(directory, { recursive: true });

async function verified(path, file) {
  const hash = createHash('sha256');
  let bytes = 0;
  try {
    for await (const chunk of createReadStream(path)) {
      bytes += chunk.length;
      if (bytes > file.bytes) return false;
      hash.update(chunk);
    }
    return bytes === file.bytes && hash.digest('hex') === file.sha256;
  } catch (error) {
    if (error.code === 'ENOENT') return false;
    throw error;
  }
}

for (const file of manifest.files) {
  if (!/^[a-z0-9._-]+$/.test(file.name) || !/^[a-f0-9]{64}$/.test(file.sha256) || !Number.isSafeInteger(file.bytes)) throw new Error('Invalid model manifest entry');
  const target = fileURLToPath(new URL(file.name, directory));
  if (await verified(target, file)) continue;
  const temporary = `${target}.${randomUUID()}.part`;
  try {
    const response = await fetch(`${manifest.baseUrl}/${file.name}`, { signal: AbortSignal.timeout(900_000) });
    if (!response.ok || !response.body) throw new Error(`Model download failed (${response.status})`);
    let bytes = 0;
    await pipeline(Readable.fromWeb(response.body), new Transform({
      transform(chunk, _, callback) {
        bytes += chunk.length;
        callback(bytes > file.bytes ? new Error('Model exceeds pinned size') : null, chunk);
      },
    }), createWriteStream(temporary, { flags: 'wx' }));
    if (!await verified(temporary, file)) throw new Error(`Model checksum mismatch for ${file.name}`);
    await rename(temporary, target);
  } finally {
    await rm(temporary, { force: true });
  }
}
