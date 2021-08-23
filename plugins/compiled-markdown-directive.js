import Vue from 'vue'
import xss from 'xss'
import marked from 'marked'

const options = {
  whiteList: {
    ...xss.whiteList,
    summary: [],
    h1: ['id'],
    h2: ['id'],
    h3: ['id'],
    h4: ['id'],
    h5: ['id'],
    h6: ['id'],
    input: ['checked', 'disabled', 'type'],
    iframe: ['width', 'height', 'allowfullscreen', 'frameborder'],
  },
  onIgnoreTagAttr: (tag, name, value) => {
    // Allow iframes from acceptable sources
    if (tag === 'iframe' && name === 'src') {
      const allowedSources = [
        {
          regex: /^https?:\/\/(www\.)?youtube\.com\/embed\/[a-zA-Z0-9_]{11}(\?&autoplay=[0-1]{1})?$/,
          remove: ['&autoplay=1'], // Prevents autoplay
        },
      ]

      for (const source of allowedSources) {
        if (source.regex.test(value)) {
          for (const remove of source.remove) {
            value = value.replace(remove, '')
          }
          return name + '="' + xss.escapeAttrValue(value) + '"'
        }
      }
    }
  },
}

const configuredXss = new xss.FilterXSS(options)
const headerPrefix = 'user-defined-'

const renderer = {
  image(href, text) {
    if (
      /^https?:\/\/(www\.)?youtube\.com\/watch\?v=[a-zA-Z0-9_]{11}$/.test(href)
    ) {
      return `<iframe width="560" height="315" src="https://www.youtube.com/embed/${href.substring(
        32,
        43
      )}" frameborder="0" allowfullscreen></iframe>`
    } else {
      return `<img src="${href}" alt="${text}">`
    }
  },
}

marked.use({ renderer })

function compileMarkdown(target, markdown) {
  target.innerHTML = configuredXss.process(marked(markdown, { headerPrefix }))
}

function onHashChange() {
  const fragment = decodeURIComponent(location.hash.substr(1)) // decodeURIComponent prevents issues with emoji and other unicode character
  const fragmentElement = document.getElementById(headerPrefix + fragment)

  if (fragmentElement != null) {
    fragmentElement.scrollIntoView()
  }
}

Vue.directive('compiled-markdown', {
  bind(el, binding, vnode) {
    compileMarkdown(el, binding.value)

    window.addEventListener('hashchange', onHashChange)
    // `hashchange` does not get triggered if the link doesn't change the hash.
    el.addEventListener('click', (event) => {
      const hrefLocation = event.target.getAttribute('href')
      if (hrefLocation === decodeURIComponent(location.hash)) {
        onHashChange()
      }
    })

    onHashChange()
  },

  update(el, binding, vnode) {
    compileMarkdown(el, binding.value)
  },
})
