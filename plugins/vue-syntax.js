import Vue from 'vue'
import hljs from 'highlight.js/lib/core'
// Scripting
import javascript from 'highlight.js/lib/languages/javascript'
import python from 'highlight.js/lib/languages/python'
import lua from 'highlight.js/lib/languages/lua'
// Coding
import java from 'highlight.js/lib/languages/java'
import kotlin from 'highlight.js/lib/languages/kotlin'
import scala from 'highlight.js/lib/languages/scala'
import groovy from 'highlight.js/lib/languages/groovy'
// Configs
import gradle from 'highlight.js/lib/languages/gradle'
import json from 'highlight.js/lib/languages/json'
import ini from 'highlight.js/lib/languages/ini'
import yaml from 'highlight.js/lib/languages/yaml'
import xml from 'highlight.js/lib/languages/xml'
import properties from 'highlight.js/lib/languages/properties'

/* REGISTRATION */
// Scripting
hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('lua', lua)
// Coding
hljs.registerLanguage('java', java)
hljs.registerLanguage('kotlin', kotlin)
hljs.registerLanguage('scala', scala)
hljs.registerLanguage('groovy', groovy)
// Configs
hljs.registerLanguage('gradle', gradle)
hljs.registerLanguage('json', json)
hljs.registerLanguage('ini', ini)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('properties', properties)

/* ALIASES */
// Scripting
hljs.registerAliases(['js'], 'javascript')
hljs.registerAliases(['py'], 'python')
// Coding
hljs.registerAliases(['kt'], 'kotlin')
// Configs
hljs.registerAliases(['json5'], 'json')
hljs.registerAliases(['toml'], 'ini')
hljs.registerAliases(['yml'], 'yaml')
hljs.registerAliases(['html', 'htm', 'xhtml', 'mcui', 'fxml'], 'xml')

Vue.directive('highlightjs', {
  deep: true,
  bind(el, binding) {
    // on first bind, highlight all targets
    const targets = el.querySelectorAll('pre > code')
    targets.forEach((target) => {
      // if a value is directly assigned to the directive, use this
      // instead of the element content.
      if (binding.value) {
        target.textContent = binding.value
      }
      hljs.highlightBlock(target)
    })
  },
  componentUpdated(el, binding) {
    // after an update, re-fill the content and then highlight
    const targets = el.querySelectorAll('pre > code')
    targets.forEach((target) => {
      if (binding.value) {
        target.textContent = binding.value
        hljs.highlightBlock(target)
      }
    })
  },
})
