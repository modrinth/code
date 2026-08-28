// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
	srcDir: 'src/',
	compatibilityDate: '2025-07-15',
	devtools: { enabled: true },
	devServer: {
		port: 42069,
	},
	app: {
		head: {
			title: 'Modrinth 2',
			htmlAttrs: {
				lang: 'en',
			},
			link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
		},
	},
	future: {
		compatibilityVersion: 5,
	},
	css: ['~/assets/styles/styles.css'],
	postcss: {
		plugins: {
			tailwindcss: {},
			autoprefixer: {},
		},
	},
})
