<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Book 1 reader and companion

A separate Dioxus **0.7.10** workspace for the complete Book 1 reader and its
floor, question map, delivery walkthrough and About screens. Manuscript prose
comes exclusively from `book-1/contents.json` and its 34 ordered inputs through
`tools/build_book.py`. Companion copy is session-drafted and author-approved
under the delegated approval of 2026-09-13.

## Build

Run from the repository root. Install Rust, Python 3.11+, and uv. Put Nibli at
the revision in `engine.pin` beside the book checkout with `./bootstrap.sh`.
The UI uses that same engine through path dependencies; it never edits Nibli.
The separate workspace has its own committed Cargo.lock.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
python3 ui/scripts/build.py web
python3 ui/scripts/serve.py
```

Visit <http://127.0.0.1:8789/rights-nobody-has-to-earn/>. Deploy the contents of
`ui/dist/rights-nobody-has-to-earn/` at the identical production prefix. The
local server also has an unrelated root page for checking lazy entry; it is a
test fixture, not a change to the author’s website.

`build.py` exports the book, executes all nine scenarios from the full source,
compares them with the complete compiled constitution, compiles separate
reader and engine Wasm modules, and generates every route with Dioxus SSR.
It does **not** run `verify.sh` or claim a complete constitutional verification.
`--reuse-results` is an explicit development shortcut for UI-only iteration;
omit it for final source/content builds.

Native builds use the same components and all content, fonts and reasoning
resources are embedded. No network connection is needed for native reading,
search or reasoning; public Markdown downloads and external source links open
the hosted site in the system browser and require a connection.

```sh
python3 ui/scripts/build.py desktop
```

Run that command **on each target OS**. Outputs are
`ui/artifacts/rights-book-linux.zip` and
`ui/artifacts/rights-book-windows.zip`, containing the executable and licences.
Windows needs the MSVC Rust toolchain, Visual Studio C++ build tools and the
Microsoft Edge WebView2 runtime. Windows 10/11 installations with WebView2 are
the supported Windows configuration. A Windows build is not produced by a
Linux `cargo build` invocation.
For a Windows checkout, enable Git's `core.longpaths` before cloning; the
repository's existing constitutional test paths exceed the legacy path limit.
The Windows CI job enables this before checkout.

Linux needs GTK 3, WebKitGTK **4.1**, libxdo, OpenSSL and their runtime
libraries. For a Debian/Ubuntu development host:

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
```

An optional Nix development shell supplies those dependencies and software
rendering libraries without a system package installation:

```sh
nix develop --extra-experimental-features 'nix-command flakes' --impure \
  --file ui/shell.nix --command python3 ui/scripts/build.py desktop
```

Linux ZIPs require their build environment’s compatible system libraries;
they are not universal AppImages. Build on the oldest supported distribution
for redistribution. The CI Linux build uses Ubuntu 24.04. A locally Nix-built
binary requires its Nix runtime environment; do not label it an Ubuntu build.
Each ZIP includes `RUN.txt` identifying its runtime requirements. To package an
already-built target executable (for example a Windows build made from WSL):

```sh
python3 ui/scripts/build.py package --platform windows --binary /path/to/rights-book-ui.exe
python3 ui/scripts/build.py package --platform linux --binary ui/target/release/rights-book-ui --nix-runtime
```

## Static rendering and navigation

The exporter uses Dioxus’s `VirtualDom` and SSR `pre_render` to generate full
HTML with hydration markers for **40 explicitly enumerated routes**. It also
provides the matching hydration bootstrap expected by Dioxus 0.7.10. This is a
small explicit SSG driver rather than an application server. See the
[Dioxus SSG deployment model](https://dioxuslabs.com/learn/0.7/essentials/fullstack/static_site_generation/).

Every web link is ordinary document navigation. Deep visits, refreshes,
heading fragments and reading work without JavaScript. Interactions hydrate
only after the static page is available. No service worker, iframe, backend,
MCP endpoint, chatbot or execution API is installed. A host must return a
real **404** with `404.html`, never a 200 SPA fallback.

The desktop navigation adapter handles internal links within the native app.
Browser state is restored after hydration. Progress, theme and walkthrough
position use the browser’s local storage or the OS application-data directory
on desktop (`directories::ProjectDirs`, dev/dhilipsiva/rights-book). Storage
failure leaves the current visit usable with a visible notice. Reset controls
clear the corresponding preferences. Reader scroll restoration yields to an
explicit heading fragment.

## Reasoning boundary

`scenarios.json` defines nine illustrative records, expected answers and
source references. `prepare.py` reads the **whole** constitution and the exact
existing `counterfactual/no-delivery-independence` edit from the substantive
suite. No constitutional source or pin is changed.

The pinned parser recursively emits very long conjunctions in the current
constitution, exceeding a browser’s fixed call stack. The build therefore
compiles **all 9,446 statements** into the pinned engine’s flat `LogicBuffer`
representation. The number is a build observation, not a constitutional rule.
No rule, declaration or cast fact is omitted. The canonical rule buffers and
the one declared replacement are serialized with postcard and compressed.
Native source execution and execution from this representation are compared
for every shipped scenario during each full UI build.

“Run locally” loads a dedicated browser Web Worker, engine Wasm and the full
compiled constitution. Desktop does the same work on a cancellable background
thread. Each request constructs a fresh knowledge base through the engine’s
normal admission and stratification checks, adds only that record and executes
the queries. The compiled resource contains formulas, not cached answers.
Query compilation still uses the pinned language front end. Only the declared
counterfactual substitutes a formula, on an isolated copy.

The reader never imports the reasoning crate in its web build. Initial examples
are labelled **precomputed**. **Live** appears only after complete successful
execution; a refused query is explicitly REFUSED, while an incomplete or failed
run leaves the precomputed examples labelled as such. Cancellation terminates
the browser worker or sets the engine’s native cancellation flag. Request IDs
and component lifetimes reject stale responses. Changing the record or rule
resets the live label. A derived answer is never an observed delivery event.

The current compressed full-model download is **24,907,985 bytes**; its decoded
representation is about 125 MB before the knowledge base is built. The browser
engine Wasm is about 1.4 MB, separate from the roughly 2.3 MB reader Wasm.
Resource use is substantial. The reader remains available when execution fails.
Live execution was checked in Chromium 151, Windows WebView2 153, and the local
Linux WebKitGTK build. Firefox, Safari, low-memory mobile devices and remote
network startup times have not been measured. No unsupported device is labelled
live without successful execution.

## Public reading interfaces

Every content route provides `index.md`, a visible Markdown link and alternate
metadata. `content.json` is the chapter/section citation index; `cases.json`
records the examples, source paths, canonical/counterfactual distinction and
precomputed answers. `llms-full.txt` contains the complete reader text without
editorial comments. `llms.txt` follows the [published discovery proposal](https://llmstxt.org/);
it makes no promise about adoption by agents. Search is marked `noindex` and
excluded from the section sitemap. Chapters have stable namespaced heading IDs.

Fonts and licences live in `assets/fonts/`. Source prose is CC BY 4.0, code
MIT OR Apache-2.0, and the constitutional source and case data are CC0. See
[the repository licence map](../LICENSING.md) and [font sources](assets/fonts/README.md).

## Development checks

```sh
uv run --with markdown-it-py==4.2.0 --with mdit-py-plugins==0.6.1 \
  python -m unittest tests.test_build_book
cargo test --manifest-path ui/Cargo.toml --locked -p book-reason
cargo fmt --manifest-path ui/Cargo.toml --all -- --check
uv run --with playwright==1.63.0 playwright install chromium
python3 ui/scripts/serve.py
# In another terminal:
uv run --script ui/tests/browser.py
```

Use `--browser-executable /path/to/chrome` for an installed Chromium.
`--skip-reasoning` is a partial UI-only check. Browser evidence and screenshots
are written to ignored `ui/artifacts/browser/`.

The native integration executable uses the actual platform WebView, shared
components, background engine, local search and embedded fonts, with isolated
preferences. Linux uses a visible window on the virtual display because hidden
WebKitGTK windows suppress scroll events; Windows uses a hidden WebView2 window.
It is a development binary and is not included in user bundles.

```sh
cargo build --manifest-path ui/Cargo.toml --locked --release \
  --features desktop --bin desktop-check
# Linux, inside an environment with GTK/WebKit and a software renderer:
GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a \
  ui/target/release/desktop-check /tmp/book-desktop-check.json
GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a \
  ui/target/release/desktop-check /tmp/book-desktop-check.json --resume
# Windows:
ui\target\release\desktop-check.exe ui\artifacts\desktop-check.json
ui\target\release\desktop-check.exe ui\artifacts\desktop-check.json --resume
```

CI builds the static section, Linux and Windows bundles, executes the focused
assembler and browser checks, and runs the native WebView integration checks.
Build and test outputs are artifacts; the constitutional verifier is deliberately
outside this UI workflow. Hosting integration is described separately in
[WEBSITE_HANDOVER.md](WEBSITE_HANDOVER.md).
