import type { FilterValue, ServerContentEnvironment } from '@modrinth/ui'
import { getHostingModEnvironmentOverride, parseServerContentEnvironment } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import type { useServerInstallContent } from './server-install-content'

const MOD_LOADERS = ['fabric', 'forge', 'quilt', 'neoforge']
const PLUGIN_LOADERS = ['paper', 'purpur']

function negativeProjectFilters(projectIds: Iterable<string>): FilterValue[] {
	return Array.from(projectIds, (id) => ({
		type: 'project_id',
		option: `project_id:${id}`,
		negative: true,
	}))
}

export interface ServerBrowseFiltersOptions {
	server: ReturnType<typeof useServerInstallContent>
	projectType: Readonly<Ref<string>>
	hideInstalledModpacks: Readonly<Ref<boolean>>
	hiddenModpackIds: Readonly<Ref<Set<string>>>
}

export function useServerBrowseFilters({
	server,
	projectType,
	hideInstalledModpacks,
	hiddenModpackIds,
}: ServerBrowseFiltersOptions) {
	const route = useRoute()
	const { serverInstall } = server
	const isSetup = serverInstall.isSetup

	const environment = ref<ServerContentEnvironment | null>(
		parseServerContentEnvironment(route.query.env),
	)
	watch(
		() => route.query.env,
		(value) => {
			environment.value = parseServerContentEnvironment(value)
		},
	)
	const showEnvironment = computed(() => server.isServerContext.value && !isSetup.value)
	function environmentOverrideFor(type: string) {
		if (showEnvironment.value && type === 'mod') {
			return getHostingModEnvironmentOverride(environment.value)
		}
		return isSetup.value ? getHostingModEnvironmentOverride(null) : undefined
	}
	const environmentOverride = computed(() => environmentOverrideFor(projectType.value))

	const hideInstalled = ref(route.query.shi === 'true')
	const hideSelected = ref(false)
	watch(
		() => serverInstall.plans.value.size,
		(count) => {
			if (count === 0) hideSelected.value = false
		},
	)

	const hiddenInstalledIds = ref(new Set<string>())
	const hiddenInstalledIdsInitialized = ref(false)

	function syncHiddenInstalledIds() {
		hiddenInstalledIds.value = new Set(server.installedProjectIds.value)
		hiddenInstalledIdsInitialized.value = true
	}

	watch(
		server.installedProjectIds,
		(ids) => {
			if (ids.size > 0 && !hiddenInstalledIdsInitialized.value) syncHiddenInstalledIds()
		},
		{ immediate: true },
	)
	watch(hideInstalled, (hide) => {
		if (hide) syncHiddenInstalledIds()
	})

	function providedFiltersFor(type: string): FilterValue[] {
		const filters: FilterValue[] = []
		if (!server.serverData.value) return filters

		if (type !== 'modpack' && !isSetup.value) {
			const { gameVersion } = server.platform.value
			if (gameVersion) filters.push({ type: 'game_version', option: gameVersion })

			const loader = server.platform.value.loader?.toLowerCase()
			if (type === 'mod' && loader && MOD_LOADERS.includes(loader)) {
				filters.push({ type: 'mod_loader', option: loader })
			}
			if (type === 'plugin' && loader && PLUGIN_LOADERS.includes(loader)) {
				filters.push({ type: 'plugin_loader', option: loader })
			}
			if (hideSelected.value) {
				filters.push(...negativeProjectFilters(serverInstall.projectIds.value))
			}
		}

		if (type === 'modpack') {
			filters.push({ type: 'environment', option: 'server' })
			if (hideInstalledModpacks.value)
				filters.push(...negativeProjectFilters(hiddenModpackIds.value))
		}

		if (!isSetup.value && hideInstalled.value) {
			filters.push(...negativeProjectFilters(hiddenInstalledIds.value))
		}

		return filters
	}

	const providedFilters = computed(() => providedFiltersFor(projectType.value))

	return {
		environment,
		showEnvironment,
		environmentOverride,
		environmentOverrideFor,
		hideInstalled,
		hideSelected,
		syncHiddenInstalledIds,
		providedFilters,
		providedFiltersFor,
	}
}
