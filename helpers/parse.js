import MarkdownIt from 'markdown-it'
import xss from 'xss'

export const configuredXss = new xss.FilterXSS({
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
    iframe: ['width', 'height', 'allowfullscreen', 'frameborder', 'start', 'end'],
    img: [...xss.whiteList.img, 'usemap', 'style'],
    map: ['name'],
    area: [...xss.whiteList.a, 'coords'],
    a: [...xss.whiteList.a, 'rel'],
    td: [...xss.whiteList.td, 'style'],
    th: [...xss.whiteList.th, 'style'],
  },
  css: {
    whiteList: {
      'image-rendering': /^pixelated$/,
      'text-align': /^center|left|right$/,
      float: /^left|right$/,
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
        {
          regex: /^https?:\/\/(www\.)?discord\.com\/widget\?id=\d{18,19}(&theme=\w+)?$/,
          remove: [/&theme=\w+/],
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

    // For Highlight.JS
    if (name === 'class' && ['pre', 'code', 'span'].includes(tag)) {
      const allowedClasses = []
      for (const className of value.split(/\s/g)) {
        if (className.startsWith('hljs-') || className.startsWith('language-')) {
          allowedClasses.push(className)
        }
      }
      return name + '="' + xss.escapeAttrValue(allowedClasses.join(' ')) + '"'
    }
  },
  safeAttrValue(tag, name, value, cssFilter) {
    if (tag === 'img' && name === 'src' && !value.startsWith('data:')) {
      try {
        const url = new URL(value)

        if (url.hostname.includes('wsrv.nl')) {
          url.searchParams.delete('errorredirect')
        }

        const allowedHostnames = [
          'imgur.com',
          'i.imgur.com',
          'cdn-raw.modrinth.com',
          'cdn.modrinth.com',
          'staging-cdn-raw.modrinth.com',
          'staging-cdn.modrinth.com',
          'github.com',
          'raw.githubusercontent.com',
          'img.shields.io',
          'i.postimg.cc',
          'wsrv.nl',
          'cf.way2muchnoise.eu',
          'bstats.org',
        ]

        if (!allowedHostnames.includes(url.hostname)) {
          return xss.safeAttrValue(
            tag,
            name,
            `https://wsrv.nl/?url=${encodeURIComponent(url.toString())}&n=-1`,
            cssFilter
          )
        } else {
          return xss.safeAttrValue(tag, name, url.toString(), cssFilter)
        }
      } catch (err) {}
    }

    return xss.safeAttrValue(tag, name, value, cssFilter)
  },
})

export const md = (options = {}) => {
  const md = new MarkdownIt('default', {
    html: true,
    linkify: true,
    breaks: false,
    ...options,
  })

  const defaultLinkOpenRenderer =
    md.renderer.rules.link_open ||
    function (tokens, idx, options, _env, self) {
      return self.renderToken(tokens, idx, options)
    }

  md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
    const token = tokens[idx]
    const index = token.attrIndex('href')

    if (index !== -1) {
      const href = token.attrs[index][1]

      try {
        const url = new URL(href)
        const allowedHostnames = ['modrinth.com']

        if (allowedHostnames.includes(url.hostname)) {
          return defaultLinkOpenRenderer(tokens, idx, options, env, self)
        }
      } catch (err) {}
    }

    tokens[idx].attrSet('rel', 'noopener nofollow ugc')

    return defaultLinkOpenRenderer(tokens, idx, options, env, self)
  }

  return md
}

export const renderString = (string) => configuredXss.process(md().render(string))
