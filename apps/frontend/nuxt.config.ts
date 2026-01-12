import { GenericModrinthClient, type Labrinth } from '@modrinth/api-client'
import { LOCALES } from '@modrinth/ui/src/composables/i18n.ts'
import serverSidedVue from '@vitejs/plugin-vue'
import fs from 'fs/promises'
import { defineNuxtConfig } from 'nuxt/config'
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

const PROD_MODRINTH_URL = 'https://modrinth.com'
const STAGING_MODRINTH_URL = 'https://staging.modrinth.com'

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
		css: {
			preprocessorOptions: {
				scss: {
					// TODO: dont forget about this
					silenceDeprecations: ['import'],
				},
			},
		},
		ssr: {
			// https://github.com/Akryum/floating-vue/issues/809#issuecomment-1002996240
			noExternal: ['v-tooltip'],
		},
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
		build: {
			rollupOptions: {
				external: ['cloudflare:workers'],
			},
		},
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

			const robotsContent =
				getDomain() === PROD_MODRINTH_URL
					? 'User-agent: *\nDisallow: /_internal/'
					: 'User-agent: *\nDisallow: /'

			await fs.writeFile('./src/public/robots.txt', robotsContent)
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
				'main',
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
	modules: ['@nuxtjs/i18n', '@pinia/nuxt', 'floating-vue/nuxt'],
	floatingVue: {
		themes: {
			'ribbit-popout': {
				$extend: 'dropdown',
				placement: 'bottom-end',
				instantMove: true,
				distance: 8,
			},
			'dismissable-prompt': {
				$extend: 'dropdown',
				placement: 'bottom-start',
			},
		},
	},
	i18n: {
		defaultLocale: 'en-US',
		lazy: true,
		langDir: '.',
		locales: LOCALES.map((locale) => ({
			...locale,
			file: 'locale-loader.ts',
		})),
		strategy: 'no_prefix',
		detectBrowserLanguage: {
			useCookie: true,
			cookieKey: 'locale',
			fallbackLocale: 'en-US',
		},
		vueI18n: './i18n.config.ts',
		bundle: {
			optimizeTranslationDirective: false,
		},
	},
	nitro: {
		rollupConfig: {
			// @ts-expect-error because of rolldown-vite - completely fine though
			plugins: [serverSidedVue()],
			external: ['cloudflare:workers'],
		},
		preset: 'cloudflare_module',
		cloudflare: {
			nodeCompat: true,
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
	compatibilityDate: '2025-01-01',
	telemetry: false,
	experimental: {
		asyncContext: isProduction(),
	},
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
		// @ts-ignore
		if (process.env.CF_PAGES_URL || globalThis.CF_PAGES_URL) {
			// @ts-ignore
			return process.env.CF_PAGES_URL ?? globalThis.CF_PAGES_URL
		} else if (process.env.HEROKU_APP_NAME) {
			return `https://${process.env.HEROKU_APP_NAME}.herokuapp.com`
		} else if (process.env.VERCEL_URL) {
			return `https://${process.env.VERCEL_URL}`
		} else if (getApiUrl() === STAGING_API_URL) {
			return STAGING_MODRINTH_URL
		} else {
			return PROD_MODRINTH_URL
		}
	} else {
		const port = process.env.PORT || 3000
		return `http://localhost:${port}`
	}
}
