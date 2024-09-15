import {
  InfoIcon,
  LightBulbIcon,
  MessageSquareWarningIcon,
  OctogonAlertIcon,
  TriangleAlertIcon,
} from '@modrinth/assets'
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
      note: InfoIcon,
      tip: LightBulbIcon,
      important: MessageSquareWarningIcon,
      warning: TriangleAlertIcon,
      caution: OctogonAlertIcon,
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
