import xss from 'xss'

/**
 * @type {import('xss').IFilterXSSOptions}
 */
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
    kbd: ['id'],
    input: ['checked', 'disabled', 'type'],
    iframe: ['width', 'height', 'allowfullscreen', 'frameborder'],
    img: [...xss.whiteList.img, 'style'],
  },
  css: {
    whiteList: {
      'image-rendering': /^pixelated$/,
    },
  },
  onIgnoreTagAttr: (tag, name, value) => {
    // Allow iframes from acceptable sources
    if (tag === 'iframe' && name === 'src') {
      const allowedSources = [
        {
          regex:
            /^https?:\/\/(www\.)?youtube(-nocookie)?\.com\/embed\/[a-zA-Z0-9_-]{11}(\?&autoplay=[0-1]{1})?$/,
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

export default (ctx, inject) => {
  inject('xss', (string) => configuredXss.process(string))
}
