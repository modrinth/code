import type { ServerSetupModalHandle } from '@modrinth/ui'
import { isServerContentEnvironmentProjectType } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useServerInstallContent } from './server-install-content'
import { useBrowseBreadcrumbs } from './use-browse-breadcrumbs'
import { useInstanceBrowseTarget } from './use-instance-browse-target'
import { useServerBrowseFilters } from './use-server-browse-filters'

const SETUP_PROJECT_TYPES = ['modpack', 'mod', 'plugin', 'datapack']

export type AppBrowseTargets = ReturnType<typeof useAppBrowseTargets>

export function useAppBrowseTargets({
	setupModal,
}: {
	setupModal: Ref<ServerSetupModalHandle | null>
}) {
	const route = useRoute()
	const router = useRouter()

	const active = computed(() => route.path.startsWith('/browse/'))
	const projectType = ref(String(route.params.projectType ?? ''))

	const server = useServerInstallContent({ setupModal })
	const { serverInstall } = server
	const instanceTarget = useInstanceBrowseTarget({
		projectType,
		isFromWorlds: server.isFromWorlds,
	})
	const serverFilters = useServerBrowseFilters({
		server,
		projectType,
		hideInstalledModpacks: instanceTarget.hideInstalledModpacks,
		hiddenModpackIds: instanceTarget.hiddenIds,
	})

	useBrowseBreadcrumbs({
		instance: instanceTarget.instance,
		serverName: computed(() => server.serverData.value?.name),
		serverBackUrl: computed(() => String(serverInstall.back.value.to)),
	})

	function allowedInSetup(type: string) {
		if (!serverInstall.isSetup.value || SETUP_PROJECT_TYPES.includes(type)) return true
		void router.replace({ path: '/browse/modpack', query: route.query })
		return false
	}

	if (server.isFromWorlds.value && route.params.projectType !== 'server') {
		void router.replace({ path: '/browse/server', query: route.query })
	}
	allowedInSetup(projectType.value)

	watch(
		() => route.params.projectType,
		(next) => {
			if (!active.value) return
			const type = String(next ?? '')
			if (!allowedInSetup(type) || !type) return
			projectType.value = type
		},
	)

	watch(
		[serverFilters.environment, projectType, serverFilters.showEnvironment],
		([environment, type, show]) => {
			if (!show || !active.value) return
			if (type !== 'modpack' && isServerContentEnvironmentProjectType(environment, type)) return
			void router.replace({
				path: '/browse/mod',
				query: { ...route.query, env: environment ?? undefined },
			})
		},
		{ immediate: true },
	)

	function providedFiltersFor(type: string) {
		return server.isServerContext.value
			? serverFilters.providedFiltersFor(type)
			: instanceTarget.providedFiltersFor(type)
	}
	const providedFilters = computed(() => providedFiltersFor(projectType.value))

	const target = computed(() =>
		server.isServerContext.value ? server.target.value : instanceTarget.target.value,
	)

	async function ready() {
		await Promise.all([server.ready(), instanceTarget.ready()])
	}

	return {
		active,
		projectType,
		server,
		instanceTarget,
		serverFilters,
		providedFilters,
		providedFiltersFor,
		target,
		ready,
	}
}
