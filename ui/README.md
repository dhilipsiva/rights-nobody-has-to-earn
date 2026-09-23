<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Book 1 reader and live companion

A standalone Dioxus 0.7.10 application for Book 1 and its executable game.
The complete reader still comes from `book-1/contents.json` and its 34 ordered
inputs through `tools/build_book.py`. The companion has a person selector,
fork track, live play card, floor panel, design joints and an authored dossier.
Companion prose is session-drafted, author-approved under delegated approval
(2026-09-13); its exact text is retained in `game.json` and `src/game.rs`.

## Build and preview

From the repository root, with Python 3.11+, uv, Rust and the Nibli revision
in `engine.pin` checked out alongside this repository:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
python3 ui/scripts/build.py web
python3 ui/scripts/serve.py
```

Open <http://127.0.0.1:8789/rights-nobody-has-to-earn/>. The static artifact is
`ui/dist/rights-nobody-has-to-earn/`. Ordinary reader pages do not start the
engine. Entering the game automatically loads it in a persistent Web Worker.
The unrelated root page in the preview server is a resource-isolation fixture.

The build exports reader text, prepares complete executable records, compiles
all constitutional statements using the pinned engine, and packages the reader
and engine as separate Wasm modules. It exports 37 HTML routes, their Markdown
counterparts, structured reading data and the redirect manifest. Compilation
supplies inputs, never displayed answers. There is no result-reuse build mode.
`verify.sh` is deliberately outside this UI task and workflow.

## Inputs and execution

`game.json` holds authored moves, query labels, interpretation categories, costs,
and dossier discussion. `case-map.json` explicitly maps each move to source
checkpoints, suite fixtures, additional premises and cross-chapter references.
It also records the delivery example's renamed constants. `prepare.py` snapshots
only persistent admitted statements before those checkpoints. Refused and
scoped acceptance controls do not become lasting facts.

The public, versioned `cases.json` contains records, queries, source references,
engine revision and the declared counterfactual transformations. It has no
outcomes or expected answers. Every step includes the selected person's floor
queries, so its floor panel never borrows another record's conclusions. Expected
answers belong only to `tests/expectations.json` and development test binaries.

The engine caches immutable compiled resources in memory, then constructs a new
knowledge base for every evaluated record. Neither replay, evidence removal,
person changes nor restored rules can retain facts from an earlier execution.
A measured joint executes its canonical and modified records before comparing
actual returned answers. Other switches open authored costs without manufacturing
an alternative verdict. A refused query remains REFUSED; an incomplete query is
not FALSE. Unexpected definitive answers stay visible and unchanged.

The game distinguishes loading, ready, running, complete, cancelled, failed and
incomplete states. Pending or failed moves have no substitute verdict. Request
and selection identifiers reject stale responses. Cancel interrupts native work
or terminates an active browser worker; retry uses a fresh knowledge base and
can reuse downloaded inputs. Run again replays a fork from its first move.

All tally tags are authored interpretations of actual completed results. Each
fork contributes once; each measured joint contributes at most one comparison
fault. Dossier badges mark an examined record, not proof of the accompanying
argument or real-world performance. The “live a life” lens is separate from
engine status. Costs remain clearly labelled authored content.

## Persistence and sharing

Game history uses `b1game:v2` in browser storage and `b1game-v2.json` in the
native application-data directory. It is separate from reader preferences.
Only completed fork IDs, joint settings and completed measured-joint IDs are
saved. Totals and verdicts are never imported. On restoration or sharing,
background execution checks each historical record before it contributes;
foreground moves take priority. The interface says “Checking saved progress”
until those checks finish. Invalid older prototype state is ignored.

Reset game clears game history alone. Storage failure leaves the visit usable
in memory. Share progress copies a history link and exposes a selectable-link
fallback. Objection links open a prefilled GitHub issue composer; nothing is
submitted automatically.

## Reading and hosting

All web links use ordinary document navigation. The static reader preserves
chapter URLs, namespaced headings, footnotes, search, saved positions, metadata,
Tamil and Markdown. Reading works without JavaScript; a no-script explanation
identifies gameplay's local-execution requirement. Desktop navigation uses the
same components with an internal adapter.

Serve the section before a host's general fallback. `redirects.json` declares
HTTP 301 redirects from `/map/` and `/walkthrough/food-delivery/` to the companion
root and `/about/` to `/#dossier`, all under the book prefix. The local server
implements GET and HEAD redirects. Other missing paths return a real 404.

Discovery includes `llms.txt`, `llms-full.txt`, `content.json`, `game.json`,
`cases.json` and `sitemap.xml`. Search is noindex and absent from the sitemap.
The llms.txt format is a discovery convention, not a guarantee of adoption.
No service worker, external execution API, chatbot or MCP service is installed.

## Native builds

```sh
python3 ui/scripts/build.py desktop
```

Run on each target OS. The Linux and Windows ZIPs live at
`ui/artifacts/rights-book-linux.zip` and `ui/artifacts/rights-book-windows.zip`.
Both embed the reader, fonts, compiled engine inputs and local reasoning.
Source/download links require a network connection; reading, search and
execution do not. The native engine initializes on a background thread when
the game opens and shares immutable resources between executions.

Linux needs GTK 3, WebKitGTK 4.1, libxdo and OpenSSL. A Nix shell is provided:

```sh
nix develop --extra-experimental-features 'nix-command flakes' --impure \
  --file ui/shell.nix --command python3 ui/scripts/build.py desktop
```

A Nix-built ZIP requires its Nix runtime; it is not a universal AppImage or an
Ubuntu binary. Windows needs the MSVC Rust toolchain, Visual Studio C++ tools
and Microsoft Edge WebView2. Enable Git long paths before a Windows checkout.
Windows bundles are unsigned. To package an existing target executable:

```sh
python3 ui/scripts/build.py package --platform windows --binary /path/to/rights-book-ui.exe
python3 ui/scripts/build.py package --platform linux --binary ui/target/release/rights-book-ui --nix-runtime
```

## Development verification

```sh
python3 -m unittest discover -s ui/tests -p 'test_*.py'
cargo test --manifest-path ui/Cargo.toml --locked --lib
cargo test --manifest-path ui/Cargo.toml --locked -p book-reason --lib
cargo fmt --manifest-path ui/Cargo.toml --all -- --check
# From ui/, compare every source/compiled record and isolation sequence:
cargo run --locked --release -p book-reason --bin precompute -- \
  generated/reason-inputs.json generated/constitution.bin tests/expectations.json
# With the local preview server running:
uv run --script ui/tests/browser.py
```

The compiler's optional third argument enables development comparison; its
results go to ignored `ui/artifacts/source-execution.json`, never to public data.
Use `--browser-executable /path/to/chrome` with an installed Chromium.
`--skip-reasoning` omits the exhaustive browser matrix and is a partial check.
Browser screenshots and results are in `ui/artifacts/browser/`.

The development-only `desktop-check` executable tests the actual native WebView,
all executable records through the production background adapter, offline
reading, navigation, fonts and restart persistence:

```sh
cargo build --manifest-path ui/Cargo.toml --locked --release \
  --features desktop --bin desktop-check
GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a \
  ui/target/release/desktop-check ui/artifacts/native/linux-game.json
GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a \
  ui/target/release/desktop-check ui/artifacts/native/linux-game.json --resume
```

On Windows run `desktop-check.exe` with the same two argument forms. The harness
is excluded from user bundles. Native test preferences are isolated by output
filename. Actual measurements, artifact limits and hosting instructions are in
[WEBSITE_HANDOVER.md](WEBSITE_HANDOVER.md).

Fonts and font licences are in `assets/fonts/`. Prose is CC BY 4.0, code is
MIT OR Apache-2.0, and constitutional inputs are CC0; see [LICENSING.md](../LICENSING.md).
