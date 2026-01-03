import type { Cosmetics } from '~/plugins/cosmetics.ts'

export function useTheme() {
	return useNuxtApp().$theme
}

export function useCosmetics() {
	return useNuxtApp().$cosmetics as Ref<Cosmetics>
}
