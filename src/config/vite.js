import Icons from 'unplugin-icons/vite'
import svelteSvg from '@poppanator/sveltekit-svg'

export const plugins = [
	svelteSvg(),
	Icons({
		compiler: 'svelte',
		defaultClass: 'icon',
		scale: 1,
	}),
]
