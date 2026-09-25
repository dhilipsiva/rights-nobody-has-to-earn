#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# /// script
# requires-python = ">=3.11"
# dependencies = ["playwright==1.63.0"]
# ///
"""Static reading, live game, failure isolation and real worker acceptance."""
import argparse
import json
import os
from pathlib import Path
import time
from urllib.parse import unquote
from playwright.sync_api import sync_playwright, expect

UI = Path(__file__).resolve().parents[1]
PREFIX = '/rights-nobody-has-to-earn/'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url', default='http://127.0.0.1:8789')
    parser.add_argument('--browser-executable')
    parser.add_argument('--skip-reasoning', action='store_true')
    parser.add_argument('--nix-browser-libraries', action='store_true')
    args = parser.parse_args()
    if args.nix_browser_libraries:
        os.environ['LD_LIBRARY_PATH'] = ':'.join(str(p/'lib') for p in Path('/nix/store').iterdir() if p.is_dir() and any(x in p.name for x in ['-nss-','-nspr-','-alsa-lib-']))
    out = UI/'artifacts/browser'; out.mkdir(parents=True, exist_ok=True)
    content = json.loads((UI/'dist/rights-nobody-has-to-earn/content.json').read_text(encoding='utf-8'))
    cases = json.loads((UI/'generated/cases.json').read_text(encoding='utf-8'))['cases']
    expected = json.loads((UI/'tests/expectations.json').read_text(encoding='utf-8'))
    routes = ['', 'read/', 'search/'] + [f'read/{Path(p["source"]).stem}/' for p in content['pages']]
    assert len(routes) == 35 and len(content['pages']) == 32 and len(cases) == 78
    for path in ['game.json', 'cases.json']:
        def inspect(value):
            if isinstance(value, dict):
                assert not {'expected','outcome','outcomes','verdicts','tally','canonical','flipped','status'} & value.keys(), (path,value.keys())
                for v in value.values(): inspect(v)
            elif isinstance(value, list):
                for v in value: inspect(v)
        inspect(json.loads((UI/'dist/rights-nobody-has-to-earn'/path).read_text(encoding='utf-8')))
    for item in content['pages']:
        assert all(s['url'] == item['canonical']+'#'+s['id'] for s in item['sections'])
        md = UI/'dist'/Path(item['markdown'].split('https://dhilipsiva.dev/')[1])
        assert md.is_file() and '<!--' not in md.read_text(encoding='utf-8')
    assert '/search/' not in (UI/'dist/rights-nobody-has-to-earn/sitemap.xml').read_text(encoding='utf-8')
    assert 'precomputed' not in (UI/'dist/rights-nobody-has-to-earn/index.md').read_text(encoding='utf-8')
    report = {'routes':35, 'reader_inputs':32, 'screens':[], 'engine':[], 'contrast':[]}
    errors = []
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=args.browser_executable)
        report['browser'] = browser.version
        context = browser.new_context(java_script_enabled=False, viewport={'width':390,'height':844})
        page = context.new_page()
        for route in routes:
            response = page.goto(args.url+PREFIX+route)
            assert response.status == 200, route
            assert page.locator('main h1').count() == 1, route
            assert page.locator('link[rel=canonical]').get_attribute('href') == 'https://dhilipsiva.dev'+PREFIX+route
            assert page.locator('meta[name=description]').get_attribute('content')
            assert page.locator('script[type="application/ld+json"]').count() == 1
            assert page.locator('link[rel=alternate][type="text/markdown"]').count() == 1
            assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), route
            if not route:
                assert page.locator('.verdict, output').count() == 0
                assert 'Gameplay requires JavaScript' in page.locator('noscript').inner_text()
            for link in page.locator('article a').all():
                href = link.get_attribute('href') or ''
                if href.startswith(PREFIX) and '#' in href:
                    dest, fragment = href.split('#',1)
                    target = UI/'dist'/dest.lstrip('/')/'index.html'
                    assert target.is_file() and f'id="{unquote(fragment)}"' in target.read_text(encoding='utf-8'), href
        for old, target in [('map/',PREFIX),('walkthrough/food-delivery/',PREFIX),('about/',PREFIX+'#dossier')]:
            response = context.request.get(args.url+PREFIX+old, max_redirects=0)
            assert response.status == 301 and response.headers['location'] == target
        assert page.goto(args.url+PREFIX+'missing/').status == 404
        page.goto(args.url+PREFIX+'read/epigraph/')
        assert page.locator('[lang=ta]').count() > 0
        page.locator('.chapter-navigation a').last.click()
        assert '/00-opening-note/' in page.url
        context.close()
        print('PASS: 35 static routes, 32 reading inputs, redirects and 404', flush=True)

        context = browser.new_context(viewport={'width':1280,'height':1000})
        page = context.new_page(); page.on('pageerror', lambda e: errors.append(str(e)))
        requests = []; page.on('request', lambda r: requests.append(r.url))
        page.goto(args.url+'/'); page.wait_for_timeout(150)
        assert not any(PREFIX in u for u in requests)
        page.goto(args.url+PREFIX+'read/'); expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        assert not any('/engine/' in u or 'engine-worker' in u for u in requests)
        started = time.monotonic(); page.goto(args.url+PREFIX)
        expect(page.locator('.game')).to_have_attribute('data-phase','ready',timeout=90000)
        report['startup_seconds'] = round(time.monotonic()-started,3)
        assert any('engine-worker' in u for u in requests) and any('constitution.bin.gz' in u for u in requests)
        assert page.locator('.verdict').count() == 0
        assert page.locator('[data-person]').count() == 12 and page.locator('[data-joint]').count() == 12
        assert page.locator('[data-dossier]').count() == 21
        worker_count = sum('engine-worker' in u for u in requests)
        for i in range(3):
            page.locator('.move-button').click()
            old_theme = page.locator('html').get_attribute('data-theme')
            before = time.monotonic(); page.get_by_role('button',name='Switch colour theme').click()
            expect(page.locator('html')).not_to_have_attribute('data-theme',old_theme)
            report.setdefault('response_ms',[]).append(round((time.monotonic()-before)*1000,1))
            expect(page.locator(f'[data-case="nell:{i}"]')).to_be_visible(timeout=90000)
        assert page.locator('[data-tally=held]').inner_text() == '3'
        assert page.locator('[data-tally=limited]').inner_text() == '2'
        assert sum('engine-worker' in u for u in requests) == worker_count, 'worker was not persistent'
        assert 'provided · TRUE' in page.locator('[data-floor=eats]').inner_text()
        history = page.evaluate("JSON.parse(localStorage.getItem('b1game:v2'))")
        assert set(history) == {'version','completed','joints','measured'} and history['completed'] == ['nell']
        # Screens contain actual executed results, with the same record in both themes.
        for theme in ['dark','light']:
            if page.locator('html').get_attribute('data-theme') != theme:
                page.get_by_role('button',name='Switch colour theme').click()
            for width in [390,768,1280]:
                page.set_viewport_size({'width':width,'height':1000}); page.evaluate('document.fonts.ready')
                assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'), (width,theme)
                small_controls = page.evaluate("""() => [...document.querySelectorAll('.book-bar a,.theme-control,.game button,.game summary,.game a,.game input')].flatMap(e => {
                    const r=e.getBoundingClientRect();
                    return r.width && r.height && (r.width<43.9 || r.height<43.9)
                        ? [{text:e.textContent.trim().slice(0,50),width:r.width,height:r.height}] : [];
                })""")
                assert not small_controls, (width,theme,small_controls)
                for state_name, selector in [('hero','.game-hero'),('nell','.play-card'),('dossier','#dossier')]:
                    page.locator(selector).scroll_into_view_if_needed()
                    name=f'{state_name}-{theme}-{width}';page.screenshot(path=str(out/(name+'.png')));report['screens'].append(name)
                contrast = page.evaluate((UI/'tests/contrast.js').read_text(encoding='utf-8'))
                report['contrast'].append({'theme':theme,'width':width,**contrast})
                assert not contrast['failures'], contrast['failures']
        page.set_viewport_size({'width':1280,'height':1000})
        page.locator('[data-person=Nell]').click()
        assert page.locator('.verdict').count() == 0
        assert 'pending' in page.locator('[data-floor=eats]').inner_text()
        page.locator('.move-button').click();expect(page.locator('[data-case="nell:0"]')).to_be_visible(timeout=90000)
        assert 'provided · FALSE' in page.locator('[data-floor=eats]').inner_text(), 'evidence leaked backwards'
        for _ in range(2):
            page.locator('.move-button').click();expect(page.locator('.game')).to_have_attribute('data-phase','complete',timeout=90000)
        assert page.locator('[data-tally=held]').inner_text() == '3', 'repeat completion added a second tally'
        page.locator('[data-joint=j-independence]').click()
        page.get_by_role('button',name='Compare both records live').click()
        expect(page.locator('[data-case="j-independence:modified"]')).to_be_visible(timeout=90000)
        assert page.locator('[data-tally=fault]').inner_text() == '1'
        page.get_by_role('button',name='Change this choice',exact=True).click()
        expect(page.locator('[data-case="j-independence:modified"]')).to_be_visible(timeout=90000)
        page.get_by_role('button',name='Restore this choice',exact=True).click()
        expect(page.locator('[data-case="j-independence:modified"]')).to_be_visible(timeout=90000)
        assert page.locator('[data-case="j-independence:canonical"] output').inner_text() == 'FALSE'
        assert page.locator('[data-tally=fault]').inner_text() == '1'
        page.get_by_role('button',name='Share progress',exact=True).click()
        expect(page.locator('#share-game')).to_be_visible()
        shared = page.locator('#share-game').input_value()
        assert '#game=' in shared and 'held' not in shared
        # Restored totals remain untrusted until replay, including imported history.
        page.reload();expect(page.locator('.restore-message')).to_contain_text('Checking saved progress')
        expect(page.locator('.game')).to_have_attribute('data-phase','ready',timeout=90000)
        page.locator('[data-person=Marisol]').click();page.locator('.move-button').click()
        expect(page.locator('[data-case="silence:0"]')).to_be_visible(timeout=90000)
        assert page.locator('.game').get_attribute('data-selection') == 'silence'
        expect(page.locator('.restore-message')).to_have_count(0,timeout=180000)
        assert page.locator('[data-tally=held]').inner_text() == '3'
        shared_context=browser.new_context();sp=shared_context.new_page();sp.goto(args.url+PREFIX+shared.split(PREFIX,1)[1])
        expect(sp.locator('.restore-message')).to_contain_text('Checking saved progress')
        expect(sp.locator('.restore-message')).to_have_count(0,timeout=180000)
        assert sp.locator('[data-tally=held]').inner_text()=='3' and sp.locator('[data-tally=fault]').inner_text()=='1'
        shared_context.close()
        print('PASS: Nell tally, evidence removal, worker reuse, joint restoration, replay priority and shared history',flush=True)
        if not args.skip_reasoning:
            for case in cases:
                now=time.monotonic();result=page.evaluate('(id)=>window.bookUI.run(id)',case['id'])
                outcome=result['outcome'];assert outcome['complete'] and outcome['id']==case['id']
                actual={v['query']:v['status'] for v in outcome['verdicts']}
                assert list(actual)==case['queries']
                for q in expected[case['id']]:assert actual[q['query']]==q['status'],(case['id'],q,actual[q['query']])
                report['engine'].append({'id':case['id'],'seconds':round(time.monotonic()-now,3),'outcome':outcome})
                print('PASS: worker',case['id'],report['engine'][-1]['seconds'],flush=True)
            source_report=UI/'artifacts/source-execution.json'
            if source_report.is_file():
                source={o['id']:o for o in json.loads(source_report.read_text(encoding='utf-8'))['outcomes']}
                assert all(row['outcome']==source[row['id']] for row in report['engine'])
        # Test response validation with modified live responses, never a runtime answer table.
        page.locator('[data-person=Nell]').click()
        live=page.evaluate("()=>window.bookUI.run('nell:0')")
        page.evaluate('(r)=>{window.realRun=window.bookUI.run;window.liveSample=r;window.bookUI.run=async()=>({...r,outcome:{...r.outcome,complete:false}})}',live)
        page.locator('.move-button').click();expect(page.locator('.game')).to_have_attribute('data-phase','incomplete')
        assert page.locator('.verdict').count()==0
        page.evaluate('()=>{window.bookUI.run=async()=>({...window.liveSample,outcome:{...window.liveSample.outcome,verdicts:[]}})}')
        page.locator('.move-button').click();expect(page.locator('.game')).to_have_attribute('data-phase','incomplete')
        assert page.locator('.verdict').count()==0
        page.evaluate('()=>{window.bookUI.run=async()=>{await new Promise(r=>setTimeout(r,700));return window.liveSample}}')
        page.locator('.move-button').click();page.locator('[data-person=Marisol]').click();page.wait_for_timeout(900)
        assert page.locator('.verdict').count()==0
        page.locator('[data-person=Nell]').click()
        page.evaluate('()=>{window.bookUI.run=async()=>{const r=structuredClone(window.liveSample);r.outcome.verdicts[1].status="FALSE";return r}}')
        page.locator('.move-button').click();expect(page.locator('[data-case="nell:0"]')).to_be_visible()
        assert page.locator('.verdict').nth(1).locator('output').inner_text()=='FALSE'
        expect(page.locator('.neutral-result')).to_be_visible()
        page.evaluate('()=>{window.bookUI.run=window.realRun}')
        page.emulate_media(reduced_motion='reduce');assert page.evaluate('getComputedStyle(document.documentElement).scrollBehavior')=='auto'
        page.goto(args.url+PREFIX+'search/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.get_by_role('searchbox').fill('independent witness');expect(page.locator('.search-results li').first).to_be_visible()
        page.get_by_role('searchbox').fill('வீழ்வே');expect(page.locator('.search-results li').first).to_be_visible()
        page.get_by_role('searchbox').fill('<script>not in manuscript</script>');expect(page.locator('.search-results li')).to_have_count(0)
        page.goto(args.url+PREFIX+'read/29-the-five-joints/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/29-the-five-joints/');page.evaluate("window.scrollTo({top:1800,behavior:'instant'})");page.wait_for_timeout(600)
        position=page.evaluate('window.scrollY');page.reload();expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/29-the-five-joints/');page.wait_for_timeout(200)
        assert abs(page.evaluate('window.scrollY')-position)<5,('scroll restore',position,page.evaluate('window.scrollY'),page.evaluate("localStorage.getItem('rights-book.preferences.v1')"))
        section=next(p for p in content['pages'] if p['number']==29)['sections'][1]['id']
        page.goto(args.url+PREFIX+'read/29-the-five-joints/#'+section)
        expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/29-the-five-joints/')
        page.wait_for_function('(id)=>Math.abs(document.getElementById(id).getBoundingClientRect().top)<80',arg=section)
        page.locator('article a[role=doc-noteref]').first.click()
        expect(page.locator('a[role=doc-backlink]').first).to_be_in_viewport()
        page.locator('a[role=doc-backlink]').first.click()
        expect(page.locator('article a[role=doc-noteref]').first).to_be_in_viewport()
        page.goto(args.url+PREFIX);expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.keyboard.press('Tab');expect(page.locator('.skip-link')).to_be_focused();page.keyboard.press('Enter');expect(page.locator('main')).to_be_focused()
        page.get_by_role('button',name='Reset game ↻',exact=True).click()
        assert page.locator('[data-tally=held]').inner_text()=='0'
        assert page.evaluate("JSON.parse(localStorage.getItem('rights-book.preferences.v1')).positions")
        context.close()
        # Startup errors on each resource boundary show no substitute answers.
        for blocked in ['**/assets/engine-worker.js','**/assets/engine/*wasm','**/assets/engine/constitution.bin.gz']:
            failure=browser.new_context();fp=failure.new_page();fp.route(blocked,lambda r:r.abort())
            fp.goto(args.url+PREFIX);expect(fp.locator('.game')).to_have_attribute('data-phase','failed',timeout=90000)
            assert fp.locator('.verdict').count()==0 and fp.locator('[data-tally=held]').inner_text()=='0'
            fp.unroute(blocked);fp.get_by_role('button',name='Retry engine startup').click()
            expect(fp.locator('.game')).to_have_attribute('data-phase','ready',timeout=90000)
            fp.locator('.move-button').click();fp.get_by_role('button',name='Cancel',exact=True).click()
            expect(fp.locator('.game')).to_have_attribute('data-phase','cancelled')
            assert fp.locator('.verdict').count()==0
            fp.locator('.move-button').click();expect(fp.locator('[data-case="nell:0"]')).to_be_visible(timeout=90000)
            failure.close()
        storage=browser.new_context();storage.add_init_script("Object.defineProperty(window,'localStorage',{get(){throw new DOMException('Blocked','SecurityError')}})")
        sp=storage.new_page();sp.goto(args.url+PREFIX);expect(sp.locator('.game')).to_have_attribute('data-phase','ready',timeout=90000)
        expect(sp.locator('.storage-notice').first).to_be_visible();sp.locator('.move-button').click();expect(sp.locator('[data-case="nell:0"]')).to_be_visible(timeout=90000)
        sp.get_by_role('button',name='Share progress',exact=True).click();expect(sp.locator('#share-game')).to_be_visible()
        storage.close();browser.close()
    assert not errors,errors
    (out/('partial-results.json' if args.skip_reasoning else 'results.json')).write_text(json.dumps(report,indent=2)+'\n', encoding='utf-8')
    print('PASS: browser acceptance; results in',out,flush=True)


if __name__=='__main__':main()
