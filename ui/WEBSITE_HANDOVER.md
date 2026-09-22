<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Paste-ready prompt for the website’s Claude session

Integrate the completed standalone Book 1 section into **https://dhilipsiva.dev/rights-nobody-has-to-earn/**.
Keep this task separate from any migration of the website’s framework. Use the
existing hosting arrangement unless an actual hosting constraint requires a change.

The implementation belongs to the book repository:

- Repository: <https://github.com/dhilipsiva/rights-nobody-has-to-earn>
- Local checkout: `/home/dhilipsiva/projects/dhilipsiva/rights-nobody-has-to-earn`
- Source and build instructions: `ui/README.md`
- Ready static directory: `ui/dist/rights-nobody-has-to-earn/`
- Linux native ZIP: `ui/artifacts/rights-book-linux.zip`
- Windows native ZIP: `ui/artifacts/rights-book-windows.zip`
- Browser results and 24 screenshots: `ui/artifacts/browser/`
- Native scenario and restart results: `ui/artifacts/native/`
- Build checks: `.github/workflows/book-ui.yml` (web, Ubuntu 24.04, Windows).

Generated outputs are ignored by Git. Use the local artifacts or rebuild; do not
expect `dist/` or ZIP files to appear in a fresh clone. Do not modify the original
wireframes, constitutional source, substantive pins, or engine pin for integration.
Do not run `verify.sh` as part of this UI task.

## What to mount

Copy the **contents** of `ui/dist/rights-nobody-has-to-earn/` into the host’s static
directory for `/rights-nobody-has-to-earn/`. Preserve every nested path. The build
already includes the book navigation, fonts and styles. It has no website header
or footer, and needs no iframe or production application server.

Serve this section **before the website’s general routing fallback**. Existing
directories resolve to their `index.html`; a missing book path must return HTTP
**404**, with the provided `404.html` if the host supports a custom error body.
Never send the website shell or the companion home with status 200 for a missing
book page. Normalize directory URLs to a trailing slash without losing fragments
or query strings.

Preserve these routes:

| Path beneath the prefix | Content |
| --- | --- |
| `/` | Companion floor |
| `/map/` | 26 questions and glossary |
| `/walkthrough/food-delivery/` | Six-step walkthrough |
| `/about/` | Explanation, sources and licences |
| `/read/` | Complete contents |
| `/read/<source-file-stem>/` | 34 reading inputs, individually generated |
| `/search/` | Local search, marked noindex |

All 40 routes contain complete Dioxus-generated HTML before hydration. Preserve
their titles, descriptions, canonical URLs, Open Graph metadata, Book/Chapter
structured data, namespaced heading IDs, and Markdown alternate links. Reading
and document navigation work without JavaScript. Do not replace this output with
a client-only router.

## Entry and discovery

Add a normal navigation anchor to `/rights-nobody-has-to-earn/` with a clear label
such as “The Rights Nobody Has to Earn”. Disable any framework-specific link
prefetching for this destination. Other website routes must not import, preload,
prefetch or download this section’s CSS, fonts, JavaScript, Wasm or constitution.

Register `https://dhilipsiva.dev/rights-nobody-has-to-earn/sitemap.xml` in the
website’s sitemap index and/or robots.txt sitemap directives. Link the section’s
`llms.txt` and `content.json` from the website’s existing agent discovery files.
Preserve `llms-full.txt`, `cases.json`, and each route’s `index.md`. These are static
reading/citation interfaces. llms.txt is a discovery proposal, not a guarantee
that agents use it. Add no MCP server, chatbot or execution API.

## Asset handling

Keep assets under this prefix. Use correct MIME types, especially
`application/wasm`, JavaScript, JSON and UTF-8 Markdown. “Run locally” creates a
same-origin module worker at `assets/engine-worker.js`; only then does it load
`assets/engine/book_reason.js`, its Wasm, and `constitution.bin.gz`. Keep those
resources separate from the initial reader download. The gzip file can be served
as a gzip resource; the worker also handles transparent HTTP decompression.

If the host has a Content Security Policy, check it against Dioxus’s hydration
bootstrap, document-evaluation bridge, module worker and WebAssembly. Apply any
necessary compatibility changes narrowly to this section and validate them;
do not silently weaken the entire website’s policy. Deploy the section atomically
and revalidate unversioned HTML/assets together to avoid mixing build versions.
Ordinary HTTP compression of HTML, CSS, JS and Wasm is useful.

The compiled full constitution is **24,907,985 compressed bytes**, approximately
125 MB decoded before knowledge-base construction, and the engine Wasm is about
1.4 MB. The reader Wasm is about 2.3 MB. A failed, cancelled or incomplete run must
retain clearly labelled **precomputed** examples. Only successful execution is
labelled **live**. Keep the distinction between a formal conclusion and an
observed event, and the canonical/counterfactual distinction.

## Rebuild commands

From the book repository, with Rust, Python 3.11+ and uv installed:

```sh
./bootstrap.sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
python3 ui/scripts/build.py web
python3 ui/scripts/serve.py
```

The local URL is <http://127.0.0.1:8789/rights-nobody-has-to-earn/>. The build uses
Dioxus 0.7.10 with an explicit SSR static exporter and hydration bootstrap. It
executes nine curated scenarios against the whole source constitution, then
compares every result with execution from all compiled constitution statements.
It does not run the complete constitutional verifier. `--reuse-results` is only
for UI-only development iteration; omit it for source/content release builds.

Native builds run `python3 ui/scripts/build.py desktop` on each target OS; see the
README for system WebView dependencies. The local Linux ZIP is a **Nix-built
x86_64 binary**, includes its Nix shell, and requires that runtime environment.
It is not an AppImage or an Ubuntu binary. The Windows ZIP contains an actual
x64 MSVC build, requires WebView2, and is unsigned. Both embed the book, fonts,
search and reasoning resources for offline use. Native releases need not be
published to integrate the web section.

## Validation already performed in the book repository

- Nine assembler tests passed, covering all 34 inputs in manuscript order,
  chapter links, footnotes, tables, Tamil and Markdown export without editorial
  comments. Public JSON citation links, sitemap exclusions and the declared
  counterfactual export were checked.
- All 40 routes passed initial-HTML, no-JavaScript reading, canonical/metadata,
  heading-fragment and genuine-404 checks under the production prefix. Browser
  refresh, saved position, fragment navigation and footnote return links passed.
- Four companion screens were inspected at 390, 768 and 1280 pixels in both
  themes; 24 screenshots were captured. Keyboard skip/focus, reduced motion and
  solid-background text contrast checks passed. This is focused testing, not an
  accessibility certification.
- Search (including Tamil), question progress, walkthrough/self-check, resets,
  theme restoration and unavailable browser storage passed.
- Source and compiled execution matched for all nine scenarios. The final Linux
  source/compiled preparation took **193.19 seconds**. No rules or pins changed.
  Windows source/compiled execution also matched the Linux output exactly and
  took **245.42 seconds** in a separate native MSVC run.
- Chromium **151.0.7922.34** matched every precomputed verdict for all nine live
  scenarios. Measured local startup/execution was **8.00–12.11 seconds**, including
  an **8.93-second** first scenario, with concurrent build activity on this host.
  The UI remained responsive during execution. Resource failure, retry,
  cancellation, incomplete responses, stale responses, evidence removal,
  backwards steps and counterfactual restoration passed.
- Native Linux WebKitGTK and Windows WebView2 **153** passed all nine scenarios,
  offline reading/search/fonts, and native chapter navigation. Both restored
  theme, step, chapter and scroll position in a second process.
- The local unrelated-host fixture downloaded zero book resources. Initial book
  navigation downloaded no engine worker, engine Wasm or constitution.

The actual dhilipsiva.dev host has **not** been changed or validated by the book
implementation. Firefox, Safari and low-memory mobile live execution have not
been measured. The local timings are not internet download benchmarks. Hosted
CI status should be checked for the commit being integrated; local build/test
results above do not assert a completed GitHub Actions run.

## Checks to perform on the real website

1. Load and refresh every generated route directly, including
   `/read/epigraph/`, `/read/31-the-five-joints/` and `/read/method/`.
   Check a heading fragment, a footnote/backlink, and a missing page returning 404.
2. Disable JavaScript: inspect page source, metadata and complete chapter text;
   follow contents, chapter and previous/next links.
3. Open unrelated website pages with cache disabled, then hover/focus the book
   link. Record that no book resource downloads until document navigation.
4. Enter the section and inspect network requests: no engine resources until
   “Run locally”. Run an example successfully and test a blocked engine request.
5. Check mobile layouts, both themes, Tamil, keyboard focus, search and restored
   progress. Confirm the host’s policies do not break hydration or the worker.
6. Fetch the sitemap, agent indexes, Markdown counterparts and JSON directly.
   Check source attribution, licences, MIME types and canonical URLs.

Report the concrete website changes, deployed artifact location, checks run and
any remaining host limitations. Do not combine this integration with a framework
migration.
