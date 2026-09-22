// SPDX-License-Identifier: MIT OR Apache-2.0
// This module contains no engine import. The worker and its resources are
// constructed only in response to the explicit Run locally action.
(() => {
  let worker = null;
  let pending = null;
  let sequence = 0;
  window.bookUI = {
    cancel() {
      sequence++;
      if (worker) worker.terminate();
      worker = null;
      if (pending) pending.reject(new Error('Cancelled'));
      pending = null;
    },
    run(id) {
      this.cancel();
      const request = sequence;
      const started = performance.now();
      return new Promise((resolve, reject) => {
        pending = {reject};
        worker = new Worker('/rights-nobody-has-to-earn/assets/engine-worker.js', {type:'module'});
        const timer = setTimeout(() => {
          if (request !== sequence) return;
          this.cancel();
        }, 180000);
        const finish = () => {
          clearTimeout(timer);
          if (worker) worker.terminate();
          worker = null;
          pending = null;
        };
        worker.onmessage = ({data}) => {
          if (request !== sequence || data.request !== request) return;
          finish();
          if (data.error) reject(new Error(data.error));
          else resolve({outcome:data.outcome, elapsed_ms: performance.now()-started});
        };
        worker.onerror = (event) => {
          if (request !== sequence) return;
          finish();
          reject(new Error(event.message || 'The local engine could not run.'));
        };
        worker.postMessage({id, request});
      });
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
