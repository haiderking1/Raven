#!/usr/bin/env python3
"""Check local handbook links and optionally validate Lua samples with Raven."""
import argparse
from html.parser import HTMLParser
import json
import re
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

class Page(HTMLParser):
    def __init__(self, path):
        super().__init__()
        self.links, self.ids = [], set()
        self.feed(path.read_text())
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            assert attrs['id'] not in self.ids, f'Duplicate ID: {attrs["id"]}'
            self.ids.add(attrs['id'])
        for name in ('href', 'src'):
            if name in attrs:
                self.links.append(attrs[name])

def verify(binary=None):
    pages = {path.resolve(): Page(path) for path in ROOT.glob('*.html')}
    assert len(pages) == len(json.loads((ROOT / 'content/sections.json').read_text()))
    for path, page in pages.items():
        for link in page.links:
            assert '://' not in link, f'Unexpected network dependency: {link}'
            name, _, fragment = link.partition('#')
            target = (path.parent / name).resolve() if name else path
            assert target.exists(), f'{path.name}: missing {link}'
            if fragment:
                assert fragment in pages[target].ids, f'{path.name}: missing anchor {link}'
    for stylesheet in (ROOT / 'assets').rglob('*.css'):
        for link in re.findall(r'url\(["\']?([^"\')]+)', stylesheet.read_text()):
            assert '://' not in link, f'Unexpected stylesheet network dependency: {link}'
            assert (stylesheet.parent / link).exists(), f'{stylesheet.name}: missing {link}'
    samples = 0
    if binary:
        binary = Path(binary).resolve()
        with tempfile.TemporaryDirectory(prefix='raven-handbook-') as directory:
            candidate = Path(directory) / 'raven.lua'
            for source in (ROOT / 'content').glob('*.json'):
                data = json.loads(source.read_text())
                if not isinstance(data, dict):
                    continue
                for item in data['blocks']:
                    if item['type'] != 'code' or item.get('language', 'Lua') != 'Lua':
                        continue
                    candidate.write_text(item['text'])
                    result = subprocess.run([binary, '--check-config', candidate], capture_output=True, text=True, timeout=5)
                    assert result.returncode == 0, f'{source.name}: {result.stderr}'
                    samples += 1
    print(f'{len(pages)} pages: local links and anchors valid. {samples} Lua examples checked.')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--raven', help='Raven binary for optional sample validation; examples require their terminal/launcher programs installed')
    verify(parser.parse_args().raven)
