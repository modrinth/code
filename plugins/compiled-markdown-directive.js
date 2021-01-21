import Vue from 'vue'
import xss from 'xss'
import marked from 'marked'

const options = {
  whiteList: {
    ...xss.whiteList,
    summary: [],
  },
}

const configuredXss = new xss.FilterXSS(options)

function compileMarkdown(target, markdown) {
  target.innerHTML = configuredXss.process(marked(markdown))
}

Vue.directive('compiled-markdown', {
  bind(el, binding, vnode) {
    compileMarkdown(el, binding.value)
  },

  update(el, binding, vnode) {
    compileMarkdown(el, binding.value)
  },
})
