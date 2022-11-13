import { promises as fs } from 'fs'
import { sortRoutes } from '@nuxt/utils'
import axios from 'axios'

const STAGING_API_URL = 'https://staging-api.modrinth.com/v2/'
const STAGING_ARIADNE_URL = 'https://staging-ariadne.modrinth.com/v1/'

export default {
  /*
   ** Nuxt target
   ** See https://nuxtjs.org/api/configuration-target
   */
  target: 'server',
  /*
   ** Headers of the page
   ** See https://nuxtjs.org/api/configuration-head
   */
  head: {
    htmlAttrs: {
      lang: 'en',
    },
    title: 'Modrinth',
    meta: [
      {
        charset: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        hid: 'description',
        name: 'description',
        content:
          'Download Minecraft mods, plugins, resource packs, and modpacks on Modrinth. Discover and publish projects on Modrinth with a modern, easy to use interface and API.',
      },
      {
        hid: 'publisher',
        name: 'publisher',
        content: 'Rinth, Inc.',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Modrinth',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Modrinth',
      },
      {
        hid: 'theme-color',
        name: 'theme-color',
        content: '#1bd96a',
      },
      {
        hid: 'color-scheme',
        name: 'color-scheme',
        content: 'light dark',
      },
      {
        hid: 'og:site_name',
        name: 'og:site_name',
        content: 'Modrinth',
      },
      {
        hid: 'og:description',
        name: 'og:description',
        content: 'An open source modding platform',
      },
      {
        hid: 'og:type',
        name: 'og:type',
        content: 'website',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: 'https://modrinth.com',
      },
      {
        hid: 'og:image',
        name: 'og:image',
        content: 'https://cdn.modrinth.com/modrinth-new.png?',
      },
      {
        hid: 'twitter:card',
        name: 'twitter:card',
        content: 'summary',
      },
      {
        hid: 'twitter:site',
        name: 'twitter:site',
        content: '@modrinth',
      },
    ],
    link: [
      {
        rel: 'icon',
        type: 'image/x-icon',
        href: '/favicon-light.ico',
        media: '(prefers-color-scheme:no-preference)',
      },
      {
        rel: 'icon',
        type: 'image/x-icon',
        href: '/favicon.ico',
        media: '(prefers-color-scheme:dark)',
      },
      {
        rel: 'icon',
        type: 'image/x-icon',
        href: '/favicon-light.ico',
        media: '(prefers-color-scheme:light)',
      },
      {
        rel: 'stylesheet',
        href: 'https://cdn-raw.modrinth.com/fonts/inter/inter.css',
      },
      {
        rel: 'search',
        type: 'application/opensearchdescription+xml',
        href: '/opensearch.xml',
        title: 'Modrinth mods',
      },
    ],
  },

  vue: {
    config: {
      devtools: true,
    },
  },
  router: {
    extendRoutes(routes, resolve) {
      routes.splice(
        routes.findIndex((x) => x.name === 'search'),
        1
      )

      routes.push({
        path: '/search',
        component: resolve(__dirname, 'pages/search.vue'),
        name: 'search',
        chunkName: 'pages/search',
        children: [
          {
            path: '/mods',
            component: resolve(__dirname, 'pages/search/mods.vue'),
            name: 'mods',
          },
          {
            path: '/modpacks',
            component: resolve(__dirname, 'pages/search/modpacks.vue'),
            name: 'modpacks',
          },
          {
            path: '/plugins',
            component: resolve(__dirname, 'pages/search/plugins.vue'),
            name: 'plugins',
          },
          {
            path: '/resourcepacks',
            component: resolve(__dirname, 'pages/search/resourcepacks.vue'),
            name: 'resourcepacks',
          },
        ],
      })

      sortRoutes(routes)
    },
    middleware: ['auth', 'analytics'],
  },
  /*
   ** Global CSS
   */
  css: ['~assets/styles/global.scss'],
  /*
   ** Plugins to load before mounting the App
   ** https://nuxtjs.org/guide/plugins
   */
  plugins: [
    '~/plugins/vue-tooltip.js',
    '~/plugins/vue-notification.js',
    '~/plugins/xss.js',
    '~/plugins/vue-syntax.js',
    '~/plugins/shorthands.js',
  ],
  /*
   ** Auto import components
   ** See https://nuxtjs.org/api/configuration-components
   */
  components: true,
  /*
   ** Nuxt.js dev-modules
   */
  buildModules: [
    // Doc: https://github.com/nuxt-community/eslint-module
    '@nuxtjs/eslint-module',
    '@nuxtjs/svg',
    '@nuxtjs/color-mode',
  ],
  /*
   ** Nuxt.js modules
   */
  modules: [
    // Doc: https://axios.nuxtjs.org/usage
    '@nuxtjs/dayjs',
    '@nuxtjs/axios',
    '@nuxtjs/robots',
    '@nuxtjs/style-resources',
    '@nuxtjs/markdownit',
    'cookie-universal-nuxt',
  ],
  ads: {
    // Module options
    ghostMode: true,
    geoEdgeId: '',
  },
  robots: {
    Sitemap: 'https://modrinth.com/sitemap.xml',
  },
  /*
   ** Axios module configuration
   ** See https://axios.nuxtjs.org/options
   */
  axios: {
    baseURL: getApiUrl(),
    headers: {
      common: {
        Accept: 'application/json',
      },
    },
  },
  dayjs: {
    locales: ['en'],
    defaultLocale: 'en',
    plugins: ['relativeTime'],
  },
  /*
   ** Build configuration
   ** See https://nuxtjs.org/api/configuration-build/
   */
  build: {
    transpile: ['vue-tooltip', 'vue-notification'],
    html: {
      minify: {
        collapseWhitespace: true, // as @dario30186 mentioned
        removeComments: true, // 👈 add this line
      },
    },
    babel: {
      plugins: [
        [
          '@babel/plugin-proposal-private-methods',
          {
            loose: true,
          },
        ],
      ],
    },
  },
  markdownit: {
    preset: 'default',
    html: true,
    linkify: true,
    breaks: false,
  },
  loading: {
    color: '#1bd96a',
    height: '2px',
  },
  env: {
    owner: process.env.VERCEL_GIT_REPO_OWNER || 'modrinth',
    slug: process.env.VERCEL_GIT_REPO_SLUG || 'knossos',
    branch: process.env.VERCEL_GIT_COMMIT_REF || 'master',
    hash: process.env.VERCEL_GIT_COMMIT_SHA || 'unknown',
    domain: getDomain(),
    authURLBase: getApiUrl(),
  },
  publicRuntimeConfig: {
    axios: {
      browserBaseURL: process.env.BROWSER_BASE_URL,
    },
    ads: {
      ethicalAds: process.env.ETHICAL_ADS,
    },
    analytics: {
      base_url: process.env.BROWSER_ARIADNE_URL || STAGING_ARIADNE_URL,
    },
  },
  privateRuntimeConfig: {
    axios: {
      baseURL: process.env.BASE_URL,
      headers: {
        common: {
          'x-ratelimit-key': process.env.RATE_LIMIT_IGNORE_KEY || '',
        },
      },
    },
  },
  hooks: {
    build: {
      async before(nuxt, buildOptions) {
        // 30 minutes
        const TTL = 30 * 60 * 1000

        let state = {}
        try {
          state = JSON.parse(
            await fs.readFile('./generated/state.json', 'utf8')
          )
        } catch {
          // File doesn't exist, create folder
          await fs.mkdir('./generated', { recursive: true })
        }

        const API_URL = getApiUrl()

        if (
          // Skip regeneration if within TTL...
          state.lastGenerated &&
          new Date(state.lastGenerated).getTime() + TTL >
            new Date().getTime() &&
          // ...but only if the API URL is the same
          state.apiUrl &&
          state.apiUrl === API_URL
        ) {
          return
        }

        console.log('Generating tags...')

        state.lastGenerated = new Date().toISOString()

        state.apiUrl = API_URL

        const headers = {
          headers: {
            'user-agent': `Knossos generator (admin@modrinth.com)`,
          },
        }

        const [
          categories,
          loaders,
          gameVersions,
          licenses,
          donationPlatforms,
          reportTypes,
        ] = (
          await Promise.all([
            axios.get(`${API_URL}tag/category`, headers),
            axios.get(`${API_URL}tag/loader`, headers),
            axios.get(`${API_URL}tag/game_version`, headers),
            axios.get(`${API_URL}tag/license`, headers),
            axios.get(`${API_URL}tag/donation_platform`, headers),
            axios.get(`${API_URL}tag/report_type`, headers),
          ])
        ).map((it) => it.data)

        state.categories = categories
        state.loaders = loaders
        state.gameVersions = gameVersions
        state.licenses = licenses
        state.donationPlatforms = donationPlatforms
        state.reportTypes = reportTypes

        await fs.writeFile('./generated/state.json', JSON.stringify(state))

        console.log('Tags generated!')
      },
    },
    render: {
      routeDone(url, result, context) {
        setTimeout(() => {
          axios
            .post(
              `${process.env.ARIADNE_URL || STAGING_ARIADNE_URL}view`,
              {
                url: getDomain() + url,
              },
              {
                headers: {
                  'Modrinth-Admin': process.env.ARIADNE_ADMIN_KEY || 'feedbeef',
                  'User-Agent':
                    context.req.rawHeaders[
                      context.req.rawHeaders.findIndex(
                        (x) => x === 'User-Agent'
                      ) + 1
                    ],
                },
              }
            )
            .then(() => {})
            .catch((e) => {
              console.error(
                'An error occurred while registering the visit: ',
                e
              )
            })
        })
      },
    },
  },
}

function getApiUrl() {
  return process.env.BROWSER_BASE_URL ?? STAGING_API_URL
}

function getDomain() {
  if (process.env.NODE_ENV === 'production') {
    if (process.env.SITE_URL) {
      return process.env.SITE_URL
    } else if (process.env.HEROKU_APP_NAME) {
      return `https://${process.env.HEROKU_APP_NAME}.herokuapp.com`
    } else if (process.env.VERCEL_URL) {
      return `https://${process.env.VERCEL_URL}`
    } else if (getApiUrl() === STAGING_API_URL) {
      return 'https://staging.modrinth.com'
    } else {
      return 'https://modrinth.com'
    }
  } else {
    return 'http://localhost:3000'
  }
}
