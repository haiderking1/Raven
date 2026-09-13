/* Progressive enhancement only: every page and link works without JavaScript. */
(() => {
  function fallbackCopy(text) {
    const field = document.createElement('textarea');
    field.value = text;
    field.setAttribute('readonly', '');
    field.style.position = 'fixed';
    field.style.top = '-10000px';
    document.body.append(field);
    field.select();
    try { return document.execCommand('copy'); }
    finally { field.remove(); }
  }
  document.querySelectorAll('.sample').forEach(sample => {
    const button = sample.querySelector('.copy');
    const code = sample.querySelector('code');
    const status = sample.querySelector('.copy-status');
    button.hidden = false;
    button.addEventListener('click', async () => {
      let copied = false;
      try {
        if (navigator.clipboard && window.isSecureContext) {
          await navigator.clipboard.writeText(code.textContent);
          copied = true;
        }
      } catch (_) { /* file:// clipboard policies differ between browsers. */ }
      if (!copied) {
        try { copied = fallbackCopy(code.textContent); } catch (_) { copied = false; }
      }
      button.focus({preventScroll: true});
      if (!copied) {
        const range = document.createRange();
        range.selectNodeContents(code);
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
      }
      status.textContent = copied ? 'Copied to clipboard.' : 'Clipboard access was blocked. The example is selected; press Ctrl+C to copy it.';
    });
  });
  const contents = document.querySelector('.contents');
  const narrow = window.matchMedia('(max-width: 820px)');
  const adaptContents = () => { contents.open = !narrow.matches; };
  adaptContents();
  narrow.addEventListener('change', adaptContents);
  const search = document.querySelector('#section-search');
  const entries = Array.from(document.querySelectorAll('#sections li'));
  const status = document.querySelector('#search-status');
  document.querySelector('.section-search').hidden = false;
  search.addEventListener('input', () => {
    const query = search.value.toLocaleLowerCase().trim();
    let visible = 0;
    entries.forEach(entry => {
      entry.hidden = !(entry.textContent + ' ' + entry.dataset.keywords).toLocaleLowerCase().includes(query);
      if (!entry.hidden) visible++;
    });
    status.textContent = visible === 0 ? 'No matching sections. Clear the search to show all.' : '';
  });
})();
