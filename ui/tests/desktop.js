// SPDX-License-Identifier: MIT OR Apache-2.0
const config = await dioxus.recv();
const sleep = ms => new Promise(resolve => setTimeout(resolve,ms));
const wait = async (predicate,label,timeout=15000) => {
  const started=Date.now();
  while(!predicate()) {if(Date.now()-started>timeout)throw new Error(`Timeout: ${label}`);await sleep(30);}
};
const click=text=>{
  const el=[...document.querySelectorAll('button,a')].find(e=>e.textContent.trim()===text);
  if(!el)throw new Error(`Control not found: ${text}`);el.click();
};
let stage='ready';const results=[];
try {
  await wait(()=>document.querySelector('#book-app')?.dataset.ready==='true','native app');
  await document.fonts.ready;
  document.querySelector('.skip-link').click();
  await wait(()=>document.activeElement?.id==='main-content','native skip focus');
  const internal=url=>new URL(url).origin===location.origin||new URL(url).hostname.endsWith('.localhost');
  const external=performance.getEntriesByType('resource').filter(r=>/^https?:/.test(r.name)&&!internal(r.name));
  if(external.length)throw new Error('Native assets made external requests');
  const fetchOriginal=window.fetch;
  window.fetch=(url,...rest)=>{
    if(/^https?:/.test(String(url))&&!internal(String(url)))throw new Error('Native check is offline: '+url);
    return fetchOriginal.call(window,url,...rest);
  };
  await wait(()=>document.querySelector('.game')?.dataset.phase==='ready','automatic native engine startup',90000);
  if(config.resume) {
    await wait(()=>document.documentElement.dataset.theme==='light','saved theme');
    await wait(()=>document.querySelector('[data-tally=held]')?.textContent==='3','replayed native history',180000);
    click('read');await wait(()=>document.querySelector('.toc-list'),'contents');click('Resume reading →');
    await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/29-the-five-joints/'),'saved chapter');
    await wait(()=>Math.abs(scrollY-1800)<5,'saved native scroll');
    dioxus.send({ok:true,persistence:true,user_agent:navigator.userAgent});
  } else {
    if(!config.reader_only) {
      click('Reset game ↻');
      for(let i=0;i<3;i++) {
        stage=`nell:${i}`;document.querySelector('.move-button').click();
        await wait(()=>document.querySelector(`[data-case="nell:${i}"]`),stage,90000);
      }
      if(document.querySelector('[data-tally=held]').textContent!=='3'||document.querySelector('[data-tally=limited]').textContent!=='2')throw new Error('Nell tally');
      click('Run again · replay from start');
      await wait(()=>document.querySelector('.game').dataset.phase==='running','native running');click('Cancel');
      await wait(()=>document.querySelector('.game').dataset.phase==='cancelled','native cancellation');
      if(document.querySelector('.verdict'))throw new Error('Cancelled replay substituted an answer');
      document.querySelector('.move-button').click();await wait(()=>document.querySelector('[data-case="nell:0"]'),'native retry',90000);
      if(!document.querySelector('[data-floor=eats]').textContent.includes('provided · FALSE'))throw new Error('Native evidence retained');
      // Every packaged record executes on the production native background adapter.
      for(const [id,expected] of Object.entries(config.expectations)) {
        stage=id;dioxus.send({kind:'execute',id});const response=await dioxus.recv();
        if(response.error)throw new Error(response.error);
        const outcome=response.outcome;
        if(!outcome.complete||outcome.id!==id)throw new Error('Incomplete native result '+id);
        for(const wanted of expected) {
          const actual=outcome.verdicts.find(v=>v.query===wanted.query);
          if(!actual||actual.status!==wanted.status)throw new Error(id+': '+wanted.query);
        }
        results.push(response);
      }
    }
  click('search');
  await wait(()=>document.querySelector('#book-search'),'search');
  const search=document.querySelector('#book-search');search.value='independent witness';search.dispatchEvent(new Event('input',{bubbles:true}));
  await wait(()=>document.querySelector('.search-results li'),'search results');
  click('read');await wait(()=>document.querySelector('.toc-list'),'contents');
  if (document.querySelectorAll('.toc-list > li').length!==config.reader_inputs) throw new Error('incomplete reader');
  click('Epigraph');await wait(()=>document.querySelector('[lang=ta]'),'Tamil reader');
  await document.fonts.load('18px "Noto Serif Tamil"');
  if (!document.fonts.check('18px "Noto Serif Tamil"')) throw new Error('Tamil font');
  // Direct chapter-to-chapter changes reuse the Reader component.
  document.querySelector('.chapter-navigation a').click();
  await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/00-opening-note/'),'next reader input');
  document.querySelector('[aria-label="Book back"]').click();
  await wait(()=>document.querySelector('[lang=ta]'),'native back');
  document.querySelector('[aria-label="Book forward"]').click();
  await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/00-opening-note/'),'native forward');
  const cross=document.querySelector('article a[href^="/rights-nobody-has-to-earn/"]');
  const destination=cross.getAttribute('href').split('#')[0];cross.click();
  await wait(()=>document.documentElement.dataset.readerReady===destination,'manuscript cross-reference');
  click('← Contents');await wait(()=>document.querySelector('.toc-list'),'contents');
  [...document.querySelectorAll('.toc-list a')].find(a=>a.getAttribute('href').endsWith('/29-the-five-joints/')).click();
  await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/29-the-five-joints/'),'last reader input');
  document.querySelector('.reader-nav details').open=true;
  const section=document.querySelector('.reader-nav a[href^="#"]');
  section.click();
  await wait(()=>Math.abs(document.getElementById(section.getAttribute('href').slice(1)).getBoundingClientRect().top)<80,'native section fragment');
  const note=document.querySelector('article a[role="doc-noteref"]');note.click();
  const noteTarget=document.getElementById(note.getAttribute('href').split('#')[1]);
  await wait(()=>noteTarget.getBoundingClientRect().top<innerHeight&&noteTarget.getBoundingClientRect().bottom>0,'native footnote');
  const backlink=noteTarget.querySelector('a[role="doc-backlink"]');backlink.click();
  await wait(()=>note.getBoundingClientRect().top<innerHeight&&note.getBoundingClientRect().bottom>0,'native footnote return');
  window.scrollTo({top:1800,behavior:'instant'});await sleep(600);
  if (document.documentElement.dataset.theme!=='light') document.querySelector('[aria-label="Switch colour theme"]').click();
  await sleep(200);
    dioxus.send({ok:true,scenarios:results.length,results,reader_inputs:config.reader_inputs,tamil:true,search:true,offline:true,user_agent:navigator.userAgent});
  }
} catch(error) {dioxus.send({ok:false,error:String(error.message)+' '+String(error.stack),stage,results,body:document.body.innerText.slice(-2500)});}
