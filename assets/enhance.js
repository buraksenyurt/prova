// Code highlighting and math rendering for Markdown blocks.
//
// highlight.js and KaTeX are vendored under ./vendor and loaded only the first
// time a block actually contains code or math. If they fail to load, the
// content stays readable: code as plain text, formulas as TeX source.
(function () {
  "use strict";

  const base = new URL(".", (document.currentScript && document.currentScript.src) || location.href).href;
  const pending = {};

  function once(key, load) {
    if (!pending[key]) {
      pending[key] = load().catch(function (err) {
        delete pending[key]; // allow a retry later
        throw err;
      });
    }
    return pending[key];
  }

  function stylesheet(path) {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = base + path;
    document.head.appendChild(link);
  }

  function script(path) {
    return new Promise(function (resolve, reject) {
      const s = document.createElement("script");
      s.src = base + path;
      s.onload = resolve;
      s.onerror = reject;
      document.head.appendChild(s);
    });
  }

  function loadHighlight() {
    return once("highlight", function () {
      stylesheet("vendor/highlight/github-dark.min.css");
      return script("vendor/highlight/highlight.min.js")
        .then(function () { return script("vendor/highlight/dockerfile.min.js"); })
        .then(function () { window.hljs.configure({ ignoreUnescapedHTML: true }); });
    });
  }

  function loadKatex() {
    return once("katex", function () {
      stylesheet("vendor/katex/katex.min.css");
      return script("vendor/katex/katex.min.js");
    });
  }

  function run(root) {
    const codes = Array.prototype.filter.call(
      root.querySelectorAll('pre > code[class*="language-"]'),
      function (c) { return !c.dataset.highlighted; }
    );
    if (codes.length) {
      loadHighlight().then(function () {
        codes.forEach(function (c) {
          const lang = (c.className.match(/language-([\w+#-]+)/) || [])[1];
          if (lang && window.hljs.getLanguage(lang)) {
            window.hljs.highlightElement(c);
          }
        });
      }).catch(function () { /* stays plain */ });
    }

    const maths = Array.prototype.filter.call(
      root.querySelectorAll(".math"),
      function (m) { return !m.dataset.rendered; }
    );
    if (maths.length) {
      loadKatex().then(function () {
        maths.forEach(function (m) {
          const tex = m.textContent;
          try {
            window.katex.render(tex, m, {
              displayMode: m.classList.contains("math-display"),
              throwOnError: false,
            });
            m.dataset.rendered = "1";
            m.title = tex;
          } catch (e) { /* keep the source */ }
        });
      }).catch(function () { /* stays as TeX */ });
    }
  }

  // Called from Rust right after a Markdown block is created; the element may
  // not be in the document yet, so wait for the next frame.
  window.provaEnhance = function (root) {
    requestAnimationFrame(function () { run(root); });
  };
})();
