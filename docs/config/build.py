#!/usr/bin/env python3
"""Render the offline configuration handbook. Uses only Python's standard library."""
from html import escape
import json
import re
from pathlib import Path
from string import Template

ROOT = Path(__file__).resolve().parent

def block(item):
    kind = item['type']
    if kind == 'code':
        language = escape(item.get('language', 'Lua'))
        return f'<div class="sample"><div class="sample-bar"><span>{language}</span><button type="button" class="copy" hidden>Copy</button></div><pre><code>{escape(item["text"])}</code></pre><p class="copy-status" role="status"></p></div>'
    if kind == 'table':
        headers = ''.join(f'<th scope="col">{escape(v)}</th>' for v in item['headers'])
        rows = []
        for row in item['rows']:
            assert len(row) == len(item['headers'])
            cells = ''.join(f'<td>{escape(v)}</td>' for v in row)
            rows.append(f'<tr>{cells}</tr>')
        return f'<div class="table-scroll" tabindex="0" role="region" aria-label="{escape(item["label"], quote=True)}"><table><caption>{escape(item["label"])}</caption><thead><tr>{headers}</tr></thead><tbody>{"".join(rows)}</tbody></table></div>'
    if kind == 'links':
        return '<p class="related">See also: ' + ' · '.join(f'<a href="{escape(v[0], quote=True)}">{escape(v[1])}</a>' for v in item['items']) + '</p>'
    if kind == 'list':
        return '<ul class="notes">' + ''.join(f'<li>{escape(text)}</li>' for text in item['items']) + '</ul>'
    if kind == 'note':
        return f'<aside class="callout"><strong>{escape(item["title"])}</strong><p>{escape(item["text"])}</p></aside>'
    if kind == 'heading':
        return f'<h2 id="{escape(item["id"], quote=True)}">{escape(item["text"])}</h2>'
    if kind == 'text':
        return f'<p>{escape(item["text"])}</p>'
    raise ValueError(f'Unknown content block: {kind}')

def build():
    manifest = json.loads((ROOT / 'content' / 'sections.json').read_text())
    pages = [json.loads((ROOT / 'content' / f'{name}.json').read_text()) for name in manifest]
    template = Template((ROOT / 'templates' / 'page.html').read_text())
    for index, (slug, page) in enumerate(zip(manifest, pages)):
        headings = []
        used = {'main', 'sections', 'section-search', 'search-status'}
        for item in page['blocks']:
            if item['type'] == 'heading':
                anchor = item.get('id') or re.sub(r'[^a-z0-9]+', '-', item['text'].lower()).strip('-')
                if anchor in used:
                    raise ValueError(f'Duplicate section anchor: {slug}#{anchor}')
                used.add(anchor)
                item['id'] = anchor
                headings.append(f'<li><a href="#{anchor}">{escape(item["text"])}</a></li>')
        toc = '<nav class="page-toc" aria-label="On this page"><strong>Contents</strong><ul>' + ''.join(headings) + '</ul></nav>' if len(headings) > 1 else ''
        nav = []
        for number, (target, entry) in enumerate(zip(manifest, pages), 1):
            active = ' aria-current="page"' if target == slug else ''
            keywords = escape(' '.join(entry.get('keywords', [])), quote=True)
            nav.append(f'<li data-keywords="{keywords}"><a href="{target}.html"{active}>{escape(entry["nav"])}</a></li>')
        previous = '' if index == 0 else f'<a href="{manifest[index - 1]}.html">Previous: {escape(pages[index - 1]["nav"])}</a>'
        following = '' if index + 1 == len(pages) else f'<a href="{manifest[index + 1]}.html">Next: {escape(pages[index + 1]["nav"])}</a>'
        output = template.substitute(title=escape(page['title']), intro=escape(page['intro']), number=f'{index + 1:02}', navigation=''.join(nav), toc=toc, content=chr(10).join(block(item) for item in page['blocks']), previous=previous, following=following)
        (ROOT / f'{slug}.html').write_text(output)

if __name__ == '__main__':
    build()
