// SPDX-License-Identifier: MIT OR Apache-2.0
self.onmessage = async ({data: {id, request}}) => {
  try {
    const [engine, response] = await Promise.all([
      import('./engine/book_reason.js'),
      fetch('./engine/constitution.bin.gz')
    ]);
    if (!response.ok) throw new Error(`Constitution download failed (${response.status})`);
    if (!self.DecompressionStream) throw new Error('This browser does not support the compressed local engine resource.');
    const bytes = new Uint8Array(await response.arrayBuffer());
    const decoded = bytes[0] === 31 && bytes[1] === 139
      ? await new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer()
      : bytes.buffer;
    const input = new Uint8Array(decoded);
    await engine.default();
    const outcome = JSON.parse(engine.execute(input, id));
    self.postMessage({request, outcome});
  } catch (error) {
    console.error(error.stack || error);
    self.postMessage({request, error: String(error.message || error)});
  }
};
