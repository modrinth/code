import MarkdownIt from 'markdown-it'
import MarkdownItGitHubAlerts from 'markdown-it-github-alerts'
import { escapeAttrValue, FilterXSS, safeAttrValue, whiteList } from 'xss'

export const configuredXss = new FilterXSS({
  whiteList: {
    ...whiteList,
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
    img: [...(whiteList.img || []), 'usemap', 'style', 'align'],
    map: ['name'],
    area: [...(whiteList.a || []), 'coords'],
    a: [...(whiteList.a || []), 'rel'],
    td: [...(whiteList.td || []), 'style'],
    th: [...(whiteList.th || []), 'style'],
    picture: [],
    source: ['media', 'sizes', 'src', 'srcset', 'type'],
    p: [...(whiteList.p || []), 'align'],
    div: [...(whiteList.p || []), 'align'],
    svg: [
      'aria-hidden',
      'width',
      'height',
      'viewBox',
      'fill',
      'stroke',
      'stroke-width',
      'stroke-linecap',
      'stroke-linejoin',
    ],
    path: ['d'],
    circle: ['cx', 'cy', 'r'],
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
          url: /^https?:\/\/(www\.)?youtube(-nocookie)?\.com\/embed\/[a-zA-Z0-9_-]{11}/,
          allowedParameters: [/start=\d+/, /end=\d+/],
        },
        {
          url: /^https?:\/\/(www\.)?discord\.com\/widget/,
          allowedParameters: [/id=\d{18,19}/],
        },
      ]

      const url = new URL(value)

      for (const source of allowedSources) {
        if (!source.url.test(url.href)) {
          continue
        }

        const newSearchParams = new URLSearchParams()
        url.searchParams.forEach((value, key) => {
          if (!source.allowedParameters.some((param) => param.test(`${key}=${value}`))) {
            newSearchParams.delete(key)
          }
        })

        url.search = newSearchParams.toString()
        return `${name}="${escapeAttrValue(url.toString())}"`
      }
    }

    // For Highlight.JS
    if (name === 'class' && ['pre', 'code', 'span'].includes(tag)) {
      const allowedClasses: string[] = []
      for (const className of value.split(/\s/g)) {
        if (className.startsWith('hljs-') || className.startsWith('language-')) {
          allowedClasses.push(className)
        }
      }
      return `${name}="${escapeAttrValue(allowedClasses.join(' '))}"`
    }

    // For markdown callouts
    if (name === 'class' && ['div', 'p'].includes(tag)) {
      const classWhitelist = [
        'markdown-alert',
        'markdown-alert-note',
        'markdown-alert-tip',
        'markdown-alert-warning',
        'markdown-alert-important',
        'markdown-alert-caution',
        'markdown-alert-title',
      ]

      const allowed: string[] = []
      for (const className of value.split(/\s/g)) {
        if (classWhitelist.includes(className)) {
          allowed.push(className)
        }
      }

      return `${name}="${escapeAttrValue(allowed.join(' '))}"`
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
          return safeAttrValue(
            tag,
            name,
            `https://wsrv.nl/?url=${encodeURIComponent(
              url.toString().replaceAll('&amp;', '&'),
            )}&n=-1`,
            cssFilter,
          )
        }
        return safeAttrValue(tag, name, url.toString(), cssFilter)
      } catch (err) {
        /* empty */
      }
    }

    return safeAttrValue(tag, name, value, cssFilter)
  },
})

export const md = (options = {}) => {
  const md = new MarkdownIt('default', {
    html: true,
    linkify: true,
    breaks: false,
    ...options,
  })

  md.use(MarkdownItGitHubAlerts, {
    icons: {
      note: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-info"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
      tip: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-lightbulb"><path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/></svg>',
      important:
        '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-message-square-warning"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M12 7v2"/><path d="M12 13h.01"/></svg>',
      warning:
        '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-triangle-alert"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>',
      caution:
        '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-octagon-alert"><path d="M12 16h.01"/><path d="M12 8v4"/><path d="M15.312 2a2 2 0 0 1 1.414.586l4.688 4.688A2 2 0 0 1 22 8.688v6.624a2 2 0 0 1-.586 1.414l-4.688 4.688a2 2 0 0 1-1.414.586H8.688a2 2 0 0 1-1.414-.586l-4.688-4.688A2 2 0 0 1 2 15.312V8.688a2 2 0 0 1 .586-1.414l4.688-4.688A2 2 0 0 1 8.688 2z"/></svg>',
    },
  })

  const defaultLinkOpenRenderer =
    md.renderer.rules.link_open ||
    function (tokens, idx, options, _env, self) {
      return self.renderToken(tokens, idx, options)
    }

  md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
    const token = tokens[idx]
    const index = token.attrIndex('href')

    if (token.attrs && index !== -1) {
      const href = token.attrs[index][1]

      try {
        const url = new URL(href)
        const allowedHostnames = ['modrinth.com']

        if (allowedHostnames.includes(url.hostname)) {
          return defaultLinkOpenRenderer(tokens, idx, options, env, self)
        }
      } catch (err) {
        /* empty */
      }
    }

    tokens[idx].attrSet('rel', 'noopener nofollow ugc')

    return defaultLinkOpenRenderer(tokens, idx, options, env, self)
  }

  return md
}

export const renderString = (string: string) => configuredXss.process(md().render(string))
