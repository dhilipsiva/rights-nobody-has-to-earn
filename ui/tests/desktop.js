// SPDX-License-Identifier: MIT OR Apache-2.0
const config = await dioxus.recv();
const sleep = ms => new Promise(resolve => setTimeout(resolve,ms));
const wait = async (predicate, label, timeout=15000) => {
  const started = Date.now();
  while (!predicate()) { if (Date.now()-started > timeout) throw new Error(`Timeout: ${label}`); await sleep(50); }
};
const click = text => {
  const el = [...document.querySelectorAll('button,a')].find(e => e.textContent.trim() === text);
  if (!el) throw new Error(`Control not found: ${text}`);
  el.click();
};
let stage = 'ready';
const results = [];
const run = async expected => {
  stage = expected.scenario.id;
  await wait(()=>document.querySelector('.term')?.dataset.case === expected.scenario.id,expected.scenario.id);
  await wait(()=>document.querySelector('[data-result-kind=precomputed]'),'scenario reset');
  click('Run locally');
  await wait(()=>document.querySelector('[data-result-kind=live]') || document.querySelector('.run-message')?.textContent.includes('failed'), 'native reasoning',180000);
  if (!document.querySelector('[data-result-kind=live]')) throw new Error(document.querySelector('.run-message').textContent);
  const rows = [...document.querySelectorAll('.query')].map(e => [e.querySelector('code').textContent.replace(/^\? /,''),e.querySelector('output').textContent]);
  for (const [query,status] of rows) {
    const wanted=expected.outcome.verdicts.find(v=>v.query===query);
    if (!wanted || status!==wanted.status) throw new Error(`${expected.scenario.id}: ${query} => ${status}`);
  }
  if (expected.scenario.id.startsWith('floor')) {
    for (const row of document.querySelectorAll('.floor-table tbody tr')) {
      const cells=row.querySelectorAll('td'), queries=row.querySelectorAll('code');
      const owed=expected.outcome.verdicts.find(v=>v.query===queries[0].textContent+'.');
      const delivered=expected.outcome.verdicts.find(v=>v.query===queries[1].textContent+'.');
      if (owed.status!=='TRUE' || cells[2].textContent!=='true' || (cells[3].textContent==='true')!==(delivered.status==='TRUE')) throw new Error('Floor table differs from execution');
    }
  }
  return document.querySelector('.run-message').textContent;
};
try {
  await wait(()=>document.querySelector('#book-app')?.dataset.ready==='true','native app ready');
  await document.fonts.ready;
  document.querySelector('.skip-link').click();
  await wait(()=>document.activeElement?.id==='main-content','native skip focus');
  const internal=url=>new URL(url).origin===location.origin || new URL(url).hostname.endsWith('.localhost');
  const externalRequests=performance.getEntriesByType('resource').filter(r=>/^https?:/.test(r.name)&&!internal(r.name));
  if (externalRequests.length) throw new Error('Native assets made an external request: '+externalRequests.map(r=>r.name).join(', '));
  const fetchOriginal=window.fetch;
  window.fetch=(url,...rest)=>{
    if (/^https?:/.test(String(url))&&!internal(String(url))) throw new Error('Native test is offline: '+url);
    return fetchOriginal.call(window,url,...rest);
  };
  if (config.resume) {
    await wait(()=>document.documentElement.dataset.theme==='light','saved native theme');
    click('walkthrough');await wait(()=>document.querySelector('.step-link[aria-current=step] .step-num')?.textContent==='6','saved native walkthrough');
    click('read');await wait(()=>document.querySelector('.toc-list'),'contents');click('Resume reading →');
    await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/31-the-five-joints/'),'saved native chapter');
    await wait(()=>Math.abs(scrollY-1800)<5,'saved native scroll');
    dioxus.send({ok:true,persistence:true,user_agent:navigator.userAgent});
  } else {

  if (!config.reader_only) {
  const c = id => config.cases.find(c=>c.scenario.id===id);
  results.push(await run(c('floor')));
  // Cancellation while a real background task is active preserves precomputed
  // display and changing the record cannot accept its eventual response.
  click('Run locally');await wait(()=>[...document.querySelectorAll('button')].some(e=>e.textContent==='Cancel'),'cancellation button');click('Cancel');
  await wait(()=>document.querySelector('.run-message')?.textContent.includes('Cancelled'),'cancelled state');
  click('Supply food evidence →');
  results.push(await run(c('floor-evidence')));
  click('Remove the evidence ↻');
  await wait(()=>document.querySelector('.term')?.dataset.case==='floor','remove evidence');
  if (document.querySelector('[data-result-kind=live]')) throw new Error('stale live result');
  stage='walkthrough navigation';
  click('walkthrough');
  await wait(()=>document.querySelector('.step-list'),'walkthrough navigation');
  for (let i=0;i<6;i++) {
    document.querySelectorAll('.step-link')[i].click();
    results.push(await run(c(`delivery-${i}`)));
    if (i===4) {
      click('Remove the food independence condition →');
      results.push(await run(c('delivery-counterfactual')));
      click('Restore the canonical rule ↻');
      await wait(()=>document.querySelector('.term')?.dataset.case==='delivery-4','restore canonical rule');
      if (document.querySelector('[data-result-kind=live]')) throw new Error('stale counterfactual');
    }
  }
  } else {
    click('walkthrough');await wait(()=>document.querySelector('.step-list'),'walkthrough');
    document.querySelectorAll('.step-link')[5].click();
    await wait(()=>document.querySelector('.self-check'),'self-check');
  }
  click('No');
  await wait(()=>document.querySelector('.self-check [role=status]'),'self-check');
  click('search');
  await wait(()=>document.querySelector('#book-search'),'search');
  const search=document.querySelector('#book-search');search.value='independent witness';search.dispatchEvent(new Event('input',{bubbles:true}));
  await wait(()=>document.querySelector('.search-results li'),'search results');
  click('read');await wait(()=>document.querySelector('.toc-list'),'contents');
  if (document.querySelectorAll('.toc-list > li').length!==34) throw new Error('incomplete reader');
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
  [...document.querySelectorAll('.toc-list a')].find(a=>a.getAttribute('href').endsWith('/31-the-five-joints/')).click();
  await wait(()=>document.documentElement.dataset.readerReady?.endsWith('/31-the-five-joints/'),'last reader input');
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
  // The native app uses embedded reader text, fonts and reasoning; no host request is required.
  dioxus.send({ok:true,scenarios:results.length,results,reader_inputs:34,tamil:true,search:true,offline:true,user_agent:navigator.userAgent});
  }
} catch(error) { dioxus.send({ok:false,error:String(error.message)+' '+String(error.stack),stage,results,body:document.body.innerText.slice(-2500)}); }
