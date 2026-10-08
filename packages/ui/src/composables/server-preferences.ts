import { useStorage } from '@vueuse/core'

export function useServerPreferences(serverId: string) {
	return useStorage(
		`pyro-server-${serverId}-preferences`,
		{
			hideSubdomainLabel: false,
			ramAsNumber: false,
			warnOnIncompatibleContent: true,
		},
		undefined,
		{ mergeDefaults: true },
	)
}
