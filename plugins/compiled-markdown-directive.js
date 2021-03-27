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
  },
}

const configuredXss = new xss.FilterXSS(options)
const headerPrefix = 'user-defined-'

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
