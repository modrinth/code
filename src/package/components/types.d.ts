import type { SvelteComponentDev } from 'svelte/internal'

export interface Option {
	label: string
	/** The element that will be in the `value` array while the option is checked */
	value: string | number
	icon?: SvelteComponentDev | string | any // SvelteComponentDev fails to type check
}
