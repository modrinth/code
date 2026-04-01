import type { Cosmetics } from '~/plugins/cosmetics.ts'

export function useCosmetics() {
	return useNuxtApp().$cosmetics as Ref<Cosmetics>
}
