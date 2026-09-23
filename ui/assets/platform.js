// SPDX-License-Identifier: MIT OR Apache-2.0
// Game entry starts this service. Reader routes never call start().
(() => {
  let worker = null, ready = false, starting = null, resource = null, sequence = 0;
  let download = null;
  const pending = new Map();
  const fail = error => {
    if (worker) worker.terminate();
    worker = null; ready = false; starting = null;
    for (const p of pending.values()) { clearTimeout(p.timer); p.reject(error); }
    pending.clear();
  };
  const request = (message, transfer=[]) => new Promise((resolve,reject) => {
    const request = ++sequence;
    const timer = setTimeout(() => fail(new Error('Local execution timed out. Retry when resources are available.')),180000);
    pending.set(request,{resolve,reject,timer});
    try { worker.postMessage({...message,request},transfer); } catch(error) { fail(error); }
  });
  window.bookUI = {
    cancel() { if (pending.size || starting) { if(download) download.abort(); fail(new Error('Cancelled')); } },
    start() {
      if (ready) return Promise.resolve({ready:true});
      if (starting) return starting;
      const current = new Worker('/rights-nobody-has-to-earn/assets/engine-worker.js',{type:'module'});
      worker = current;
      worker.onmessage = ({data}) => {
        if (worker !== current) return;
        const p=pending.get(data.request);
        if (!p) return;
        clearTimeout(p.timer); pending.delete(data.request);
        if (data.error) { p.reject(new Error(data.error)); fail(new Error(data.error)); }
        else p.resolve(data);
      };
      worker.onerror = event => { if (worker===current) fail(new Error(event.message || 'Engine worker failed')); };
      // Keep downloaded input in memory for cancellation/retry; never cache a verdict.
      if (!resource) {
        download=new AbortController();
        const timer=setTimeout(()=>download?.abort(),180000);
        resource = fetch('/rights-nobody-has-to-earn/assets/engine/constitution.bin.gz',{signal:download.signal}).then(r=>{
        if (!r.ok) throw new Error(`Constitution download failed (${r.status})`);
        return r.arrayBuffer();
      }).catch(error=>{resource=null;throw error;}).finally(()=>{clearTimeout(timer);download=null;});
      }
      starting=(async()=>{
        try {
          const bytes=(await resource).slice(0);
          if (worker!==current) throw new Error('Cancelled');
          await request({kind:'initialize',bytes},[bytes]);
          if (worker!==current) throw new Error('Cancelled');
          ready=true; starting=null;
          return {ready:true};
        } catch(error) { if(worker===current) fail(error); throw error; }
      })();
      return starting;
    },
    async run(id) {
      const started=performance.now();
      await this.start();
      const data=await request({kind:'run',id});
      return {outcome:data.outcome, elapsed_ms:performance.now()-started};
    },
    async reading(config, send) {
      if (this.stopReading) this.stopReading();
      let restored = false;
      let active = true;
      delete document.documentElement.dataset.readerReady;
      if (!config.desktop) history.scrollRestoration = 'manual';
      const save = () => {
        if (active && restored) send({kind:'scroll', path:config.path, y:window.scrollY});
      };
      let timer;
      const scroll = () => { clearTimeout(timer); timer=setTimeout(save, 180); };
      const click = event => {
        const a = event.target.closest('a');
        if (!a || !config.desktop || event.button || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
        const href = a.getAttribute('href');
        const localFragment = href?.startsWith('#') ? href.slice(1)
          : href?.startsWith(config.path+'#') ? href.slice(config.path.length+1) : null;
        if (localFragment !== null) {
          // The native WebView's navigation guard blocks same-document URLs.
          // Handle section and footnote links within the current reader instead.
          event.preventDefault();
          try {
            const target = document.getElementById(decodeURIComponent(localFragment));
            if (target) {
              target.scrollIntoView({behavior:'instant'});
              if (!target.hasAttribute('tabindex')) target.setAttribute('tabindex','-1');
              target.focus({preventScroll:true});
              save();
            }
          } catch (_) { /* A malformed fragment does not navigate away. */ }
          return;
        }
        if (href && href.startsWith('/rights-nobody-has-to-earn/')) {
          event.preventDefault();
          save();
          send({kind:'navigate', path:href, y:0});
        }
      };
      window.addEventListener('scroll', scroll, {passive:true});
      window.addEventListener('pagehide', save);
      document.addEventListener('click', click);
      this.stopReading = () => {
        active = false;
        clearTimeout(timer);
        window.removeEventListener('scroll', scroll);
        window.removeEventListener('pagehide', save);
        document.removeEventListener('click', click);
      };
      await document.fonts.ready;
      const restore = () => {
        if (!active) return;
        const fragment = config.desktop ? config.fragment : location.hash.slice(1);
        let target;
        try { target = fragment && document.getElementById(decodeURIComponent(fragment)); } catch (_) { /* Malformed fragments do not prevent reading. */ }
        if (target) target.scrollIntoView({behavior:config.desktop?'instant':'auto'});
        else window.scrollTo({top:config.y || 0,behavior:'instant'});
        restored = true;
        document.documentElement.dataset.readerReady = config.path;
      };
      // Native WebViews can suspend animation frames while their window opens.
      // The font promise already gives us the layout needed to restore there.
      if (config.desktop) restore(); else requestAnimationFrame(restore);
    }
  };
})();
