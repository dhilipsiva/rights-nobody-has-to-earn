#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Local static hosting model: book prefix before fallback; genuine missing-route 404s."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse
import json
from urllib.parse import urlsplit, urlunsplit

ROOT = Path(__file__).resolve().parents[1] / 'dist'
class Handler(SimpleHTTPRequestHandler):
    def redirect(self):
        manifest = ROOT / 'rights-nobody-has-to-earn/redirects.json'
        if not manifest.is_file():
            return False
        requested = urlsplit(self.path)
        for rule in json.loads(manifest.read_text())['redirects']:
            if requested.path == rule['from']:
                target = urlsplit(rule['to'])
                self.send_response(rule['status'])
                self.send_header('Location', urlunsplit(('', '', target.path, requested.query, target.fragment)))
                self.send_header('Content-Length', '0')
                self.end_headers()
                return True
        return False
    def do_HEAD(self):
        if not self.redirect():
            super().do_HEAD()
    def do_GET(self):
        if self.redirect():
            return
        if self.path == '/':
            content = b'<!doctype html><html lang="en"><title>Host test page</title><h1>Unrelated host page</h1><a href="/rights-nobody-has-to-earn/">Book 1</a></html>'
            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(content)))
            self.end_headers()
            self.wfile.write(content)
        else:
            super().do_GET()
    def send_error(self, code, message=None, explain=None):
        path = ROOT / 'rights-nobody-has-to-earn/404.html'
        if code == 404 and path.is_file():
            body = path.read_bytes()
            self.send_response(404)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            if self.command != 'HEAD': self.wfile.write(body)
        else:
            super().send_error(code, message, explain)
    def log_message(self, *_): pass
if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--port', type=int, default=8789)
    args = parser.parse_args()
    server = ThreadingHTTPServer(('127.0.0.1', args.port), partial(Handler, directory=str(ROOT)))
    print(f'http://127.0.0.1:{args.port}/rights-nobody-has-to-earn/', flush=True)
    server.serve_forever()
