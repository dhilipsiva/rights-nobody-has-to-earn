<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Live companion: website integration handover

Mount the standalone Book 1 section at
**https://dhilipsiva.dev/rights-nobody-has-to-earn/** using the existing website
hosting arrangement. This implementation does not change or deploy that host.
Keep integration separate from any website framework migration.

## Artifacts and source

- Repository: <https://github.com/dhilipsiva/rights-nobody-has-to-earn>
- Checkout: `/home/dhilipsiva/projects/dhilipsiva/rights-nobody-has-to-earn`
- Build instructions and architecture: [README.md](README.md)
- Static section: `ui/dist/rights-nobody-has-to-earn/`
- Native bundles: `ui/artifacts/rights-book-linux.zip` and
  `ui/artifacts/rights-book-windows.zip`
- Browser report and 18 screenshots: `ui/artifacts/browser/`
- Source/compiled execution report: `ui/artifacts/source-execution.json`
- Native execution and restart reports: `ui/artifacts/native/`
- CI definition: `.github/workflows/book-ui.yml`

Generated artifacts are ignored by Git. Use these local artifacts or rebuild;
a fresh clone does not contain the ZIPs or static output. The constitutional
source, substantive pins, engine pin and original wireframes remain unchanged.
`verify.sh` was not run and is outside this UI build and workflow.

The companion contains 12 people, 30 forks with 64 executable steps, 12 design
joints and 21 authored dossier entries. Seven joints have explicit executable
counterfactuals. Those add 14 records, making 78 executable records in total.
The seven comparisons always execute both the canonical and modified records.
No expected verdict is packaged in the application or public data.

## Mounting, routes and redirects

Copy the **contents** of `ui/dist/rights-nobody-has-to-earn/` into the host's
static directory for `/rights-nobody-has-to-earn/`, preserving nested paths.
The section includes its own navigation, styles and bundled fonts. It needs
neither an iframe nor a production application server.

Serve this section before the website's general routing fallback. Directories
resolve to `index.html`. Missing book paths return HTTP **404**, using the
provided `404.html` where supported. Do not serve the website shell or game
with a 200 response for a missing path.

| Path beneath the book prefix | Response |
| --- | --- |
| `/` | Live companion game |
| `/read/` | Complete book contents |
| `/read/<source-file-stem>/` | One of 34 complete reading inputs |
| `/search/` | Local search; noindex |
| `/map/` | HTTP 301 to `/rights-nobody-has-to-earn/` |
| `/walkthrough/food-delivery/` | HTTP 301 to `/rights-nobody-has-to-earn/` |
| `/about/` | HTTP 301 to `/rights-nobody-has-to-earn/#dossier` |

`redirects.json` exports the three redirect rules with full prefixed paths.
Implement them for GET and HEAD, preserving query strings. The supplied local
server implements the manifest. There are **37 generated routes**, plus a
separate 404 document; the three retired routes are redirects, not extra HTML
pages. Preserve trailing-slash URLs, fragments and query strings.

All routes contain Dioxus-rendered HTML before hydration. Preserve complete
reader text, titles, descriptions, canonical URLs, Open Graph metadata,
Book/Chapter structured data, namespaced headings and Markdown alternate links.
The game's initial HTML includes its introduction, controls, sources and reader
links, but no verdicts. Without JavaScript it explains that gameplay requires
local execution; the complete book remains readable and navigable.

## Automatic local execution

Entering the game automatically starts a persistent same-origin module worker
at `assets/engine-worker.js`. It loads `assets/engine/book_reason.js`, engine
Wasm and the full `assets/engine/constitution.bin.gz`. Ordinary reader pages do
not start the engine. Unrelated website pages must not import, preload or
prefetch any book resource. Add a normal navigation anchor to the section and
disable framework-specific prefetching for that link.

Loaded engine resources stay in memory. Every move constructs a fresh knowledge
base, including replays, evidence removal and counterfactual restoration.
Pending, cancelled, failed and incomplete executions produce no substitute
answer or completion. Retry executes again. Refusal remains distinct from FALSE;
unexpected definitive results remain visible unchanged. Costs, tally categories
and dossier arguments are explicitly authored interpretation, not findings.

Game history is stored separately from reader preferences under `b1game:v2`
(`b1game-v2.json` natively). Saved and shared history carries completed fork IDs,
joint settings and measured-joint IDs, never trusted totals or verdicts. On
restoration, background execution checks history before awarding dependent
progress; foreground moves take priority. Reset clears only game state.
Storage failure uses memory, and sharing provides a selectable-link fallback.
Objections open a prefilled GitHub issue composer without submitting anything.

Serve Wasm as `application/wasm`, and use appropriate JavaScript, JSON and UTF-8
Markdown MIME types. The constitution gzip can be served as a gzip resource;
the worker also handles transparent HTTP decompression. Keep reader and engine
assets separate. Current resource sizes are:

| Resource | Bytes |
| --- | ---: |
| Compressed compiled constitution and executable inputs | 25,023,214 |
| Decoded compiled input | 127,375,269 |
| Engine Wasm | 1,452,759 |
| Reader/game Wasm | 4,554,701 |

These are resource sizes, not peak memory measurements. Knowledge-base
construction also uses memory. Check the host's Content Security Policy against
Dioxus hydration, the document-evaluation bridge, module workers and Wasm.
Apply any required changes narrowly to this section and validate them. Deploy
HTML and unversioned assets atomically to avoid mixing builds. HTTP compression
of HTML, CSS, JavaScript and Wasm is useful.

## Discovery

Register `https://dhilipsiva.dev/rights-nobody-has-to-earn/sitemap.xml` in the
website's sitemap index or robots.txt. Link `llms.txt` and `content.json` from
existing agent discovery. Preserve `llms-full.txt`, `game.json`, versioned
input-only `cases.json`, and route `index.md` files. The inputs include source
references, complete records, queries and declared transformations. Development
expectations and execution reports are not public exports. Search is excluded
from the sitemap. No execution API, chatbot or MCP server is needed.

## Local preview and rebuild

With Python 3.11+, uv, Rust and the pinned sibling Nibli checkout available:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
python3 ui/scripts/build.py web
python3 ui/scripts/serve.py
```

Open <http://127.0.0.1:8789/rights-nobody-has-to-earn/>. `--port` selects another
preview port. The server root is an unrelated-host fixture for resource checks.
The build compiles inputs; it does not execute examples to manufacture displayed
answers. There is no result-reuse option. See the README for the separate
source/compiled comparison and browser acceptance commands.

Native builds use `python3 ui/scripts/build.py desktop` on each target OS. Both
embed reader text, fonts, search and reasoning inputs for offline use. The engine
initializes on a background thread when the game opens. The Linux x86_64 ZIP
requires the included Nix runtime configuration; it is not an AppImage or an
Ubuntu binary. The Windows x64 MSVC ZIP requires WebView2 and is unsigned.
Neither includes the development test harness. Source and website links still
need a network connection.

## Verification performed on 2026-09-23

- All 78 records matched source and compiled execution, including fixture-backed
  records and all seven measured joints. An additional seven-run sequence checked
  replay, evidence removal and rule restoration. The source/compiled check took
  **1,422.75 seconds** (23 minutes 42.75 seconds).
- Six input-boundary tests, five game-state tests, the engine cancellation test
  and nine reader-assembler tests passed. These cover complete source snapshots,
  scoped controls, fixture isolation, trusted source preconditions, history trust,
  Nell's tally, duplicate completion and foreground priority.
- Chromium **151.0.7922.34** matched all 78 complete worker outcomes to source
  execution. Automatic game startup took **5.017 seconds**. Individual worker
  executions took **5.910–9.488 seconds**. Theme changes during execution took
  **34.9–63.7 ms**, demonstrating that the worker kept UI interaction responsive.
- Browser checks covered Nell's live **+3 held / +2 limited**, evidence removal,
  repeated completion, counterfactual restoration, persistent worker reuse,
  saved/shared history replay and reset preserving reader preferences. Worker,
  Wasm and input-resource failures, cancellation, retry, incomplete, stale and
  unexpected responses, unavailable storage and clipboard fallback passed.
  None supplied a precomputed substitute answer.
- All 37 routes passed initial-HTML, metadata, no-JavaScript reading, fragment,
  redirect and genuine-404 checks. All 34 reading inputs, English/Tamil search,
  saved reading positions and footnote return links passed. The unrelated-host
  fixture downloaded zero book resources; reader routes loaded no engine.
- Hero, completed Nell play and dossier states were inspected at **390, 768 and
  1280px**, in both themes, with 18 screenshots. Overflow, keyboard skip/focus,
  44px control dimensions, reduced motion and solid-background text-contrast
  checks passed. Contrast checks use 4.5:1 for ordinary text and 3:1 for large
  text. This is focused
  accessibility testing, not certification.
- Native Linux WebKitGTK passed all 78 records through the production background
  adapter, with exact source-outcome equality, offline reading, Tamil, search,
  chapter/fragment navigation, cancellation and retry. Native record execution
  took **5.439–6.966 seconds**. A second process restored reader preferences,
  reading position and game history, replaying Nell before showing its tally.
  Reports: `linux-game-scenarios.json` and `linux-game.json` in the native folder.
- Native Windows x64 MSVC with Edge WebView2 **153** passed all 78 records,
  with exact source-outcome equality, offline reading, Tamil, search, native
  navigation, cancellation and retry. Executions took **10.312–14.961 seconds**.
  A second process restored reader preferences and position and replayed saved
  game history before counting it. Reports: `windows-game-scenarios.json` and
  `windows-game-persistence.json` in the native folder.

Browser and Linux timings include concurrent activity on this host. They are
local measurements, not internet download benchmarks. Firefox, Safari and live
execution on low-memory mobile hardware have not been measured. The actual
website's routing, asset policy and deployment have not been changed or tested.
Hosted CI status must be checked for the commit being integrated; these local
results do not assert a successful GitHub Actions run.

## Checks on the real host

1. Load and refresh reader, game and search URLs directly. Follow a heading
   fragment and footnote backlink. Check all three HTTP 301 redirects and a
   missing path's 404 response.
2. Disable JavaScript and verify complete reader text, metadata, contents and
   previous/next links. Confirm the game has no initial verdicts.
3. With cache disabled, open unrelated pages and hover/focus the book link;
   no book resources should download. Reader navigation loads no engine.
4. Open the game and confirm automatic engine requests. Execute Nell, block an
   engine resource, then retry. Check that failure shows no substitute result.
5. Check both themes, small-screen controls, keyboard focus, Tamil, search and
   restored history. Validate the host's CSP, MIME types and gzip handling.
6. Fetch discovery files, input JSON and Markdown directly. Report the deployed
   location, actual checks and remaining host limitations.
