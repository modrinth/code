import { pathToFileURL } from 'node:url'

import { match as matchLocale } from '@formatjs/intl-localematcher'
import { GenericModrinthClient, type Labrinth } from '@modrinth/api-client'
import serverSidedVue from '@vitejs/plugin-vue'
import { consola } from 'consola'
import { promises as fs } from 'fs'
import { globIterate } from 'glob'
import { defineNuxtConfig } from 'nuxt/config'
import { basename, relative } from 'pathe'
import svgLoader from 'vite-svg-loader'

const STAGING_API_URL = 'https://staging-api.modrinth.com/v2/'

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

/**
 * Tags of locales that are auto-discovered besides the default locale.
 *
 * Preferably only the locales that reach a certain threshold of complete
 * translations would be included in this array.
 */
// const enabledLocales: string[] = []

/**
 * Overrides for the categories of the certain locales.
 */
const localesCategoriesOverrides: Partial<Record<string, 'fun' | 'experimental'>> = {
	'en-x-pirate': 'fun',
	'en-x-updown': 'fun',
	'en-x-lolcat': 'fun',
	'en-x-uwu': 'fun',
	'ru-x-bandit': 'fun',
	ar: 'experimental',
	he: 'experimental',
	pes: 'experimental',
}

export default defineNuxtConfig({
	srcDir: 'src/',
	app: {
		head: {
			htmlAttrs: {
				lang: 'en',
			},
			title: 'Modrinth',
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
				...Object.entries(favicons).map(([media, href]): object => {
					return { rel: 'apple-touch-icon', type: 'image/x-icon', href, media, sizes: '64x64' }
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
		define: {
			global: {},
		},
		esbuild: {
			define: {
				global: 'globalThis',
			},
		},
		cacheDir: '../../node_modules/.vite/apps/knossos',
		resolve: {
			dedupe: ['vue'],
		},
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
		async 'nitro:config'(nitroConfig) {
			const emailTemplates = Object.keys(
				await import('./src/templates/emails/index.ts').then((m) => m.default),
			)
			const docTemplates = Object.keys(
				await import('./src/templates/docs/index.ts').then((m) => m.default),
			)

			nitroConfig.prerender = nitroConfig.prerender || {}
			nitroConfig.prerender.routes = nitroConfig.prerender.routes || []
			for (const template of emailTemplates) {
				nitroConfig.prerender.routes.push(`/_internal/templates/email/${template}`)
			}
			for (const template of docTemplates) {
				nitroConfig.prerender.routes.push(`/_internal/templates/doc/${template}`)
			}
		},
		async 'build:before'() {
			// 30 minutes
			const TTL = 30 * 60 * 1000

			let state: Partial<Labrinth.State.GeneratedState & Record<string, any>> = {}

			try {
				state = JSON.parse(await fs.readFile('./src/generated/state.json', 'utf8'))
			} catch {
				// File doesn't exist, create folder
				await fs.mkdir('./src/generated', { recursive: true })
			}

			const API_URL = getApiUrl()

			if (
				// Skip regeneration if within TTL...
				state.lastGenerated &&
				new Date(state.lastGenerated).getTime() + TTL > new Date().getTime() &&
				// ...but only if the API URL is the same
				state.apiUrl === API_URL &&
				// ...and if no errors were caught during the last generation
				(state.errors ?? []).length === 0
			) {
				console.log(
					'Tags already recently generated. Delete apps/frontend/src/generated/state.json to force regeneration.',
				)
				return
			}

			const client = new GenericModrinthClient({
				labrinthBaseUrl: API_URL.replace('/v2/', ''),
				userAgent: 'Knossos generator (support@modrinth.com)',
			})

			const generatedState = await client.labrinth.state.build()
			state.lastGenerated = new Date().toISOString()
			state.apiUrl = API_URL
			state = {
				...state,
				...generatedState,
			}

			await fs.writeFile('./src/generated/state.json', JSON.stringify(state))

			console.log('Tags generated!')
		},
		async 'vintl:extendOptions'(opts) {
			opts.locales ??= []

			// const isProduction = getDomain() === 'https://modrinth.com'

			const resolveCompactNumberDataImport = await (async () => {
				const compactNumberLocales: string[] = []

				for await (const localeFile of globIterate(
					'node_modules/@vintl/compact-number/dist/locale-data/*.mjs',
					{ ignore: '**/*.data.mjs' },
				)) {
					const tag = basename(localeFile, '.mjs')
					compactNumberLocales.push(tag)
				}

				function resolveImport(tag: string) {
					const matchedTag = matchLocale([tag], compactNumberLocales, 'en-x-placeholder')
					return matchedTag === 'en-x-placeholder'
						? undefined
						: `@vintl/compact-number/locale-data/${matchedTag}`
				}

				return resolveImport
			})()

			const resolveOmorphiaLocaleImport = await (async () => {
				const omorphiaLocales: string[] = []
				const omorphiaLocaleSets = new Map<string, { files: { from: string; format?: string }[] }>()

				for (const pkgLocales of [`node_modules/@modrinth/**/src/locales/*`]) {
					for await (const localeDir of globIterate(pkgLocales, {
						posix: true,
					})) {
						const tag = basename(localeDir)
						if (!omorphiaLocales.includes(tag)) {
							omorphiaLocales.push(tag)
						}

						const entry = omorphiaLocaleSets.get(tag) ?? { files: [] }
						omorphiaLocaleSets.set(tag, entry)

						for await (const localeFile of globIterate(`${localeDir}/*`, { posix: true })) {
							entry.files.push({
								from: pathToFileURL(localeFile).toString(),
								format: 'default',
							})
						}
					}
				}

				return function resolveLocaleImport(tag: string) {
					return omorphiaLocaleSets.get(matchLocale([tag], omorphiaLocales, 'en-x-placeholder'))
				}
			})()

			for await (const localeDir of globIterate('src/locales/*/', { posix: true })) {
				const tag = basename(localeDir)

				// NOTICE: temporarily disabled all locales except en-US
				if (opts.defaultLocale !== tag) continue

				const locale =
					opts.locales.find((locale) => locale.tag === tag) ??
					opts.locales[opts.locales.push({ tag }) - 1]!

				const localeFiles = (locale.files ??= [])

				for await (const localeFile of globIterate(`${localeDir}/*`, { posix: true })) {
					const fileName = basename(localeFile)
					if (fileName === 'index.json') {
						localeFiles.push({
							from: `./${relative('./src', localeFile)}`,
							format: 'crowdin',
						})
					} else if (fileName === 'meta.json') {
						const meta: Record<string, { message: string }> = await fs
							.readFile(localeFile, 'utf8')
							.then((date) => JSON.parse(date))
						const localeMeta = (locale.meta ??= {})
						for (const key in meta) {
							const value = meta[key]
							if (value === undefined) continue
							localeMeta[key] = value.message
						}
					} else {
						;(locale.resources ??= {})[fileName] = `./${relative('./src', localeFile)}`
					}
				}

				const categoryOverride = localesCategoriesOverrides[tag]
				if (categoryOverride != null) {
					;(locale.meta ??= {}).category = categoryOverride
				}

				const omorphiaLocaleData = resolveOmorphiaLocaleImport(tag)
				if (omorphiaLocaleData != null) {
					localeFiles.push(...omorphiaLocaleData.files)
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
		// @ts-ignore
		apiBaseUrl: process.env.BASE_URL ?? globalThis.BASE_URL ?? getApiUrl(),
		// @ts-ignore
		rateLimitKey: process.env.RATE_LIMIT_IGNORE_KEY ?? globalThis.RATE_LIMIT_IGNORE_KEY,
		pyroBaseUrl: process.env.PYRO_BASE_URL,
		public: {
			apiBaseUrl: getApiUrl(),
			pyroBaseUrl: process.env.PYRO_BASE_URL,
			siteUrl: getDomain(),
			production: isProduction(),
			featureFlagOverrides: getFeatureFlagOverrides(),

			owner: process.env.VERCEL_GIT_REPO_OWNER || 'modrinth',
			slug: process.env.VERCEL_GIT_REPO_SLUG || 'code',
			branch:
				process.env.VERCEL_GIT_COMMIT_REF ||
				process.env.CF_PAGES_BRANCH ||
				// @ts-ignore
				globalThis.CF_PAGES_BRANCH ||
				'master',
			hash:
				process.env.VERCEL_GIT_COMMIT_SHA ||
				process.env.CF_PAGES_COMMIT_SHA ||
				// @ts-ignore
				globalThis.CF_PAGES_COMMIT_SHA ||
				'unknown',

			stripePublishableKey:
				process.env.STRIPE_PUBLISHABLE_KEY ||
				globalThis.STRIPE_PUBLISHABLE_KEY ||
				'pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b',
		},
	},
	typescript: {
		shim: false,
		strict: true,
		typeCheck: false,
		tsConfig: {
			compilerOptions: {
				moduleResolution: 'bundler',
				allowImportingTsExtensions: true,
			},
		},
	},
	modules: ['@vintl/nuxt', '@pinia/nuxt'],
	vintl: {
		defaultLocale: 'en-US',
		locales: [
			{
				tag: 'en-US',
				meta: {
					static: {
						iso: 'en',
					},
				},
			},
		],
		storage: 'cookie',
		parserless: 'only-prod',
		seo: {
			defaultLocaleHasParameter: false,
		},
		onParseError({ error, message, messageId, moduleId, parseMessage, parserOptions }) {
			const errorMessage = String(error)
			const modulePath = relative(__dirname, moduleId)

			try {
				const fallback = parseMessage(message, { ...parserOptions, ignoreTag: true })

				consola.warn(
					`[i18n] ${messageId} in ${modulePath} cannot be parsed normally due to ${errorMessage}. The tags will will not be parsed.`,
				)

				return fallback
			} catch (err) {
				const secondaryErrorMessage = String(err)

				const reason =
					errorMessage === secondaryErrorMessage
						? errorMessage
						: `${errorMessage} and ${secondaryErrorMessage}`

				consola.warn(
					`[i18n] ${messageId} in ${modulePath} cannot be parsed due to ${reason}. It will be skipped.`,
				)
			}
		},
	},
	nitro: {
		moduleSideEffects: ['@vintl/compact-number/locale-data'],
		rollupConfig: {
			// @ts-expect-error it's not infinite.
			plugins: [serverSidedVue()],
		},
	},
	devtools: {
		enabled: true,
	},
	css: ['~/assets/styles/tailwind.css'],
	postcss: {
		plugins: {
			tailwindcss: {},
			autoprefixer: {},
		},
	},
	routeRules: {
		'/**': {
			headers: {
				'Accept-CH': 'Sec-CH-Prefers-Color-Scheme',
				'Critical-CH': 'Sec-CH-Prefers-Color-Scheme',
			},
		},
		'/dashboard/revenue/withdraw': {
			redirect: {
				to: '/dashboard/revenue',
				statusCode: 410,
			},
		},
		'/email/**': {
			redirect: '/_internal/templates/email/**',
		},
		'/_internal/templates/email/**': {
			prerender: true,
			headers: {
				'Content-Type': 'text/html',
				'Cache-Control': 'public, max-age=3600',
			},
		},
		'/_internal/templates/doc/**': {
			prerender: true,
			headers: {
				'Content-Type': 'text/html',
				'Cache-Control': 'public, max-age=3600',
			},
		},
	},
	compatibilityDate: '2024-07-03',
	telemetry: false,
})

function getApiUrl() {
	// @ts-ignore
	return process.env.BROWSER_BASE_URL ?? globalThis.BROWSER_BASE_URL ?? STAGING_API_URL
}

function isProduction() {
	return process.env.NODE_ENV === 'production'
}

function getFeatureFlagOverrides() {
	return JSON.parse(process.env.FLAG_OVERRIDES ?? '{}')
}

function getDomain() {
	if (process.env.NODE_ENV === 'production') {
		if (process.env.SITE_URL) {
			return process.env.SITE_URL
		}
		// @ts-ignore
		else if (process.env.CF_PAGES_URL || globalThis.CF_PAGES_URL) {
			// @ts-ignore
			return process.env.CF_PAGES_URL ?? globalThis.CF_PAGES_URL
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
		const port = process.env.PORT || 3000
		return `http://localhost:${port}`
	}
}
