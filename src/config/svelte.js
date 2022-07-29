import sveltePreprocess from 'svelte-preprocess'

export const preprocess = sveltePreprocess({
	postcss: true,
	preserve: ['ld+json'],
})
