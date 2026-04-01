export default defineNuxtPlugin({
	name: 'theme',
	setup() {
		useHead({ htmlAttrs: { class: 'light-mode' } })
	},
})
