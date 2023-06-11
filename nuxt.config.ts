import { promises as fs } from 'fs'
import { pathToFileURL } from 'node:url'
import svgLoader from 'vite-svg-loader'
import { resolve, basename } from 'pathe'
import { defineNuxtConfig } from 'nuxt/config'
import { $fetch } from 'ofetch'
import { globIterate } from 'glob'
import { match as matchLocale } from '@formatjs/intl-localematcher'

const STAGING_API_URL = 'https://staging-api.modrinth.com/v2/'
const STAGING_ARIADNE_URL = 'https://staging-ariadne.modrinth.com/v1/'

const preloadedFonts = [
  'inter/Inter-Regular.woff2',
  'inter/Inter-Medium.woff2',
  'inter/Inter-SemiBold.woff2',
  'inter/Inter-Bold.woff2',
]

const favicons = {
  '(prefers-color-scheme:no-preference)': '/favicon-light.ico',
  '(prefers-color-scheme:light)': '/favicon-light.ico',
  '(prefers-color-scheme:dark)': '/favicon.ico',
}

const meta = {
  description:
    'Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. Discover and publish projects on Modrinth with a modern, easy to use interface and API.',
  publisher: 'Rinth, Inc.',
  'apple-mobile-web-app-title': 'Modrinth',
  'theme-color': '#1bd96a',
  'color-scheme': 'dark light',
  // OpenGraph
  'og:title': 'Modrinth',
  'og:site_name': 'Modrinth',
  'og:description': 'An open source modding platform',
  'og:type': 'website',
  'og:url': 'https://modrinth.com',
  'og:image': 'https://cdn.modrinth.com/modrinth-new.png?',
  // Twitter
  'twitter:card': 'summary',
  'twitter:site': '@modrinth',
}

/**
 * Tags of locales that are auto-discovered besides the default locale.
 *
 * Preferably only the locales that reach a certain threshold of complete
 * translations would be included in this array.
 */
const ENABLED_LOCALES: string[] = []

export default defineNuxtConfig({
  app: {
    head: {
      htmlAttrs: {
        lang: 'en',
      },
      title: 'Modrinth',
      meta: Object.entries(meta).map(([name, content]): object => {
        return { name, content }
      }),
      link: [
        // The type is necessary because the linter can't always compare this very nested/complex type on itself
        ...preloadedFonts.map((font): object => {
          return {
            rel: 'preload',
            href: `https://cdn-raw.modrinth.com/fonts/${font}?v=3.19`,
            as: 'font',
            type: 'font/woff2',
            crossorigin: 'anonymous',
          }
        }),
        ...Object.entries(favicons).map(([media, href]): object => {
          return { rel: 'icon', type: 'image/x-icon', href, media }
        }),
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
    ],
  },
  hooks: {
    async 'build:before'() {
      // 30 minutes
      const TTL = 30 * 60 * 1000

      let state: {
        lastGenerated?: string
        apiUrl?: string
        categories?: any[]
        loaders?: any[]
        gameVersions?: any[]
        donationPlatforms?: any[]
        reportTypes?: any[]
      } = {}
      let homePageProjects: any[] = []

      try {
        state = JSON.parse(await fs.readFile('./generated/state.json', 'utf8'))
        homePageProjects = JSON.parse(await fs.readFile('./generated/homepage.json', 'utf8'))
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
        state.apiUrl === API_URL &&
        homePageProjects.length !== 0
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

      const [categories, loaders, gameVersions, donationPlatforms, reportTypes, projects] =
        await Promise.all([
          $fetch(`${API_URL}tag/category`, headers),
          $fetch(`${API_URL}tag/loader`, headers),
          $fetch(`${API_URL}tag/game_version`, headers),
          $fetch(`${API_URL}tag/donation_platform`, headers),
          $fetch(`${API_URL}tag/report_type`, headers),
          $fetch(`${API_URL}projects_random?count=40`, headers),
        ])

      state.categories = categories
      state.loaders = loaders
      state.gameVersions = gameVersions
      state.donationPlatforms = donationPlatforms
      state.reportTypes = reportTypes
      homePageProjects = projects

      await fs.writeFile('./generated/state.json', JSON.stringify(state))
      await fs.writeFile('./generated/homepage.json', JSON.stringify(homePageProjects))

      console.log('Tags generated!')
    },
    'pages:extend'(routes) {
      routes.splice(
        routes.findIndex((x) => x.name === 'search-searchProjectType'),
        1
      )

      const types = ['mods', 'modpacks', 'plugins', 'resourcepacks', 'shaders', 'datapacks']

      types.forEach((type) =>
        routes.push({
          name: `search-${type}`,
          path: `/${type}`,
          file: resolve(__dirname, 'pages/search/[searchProjectType].vue'),
          children: [],
        })
      )
    },
    async 'vintl:extendOptions'(opts) {
      opts.locales ??= []

      const resolveCompactNumberDataImport = await (async () => {
        const compactNumberLocales: string[] = []
        const resolvedImports = new Map<string, string>()

        for await (const localeFile of globIterate(
          'node_modules/@vintl/compact-number/dist/locale-data/*.mjs',
          { ignore: '**/*.data.mjs' }
        )) {
          const tag = basename(localeFile, '.mjs')
          compactNumberLocales.push(tag)
          resolvedImports.set(tag, String(pathToFileURL(resolve(localeFile))))
        }

        function resolveImport(tag: string) {
          const matchedTag = matchLocale([tag], compactNumberLocales, 'en-x-placeholder')
          return matchedTag === 'en-x-placeholder'
            ? undefined
            : `@vintl/compact-number/locale-data/${matchedTag}`
        }

        return resolveImport
      })()

      for await (const localeDir of globIterate('locales/*/', { posix: true })) {
        const tag = basename(localeDir)
        if (!ENABLED_LOCALES.includes(tag) && opts.defaultLocale !== tag) continue

        const locale =
          opts.locales.find((locale) => locale.tag === tag) ??
          opts.locales[opts.locales.push({ tag }) - 1]

        for await (const localeFile of globIterate(`${localeDir}/*`, { posix: true })) {
          const fileName = basename(localeFile)
          if (fileName === 'index.json') {
            if (locale.file == null) {
              locale.file = {
                from: `./${localeFile}`,
                format: 'crowdin',
              }
            } else {
              ;(locale.files ??= []).push({
                from: `./${localeFile}`,
                format: 'crowdin',
              })
            }
          } else if (fileName === 'meta.json') {
            /** @type {Record<string, { message: string }>} */
            const meta = await fs.readFile(localeFile, 'utf8').then((date) => JSON.parse(date))
            locale.meta ??= {}
            for (const key in meta) {
              locale.meta[key] = meta[key].message
            }
          } else {
            ;(locale.resources ??= {})[fileName] = `./${localeFile}`
          }
        }

        const cnDataImport = resolveCompactNumberDataImport(tag)
        if (cnDataImport != null) {
          ;(locale.additionalImports ??= []).push({
            from: cnDataImport,
            resolve: false,
          })
        }
      }
    },
  },
  runtimeConfig: {
    apiBaseUrl: process.env.BASE_URL ?? getApiUrl(),
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
  typescript: {
    shim: false,
    strict: true,
    typeCheck: true,
    tsConfig: {
      compilerOptions: {
        moduleResolution: 'bundler',
        allowImportingTsExtensions: true,
      },
    },
  },
  modules: ['@vintl/nuxt'],
  vintl: {
    defaultLocale: 'en-US',
    storage: 'cookie',
    parserless: 'only-prod',
  },
  nitro: {
    moduleSideEffects: ['@vintl/compact-number/locale-data'],
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
