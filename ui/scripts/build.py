#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Build the static Dioxus reader or an offline native desktop bundle. No verify.sh."""
import argparse
import base64
import gzip
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

UI = Path(__file__).resolve().parents[1]
ROOT = UI.parent
PREFIX = 'rights-nobody-has-to-earn'

def run(*args, cwd=ROOT):
    subprocess.run([str(a) for a in args], cwd=cwd, check=True)

def bindgen():
    lock = (UI / 'Cargo.lock').read_text(encoding='utf-8')
    import tomllib
    version = next(p['version'] for p in tomllib.loads(lock)['package'] if p['name'] == 'wasm-bindgen')
    candidates = [os.environ.get('WASM_BINDGEN', ''), shutil.which('wasm-bindgen') or '']
    candidates.extend(str(p) for p in (Path.home() / '.cache/.wasm-pack').glob('wasm-bindgen-*/wasm-bindgen'))
    for candidate in candidates:
        if candidate and Path(candidate).is_file():
            if subprocess.check_output([candidate, '--version'], text=True).strip() == f'wasm-bindgen {version}':
                return candidate
    raise SystemExit(f'Install wasm-bindgen-cli {version}, or set WASM_BINDGEN to its executable. See README.md.')

def prepare():
    run('uv', 'run', '--script', 'tools/build_book.py', '--ui-export', 'ui/generated')
    run(sys.executable, UI / 'scripts/prepare.py')
    run('cargo', 'run', '--locked', '--release', '-p', 'book-reason', '--bin', 'precompute', '--',
        'generated/reason-inputs.json', 'generated/constitution.bin', cwd=UI)
    (UI / 'generated/constitution.bin.gz').write_bytes(gzip.compress((UI / 'generated/constitution.bin').read_bytes(), mtime=0))
    css = '\n'.join((UI / 'assets' / f).read_text(encoding='utf-8') for f in ['fonts.css','quine.css','app.css'])
    for font in (UI / 'assets/fonts').glob('*.ttf'):
        css = css.replace(f"fonts/{font.name}", 'data:font/ttf;base64,' + base64.b64encode(font.read_bytes()).decode())
    (UI / 'generated/desktop-head.html').write_text('<style>'+css+'</style><script>'+(UI / 'assets/platform.js').read_text(encoding='utf-8')+'</script>', encoding='utf-8')

def web():
    run('cargo', 'build', '--locked', '--release', '--features', 'web', '--target', 'wasm32-unknown-unknown', '--bin', 'rights-book-ui', cwd=UI)
    run('cargo', 'build', '--locked', '--release', '-p', 'book-reason', '--target', 'wasm32-unknown-unknown', '--lib', cwd=UI)
    out = UI / 'dist' / PREFIX
    if out.is_symlink():
        raise SystemExit('The generated output directory must not be a symlink')
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True, exist_ok=True)
    shutil.copytree(UI / 'assets', out / 'assets', dirs_exist_ok=True)
    licences(out/'assets/licences')
    wasm_bindgen = bindgen()
    run(wasm_bindgen, '--target', 'web', '--out-dir', out/'assets/app', '--out-name', 'rights_book_ui', UI/'target/wasm32-unknown-unknown/release/rights-book-ui.wasm')
    run(wasm_bindgen, '--target', 'web', '--out-dir', out/'assets/engine', '--out-name', 'book_reason', UI/'target/wasm32-unknown-unknown/release/book_reason.wasm')
    shutil.copy2(UI/'generated/constitution.bin.gz', out/'assets/engine/constitution.bin.gz')
    run('cargo', 'run', '--locked', '--release', '--features', 'ssg', '--bin', 'export-site', '--', out, cwd=UI)
    print(f'Static artifact: {out}')

def desktop():
    run('cargo', 'build', '--locked', '--release', '--features', 'desktop', '--bin', 'rights-book-ui', cwd=UI)
    exe = 'rights-book-ui.exe' if sys.platform == 'win32' else 'rights-book-ui'
    package_desktop(UI/'target/release'/exe, 'windows' if sys.platform == 'win32' else 'linux', bool(os.environ.get('IN_NIX_SHELL')))

def licences(out):
    out.mkdir(parents=True, exist_ok=True)
    for name in ['LICENSE-MIT','LICENSE-APACHE','LICENSING.md']:
        shutil.copy2(ROOT/name, out/name)
    shutil.copy2(ROOT/'book-1/LICENSE-CC-BY', out/'LICENSE-CC-BY')
    shutil.copy2(ROOT/'LICENSE', out/'LICENSE-CC0')

def package_desktop(binary, platform, nix_runtime=False):
    exe = 'rights-book-ui.exe' if platform == 'windows' else 'rights-book-ui'
    out = UI/'artifacts'/platform
    out.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, out/exe)
    licences(out)
    shutil.copytree(UI/'assets/fonts', out/'font-licences', ignore=shutil.ignore_patterns('*.ttf'), dirs_exist_ok=True)
    shutil.copy2(UI/'README.md', out/'README.md')
    if nix_runtime:
        shutil.copy2(UI/'shell.nix', out/'shell.nix')
        runtime = "This Linux x86_64 build uses Nix. With Nix and a nixpkgs channel installed, run from this extracted folder:\n\nnix develop --extra-experimental-features 'nix-command flakes' --impure --file shell.nix --command ./rights-book-ui\n\nThe build is not a universal AppImage or an Ubuntu binary. The Nix runtime must be available locally before offline use."
    elif platform == 'windows':
        runtime = 'Run rights-book-ui.exe on Windows x64 with Microsoft Edge WebView2 installed. This ZIP is unsigned; no installer or code-signing certificate is included.'
    else:
        runtime = 'Run ./rights-book-ui on a compatible Linux x86_64 distribution with GTK 3, WebKitGTK 4.1, libxdo and OpenSSL. This ZIP requires system libraries and is not an AppImage.'
    (out/'RUN.txt').write_text(runtime+'\n\nReading, search, fonts and reasoning resources are embedded and operate offline. Website and source download links require a connection.\n',encoding='utf-8')
    archive = UI/'artifacts'/f'rights-book-{platform}.zip'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as package:
        for file in sorted(out.rglob('*')):
            if file.is_file(): package.write(file, str(Path(f'rights-book-{platform}') / file.relative_to(out)))
    print(f'Native bundle: {archive}')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('target', choices=['prepare', 'web', 'desktop', 'package'])
    parser.add_argument('--binary', type=Path, help='Already-built native executable, for package only')
    parser.add_argument('--platform', choices=['linux','windows'], help='Target OS, for package only')
    parser.add_argument('--nix-runtime', action='store_true', help='Label an already-built Linux binary as requiring Nix')
    args = parser.parse_args()
    if args.target == 'package':
        if not args.binary or not args.platform: parser.error('package requires --binary and --platform')
        package_desktop(args.binary, args.platform, args.nix_runtime)
        return
    prepare()
    if args.target == 'web': web()
    if args.target == 'desktop': desktop()
if __name__ == '__main__': main()
