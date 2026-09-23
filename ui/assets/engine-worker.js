// SPDX-License-Identifier: MIT OR Apache-2.0
let engine;
self.onmessage = async ({data}) => {
  const {request} = data;
  try {
    if (data.kind === 'initialize') {
      const module = await import('./engine/book_reason.js');
      await module.default();
      const bytes = new Uint8Array(data.bytes);
      let decoded = bytes;
      if (bytes[0] === 31 && bytes[1] === 139) {
        if (!self.DecompressionStream) throw new Error('Compressed engine resources are unsupported in this browser.');
        decoded = new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer());
      }
      engine = new module.Engine(decoded);
      self.postMessage({request,ready:true});
    } else {
      if (!engine) throw new Error('Engine is not initialized');
      const outcome = JSON.parse(engine.execute(data.id));
      self.postMessage({request,outcome});
    }
  } catch(error) { self.postMessage({request,error:String(error.message || error)}); }
};
