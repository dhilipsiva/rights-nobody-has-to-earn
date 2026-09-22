#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# /// script
# requires-python = ">=3.11"
# dependencies = ["playwright==1.63.0"]
# ///
"""Real-browser acceptance checks against the prefix-mounted static artifact."""
import argparse
import json
import os
from pathlib import Path
import time
from urllib.parse import unquote
from playwright.sync_api import sync_playwright, expect

UI=Path(__file__).resolve().parents[1]
PREFIX='/rights-nobody-has-to-earn/'

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url',default='http://127.0.0.1:8789')
    parser.add_argument('--browser-executable')
    parser.add_argument('--skip-reasoning',action='store_true')
    parser.add_argument('--nix-browser-libraries',action='store_true',help='Use already-installed Nix NSPR/NSS/ALSA libraries on minimal Linux hosts')
    args=parser.parse_args()
    if args.nix_browser_libraries:
        os.environ['LD_LIBRARY_PATH']=':'.join(str(p/'lib') for p in Path('/nix/store').iterdir() if p.is_dir() and any(x in p.name for x in ['-nss-','-nspr-','-alsa-lib-']))
    out=UI/'artifacts/browser';out.mkdir(parents=True,exist_ok=True)
    content=json.loads((UI/'dist/rights-nobody-has-to-earn/content.json').read_text())
    cases=json.loads((UI/'generated/cases.json').read_text())['cases']
    assert [p['number'] for p in content['pages'] if p['number']] == list(range(1,32))
    for item in content['pages']:
        assert all(s['url']==item['canonical']+'#'+s['id'] for s in item['sections'])
        md=UI/'dist'/Path(item['markdown'].split('https://dhilipsiva.dev/')[1])
        assert md.is_file() and '<!--' not in md.read_text()
    assert '<!--' not in (UI/'dist/rights-nobody-has-to-earn/llms-full.txt').read_text()
    assert '/search/' not in (UI/'dist/rights-nobody-has-to-earn/sitemap.xml').read_text()
    routes=['','map/','walkthrough/food-delivery/','about/','read/','search/']+[f'read/{Path(p["source"]).stem}/' for p in content['pages']]
    report={'routes':len(routes),'reader_inputs':len(content['pages']),'screens':[],'engine':[],'contrast':[]}
    errors=[]
    with sync_playwright() as p:
        browser=p.chromium.launch(executable_path=args.browser_executable)
        report['browser']=browser.version
        context=browser.new_context(java_script_enabled=False,viewport={'width':390,'height':844})
        page=context.new_page()
        titles=set()
        for route in routes:
            response=page.goto(args.url+PREFIX+route)
            assert response.status==200,route
            assert page.locator('main h1').count()==1,route
            title=page.title();assert title not in titles,(route,title);titles.add(title)
            assert page.locator('link[rel=canonical]').get_attribute('href')=='https://dhilipsiva.dev'+PREFIX+route
            assert page.locator('meta[name=description]').get_attribute('content'),route
            assert page.locator('script[type="application/ld+json"]').count()==1
            assert page.locator('link[rel=alternate][type="text/markdown"]').count()==1
            assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),route
            for link in page.locator('article a').all():
                href=link.get_attribute('href') or ''
                if href.startswith(PREFIX) and '#' in href:
                    dest,fragment=href.split('#',1)
                    target=UI/'dist'/dest.lstrip('/')/'index.html'
                    assert target.is_file(),href
                    assert f'id="{unquote(fragment)}"' in target.read_text(),href
        assert page.goto(args.url+PREFIX+'missing-page/').status==404
        page.goto(args.url+PREFIX+'read/epigraph/')
        assert page.locator('[lang=ta]').count()>0
        page.locator('.chapter-navigation a').last.click()
        assert '/00-opening-note/' in page.url
        context.close()
        print('PASS: all initial HTML routes, no-JavaScript navigation, metadata, text targets and 404',flush=True)
        context=browser.new_context(viewport={'width':1280,'height':1000})
        page=context.new_page();page.on('pageerror',lambda e:errors.append(str(e)))
        requests=[];page.on('request',lambda r:requests.append(r.url))
        page.goto(args.url+'/');page.wait_for_timeout(250)
        assert not any(PREFIX in u for u in requests),'host page eagerly downloads book assets'
        page.get_by_role('link',name='Book 1',exact=True).click()
        expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        assert not any('/engine/' in u or 'engine-worker' in u for u in requests),'engine eagerly loaded'
        page.get_by_role('button',name='Supply food evidence').click()
        expect(page.locator('.term')).to_have_attribute('data-case','floor-evidence')
        assert page.locator('.floor-table tbody tr').first.locator('td').last.inner_text()=='true'
        page.get_by_role('button',name='Remove the evidence').click()
        expect(page.locator('.term')).to_have_attribute('data-case','floor')
        assert page.locator('.floor-table tbody tr').first.locator('td').last.inner_text()=='not derivable'
        for theme in ['dark','light']:
            if theme=='light':page.get_by_role('button',name='Switch colour theme').click()
            for width in [390,768,1280]:
                page.set_viewport_size({'width':width,'height':1000})
                for route,name in [('', 'floor'),('map/','map'),('walkthrough/food-delivery/','walkthrough'),('about/','about')]:
                    page.goto(args.url+PREFIX+route);expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
                    expect(page.locator('html')).to_have_attribute('data-theme',theme)
                    page.evaluate('document.fonts.ready');page.wait_for_timeout(250)
                    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth+1'),(route,width)
                    if width==1280:
                        contrast=page.evaluate((UI/'tests/contrast.js').read_text())
                        report['contrast'].append({'page':name,'theme':theme,**contrast})
                        assert not contrast['failures'],(route,theme,contrast['failures'])
                    page.screenshot(path=str(out/f'{name}-{theme}-{width}.png'),full_page=True)
                    report['screens'].append(f'{name}-{theme}-{width}')
        page.goto(args.url+PREFIX+'search/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.get_by_role('searchbox').fill('independent witness');expect(page.locator('.search-results li').first).to_be_visible()
        page.get_by_role('searchbox').fill('வீழ்வே');expect(page.locator('.search-results li').first).to_be_visible()
        page.get_by_role('searchbox').fill('<script>not in manuscript</script>');expect(page.locator('.search-results li')).to_have_count(0)
        page.goto(args.url+PREFIX+'read/31-the-five-joints/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/31-the-five-joints/');page.evaluate("window.scrollTo({top:1800,behavior:'instant'})");page.wait_for_timeout(600)
        position=page.evaluate('window.scrollY');page.reload();expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/31-the-five-joints/');page.wait_for_timeout(200)
        assert abs(page.evaluate('window.scrollY')-position)<5,('scroll restore',position,page.evaluate('window.scrollY'),page.evaluate("localStorage.getItem('rights-book.preferences.v1')"))
        section=next(p for p in content['pages'] if p['number']==31)['sections'][1]['id']
        page.goto(args.url+PREFIX+'read/31-the-five-joints/#'+section)
        expect(page.locator('html')).to_have_attribute('data-reader-ready',PREFIX+'read/31-the-five-joints/')
        page.wait_for_function('(id)=>Math.abs(document.getElementById(id).getBoundingClientRect().top)<80',arg=section)
        page.locator('article a[role=doc-noteref]').first.click()
        expect(page.locator('a[role=doc-backlink]').first).to_be_in_viewport()
        page.locator('a[role=doc-backlink]').first.click()
        expect(page.locator('article a[role=doc-noteref]').first).to_be_in_viewport()
        page.goto(args.url+PREFIX+'map/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.locator('.question-row').first.click();expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.goto(args.url+PREFIX+'map/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        assert '1 ' in page.locator('.progress-count').inner_text(),'saved question visit'
        page.get_by_role('button',name='Clear visits').click();expect(page.locator('.progress-count')).to_contain_text('0 ')
        page.goto(args.url+PREFIX+'walkthrough/food-delivery/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        page.locator('.step-link').nth(5).click();page.get_by_role('button',name='No',exact=True).click();expect(page.locator('.self-check [role=status]')).to_contain_text('does not')
        page.reload();expect(page.locator('.step-link').nth(5)).to_have_attribute('aria-current','step')
        page.get_by_role('button',name='Reset walkthrough').click();expect(page.locator('.term')).to_have_attribute('data-case','delivery-0')
        page.emulate_media(reduced_motion='reduce');assert page.evaluate('getComputedStyle(document.documentElement).scrollBehavior')=='auto'
        page.goto(args.url+PREFIX);page.keyboard.press('Tab');expect(page.locator('.skip-link')).to_be_focused();page.keyboard.press('Enter');expect(page.locator('main')).to_be_focused()
        print('PASS: responsive screens, theme, local search, progress, restoration, self-check, keyboard and reduced motion',flush=True)
        # Real failure and cancel paths: preserve labelled precomputed results.
        page.route('**/assets/engine/**',lambda route:route.abort())
        page.get_by_role('button',name='Run locally',exact=True).click();expect(page.locator('.run-message')).to_contain_text('failed',timeout=15000)
        expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
        page.unroute('**/assets/engine/**')
        page.get_by_role('button',name='Retry locally',exact=True).click();page.get_by_role('button',name='Cancel',exact=True).click()
        expect(page.locator('.run-message')).to_contain_text('Cancelled')
        page.get_by_role('button',name='Supply food evidence').click();expect(page.locator('.term')).to_have_attribute('data-case','floor-evidence')
        expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
        # An incomplete response is never displayed as FALSE or labelled live.
        page.evaluate("() => {window.realRun=window.bookUI.run;window.bookUI.run=async id=>({outcome:{id,counterfactual:false,complete:false,verdicts:[]},elapsed_ms:1});}")
        page.get_by_role('button',name='Run locally',exact=True).click()
        expect(page.locator('.run-message')).to_contain_text('incomplete')
        expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
        # Simulate a response arriving after evidence removal, even if an adapter
        # ignores cancellation: the component must also reject its stale ID.
        page.evaluate("() => {window.bookUI.run=async id=>{await new Promise(r=>setTimeout(r,500));return {outcome:{id,counterfactual:false,complete:true,verdicts:[]},elapsed_ms:500}};}")
        page.get_by_role('button',name='Retry locally',exact=True).click()
        page.get_by_role('button',name='Remove the evidence').click();page.wait_for_timeout(700)
        expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
        assert page.locator('.floor-table tbody tr').first.locator('td').last.inner_text()=='not derivable'
        page.evaluate('() => {window.bookUI.run=window.realRun;}')
        if not args.skip_reasoning:
            for case in cases:
                ident=case['scenario']['id']
                if ident.startswith('floor'):
                    page.goto(args.url+PREFIX);expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
                    if ident=='floor-evidence':page.get_by_role('button',name='Supply food evidence').click()
                else:
                    page.goto(args.url+PREFIX+'walkthrough/food-delivery/');expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
                    index=4 if case['scenario']['counterfactual'] else int(ident.split('-')[-1])
                    page.locator('.step-link').nth(index).click()
                    if case['scenario']['counterfactual']:page.get_by_role('button',name='Remove the food independence').click()
                expect(page.locator('.term')).to_have_attribute('data-case',ident)
                page.evaluate('() => {window.realRun=window.bookUI.run;window.bookUI.run=async id=>{const result=await window.realRun.call(window.bookUI,id);window.observedOutcome=result.outcome;return result};}')
                started=time.monotonic();page.get_by_role('button',name='Run locally',exact=True).click()
                # Theme responds while the worker is compiling/executing.
                old_theme=page.locator('html').get_attribute('data-theme');page.get_by_role('button',name='Switch colour theme').click()
                expect(page.locator('html')).not_to_have_attribute('data-theme',old_theme)
                expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','live',timeout=180000)
                assert page.evaluate('window.observedOutcome')==case['outcome'],ident
                actual=page.locator('.query').evaluate_all("rows=>rows.map(e=>({query:e.querySelector('code').textContent.replace(/^\\? /,''),status:e.querySelector('output').textContent}))")
                for row in actual:
                    expected=next(v for v in case['outcome']['verdicts'] if v['query']==row['query'])
                    assert expected['status']==row['status'],(ident,row,expected)
                duration=round(time.monotonic()-started,2);report['engine'].append({'id':ident,'seconds':duration});print('PASS:',ident,duration,'seconds',flush=True)
                if ident=='floor-evidence':
                    page.get_by_role('button',name='Remove the evidence').click()
                    expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
                    assert page.locator('.floor-table tbody tr').first.locator('td').last.inner_text()=='not derivable'
            # Canonical restoration and backwards movement reset the visible result.
            page.get_by_role('button',name='Restore the canonical rule').click();expect(page.locator('.term')).to_have_attribute('data-case','delivery-4')
            expect(page.locator('[data-result-kind]')).to_have_attribute('data-result-kind','precomputed')
            page.locator('.step-link').nth(0).click();expect(page.locator('.query').first.locator('output')).to_have_text('FALSE')
        context.close()
        storage=browser.new_context();storage.add_init_script("Object.defineProperty(window,'localStorage',{get(){throw new DOMException('Blocked','SecurityError')}})")
        page=storage.new_page();page.goto(args.url+PREFIX);expect(page.locator('#book-app')).to_have_attribute('data-ready','true')
        expect(page.locator('.storage-notice')).to_be_visible();page.get_by_role('button',name='Supply food evidence').click();expect(page.locator('.term')).to_have_attribute('data-case','floor-evidence')
        storage.close();browser.close()
    assert not errors,errors
    (out/('partial-results.json' if args.skip_reasoning else 'results.json')).write_text(json.dumps(report,indent=2)+'\n')
    print('PASS: browser acceptance; results in',out,flush=True)
if __name__=='__main__':main()
