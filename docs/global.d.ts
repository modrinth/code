declare module '$assets/images/*'
declare module '$locales/*'

declare module '*.svg?component' {
	import type { SvelteComponentTyped } from 'svelte/internal'
	class SVGComponent extends SvelteComponentTyped<{ class: string }> {}
	export default SVGComponent
}
