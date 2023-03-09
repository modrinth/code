import { promises as fs } from 'fs'
import svgLoader from 'vite-svg-loader'
import eslintPlugin from 'vite-plugin-eslint'
import { resolve } from 'pathe'
import { defineNuxtConfig } from 'nuxt/config'
import { $fetch } from 'ofetch'

const STAGING_API_URL = 'https://staging-api.modrinth.com/v2/'
const STAGING_ARIADNE_URL = 'https://staging-ariadne.modrinth.com/v1/'

export default defineNuxtConfig({
  app: {
    head: {
      htmlAttrs: {
        lang: 'en',
      },
      title: 'Modrinth',
      meta: [
        {
          name: 'description',
          content:
            'Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. Discover and publish projects on Modrinth with a modern, easy to use interface and API.',
        },
        {
          name: 'publisher',
          content: 'Rinth, Inc.',
        },
        {
          name: 'og:title',
          content: 'Modrinth',
        },
        {
          name: 'apple-mobile-web-app-title',
          content: 'Modrinth',
        },
        {
          name: 'theme-color',
          content: '#1bd96a',
        },
        {
          name: 'color-scheme',
          content: 'dark light',
        },
        {
          name: 'og:site_name',
          content: 'Modrinth',
        },
        {
          name: 'og:description',
          content: 'An open source modding platform',
        },
        {
          name: 'og:type',
          content: 'website',
        },
        {
          name: 'og:url',
          content: 'https://modrinth.com',
        },
        {
          name: 'og:image',
          content: 'https://cdn.modrinth.com/modrinth-new.png?',
        },
        {
          name: 'twitter:card',
          content: 'summary',
        },
        {
          name: 'twitter:site',
          content: '@modrinth',
        },
      ],
      link: [
        {
          rel: 'preload',
          href: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Regular.woff2?v=3.19',
          as: 'font',
          type: 'font/woff2',
          crossorigin: true,
        },
        {
          rel: 'preload',
          href: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Medium.woff2?v=3.19',
          as: 'font',
          type: 'font/woff2',
          crossorigin: true,
        },
        {
          rel: 'preload',
          href: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-SemiBold.woff2?v=3.19',
          as: 'font',
          type: 'font/woff2',
          crossorigin: true,
        },
        {
          rel: 'preload',
          href: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Bold.woff2?v=3.19',
          as: 'font',
          type: 'font/woff2',
          crossorigin: true,
        },
        {
          rel: 'preload',
          href: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-ExtraBold.woff2?v=3.19',
          as: 'font',
          type: 'font/woff2',
          crossorigin: true,
        },
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
          rel: 'search',
          type: 'application/opensearchdescription+xml',
          href: '/opensearch.xml',
          title: 'Modrinth mods',
        },
      ],
    },
  },
  vite: {
    plugins: [
      svgLoader({
        svgoConfig: {
          plugins: [
            {
              name: 'preset-default',
              params: {
                overrides: {
                  removeViewBox: false,
                },
              },
            },
          ],
        },
      }),
      eslintPlugin(),
    ],
  },
  dayjs: {
    locales: ['en'],
    defaultLocale: 'en',
    plugins: ['relativeTime'],
  },
  hooks: {
    async 'build:before'() {
      // 30 minutes
      const TTL = 30 * 60 * 1000

      let state = {}
      try {
        state = JSON.parse(await fs.readFile('./generated/state.json', 'utf8'))
      } catch {
        // File doesn't exist, create folder
        await fs.mkdir('./generated', { recursive: true })
      }

      const API_URL = getApiUrl()

      if (
        // Skip regeneration if within TTL...
        state.lastGenerated &&
        new Date(state.lastGenerated).getTime() + TTL > new Date().getTime() &&
        // ...but only if the API URL is the same
        state.apiUrl &&
        state.apiUrl === API_URL
      ) {
        return
      }

      state.lastGenerated = new Date().toISOString()

      state.apiUrl = API_URL

      const headers = {
        headers: {
          'user-agent': 'Knossos generator (support@modrinth.com)',
        },
      }

      const [categories, loaders, gameVersions, donationPlatforms, reportTypes] = await Promise.all(
        [
          $fetch(`${API_URL}tag/category`, headers),
          $fetch(`${API_URL}tag/loader`, headers),
          $fetch(`${API_URL}tag/game_version`, headers),
          $fetch(`${API_URL}tag/donation_platform`, headers),
          $fetch(`${API_URL}tag/report_type`, headers),
        ]
      )

      state.categories = categories
      state.loaders = loaders
      state.gameVersions = gameVersions
      state.donationPlatforms = donationPlatforms
      state.reportTypes = reportTypes

      await fs.writeFile('./generated/state.json', JSON.stringify(state))

      console.log('Tags generated!')
    },
    'pages:extend'(routes) {
      routes.splice(
        routes.findIndex((x) => x.name === 'search-searchProjectType'),
        1
      )

      routes.push({
        name: 'search-mods',
        path: '/mods',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
      routes.push({
        name: 'search-modpacks',
        path: '/modpacks',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
      routes.push({
        name: 'search-plugins',
        path: '/plugins',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
      routes.push({
        name: 'search-resourcepacks',
        path: '/resourcepacks',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
      routes.push({
        name: 'search-shaders',
        path: '/shaders',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
      routes.push({
        name: 'search-datapacks',
        path: '/datapacks',
        file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
        children: [],
      })
    },
  },
  runtimeConfig: {
    apiBaseUrl: process.env.BASE_URL ?? getApiUrl(),
    ariadneBaseUrl: process.env.ARIADNE_URL ?? getAriadneUrl(),
    ariadneAdminKey: process.env.ARIADNE_ADMIN_KEY,
    rateLimitKey: process.env.RATE_LIMIT_IGNORE_KEY,
    public: {
      apiBaseUrl: getApiUrl(),
      ariadneBaseUrl: getAriadneUrl(),
      siteUrl: getDomain(),

      owner: process.env.VERCEL_GIT_REPO_OWNER || 'modrinth',
      slug: process.env.VERCEL_GIT_REPO_SLUG || 'knossos',
      branch: process.env.VERCEL_GIT_COMMIT_REF || 'master',
      hash: process.env.VERCEL_GIT_COMMIT_SHA || 'unknown',
    },
  },
})

function getApiUrl() {
  return process.env.BROWSER_BASE_URL ?? STAGING_API_URL
}

function getAriadneUrl() {
  return process.env.BROWSER_ARIADNE_URL ?? STAGING_ARIADNE_URL
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
