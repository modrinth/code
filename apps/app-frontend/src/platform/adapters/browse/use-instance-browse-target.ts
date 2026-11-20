import type { BrowseTarget, FilterValue, MessageDescriptor } from '@modrinth/ui'
import { commonMessages, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { getInstanceIconUrl } from '@/helpers/instance'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import {
	instanceDetailQueryOptions,
	instanceLinkedProjectQueryOptions,
} from '@/pages/instance/query-options'

import { installedProjectIdsQueryOptions, settleQuery } from './query-options'

const MOD_LOADERS = ['fabric', 'forge', 'quilt', 'neoforge']

const messages = defineMessages({
	addServersToInstance: {
		id: 'app.browse.add-servers-to-instance',
		defaultMessage: 'Adding server to instance',
	},
	backToInstance: {
		id: 'app.browse.back-to-instance',
		defaultMessage: 'Back to instance',
	},
	serverInstanceContentWarning: {
		id: 'app.browse.server-instance-content-warning',
		defaultMessage:
			'Adding content may prevent you from joining this server. Any content you add will be removed when the managed server content is updated.',
	},
})

function negativeProjectFilters(projectIds: Iterable<string>): FilterValue[] {
	return Array.from(projectIds, (id) => ({
		type: 'project_id',
		option: `project_id:${id}`,
		negative: true,
	}))
}

export interface InstanceBrowseTargetOptions {
	projectType: Readonly<Ref<string>>
	isFromWorlds: Readonly<Ref<boolean>>
}

export function instanceBrowseHeadingMessage(fromWorlds: boolean): MessageDescriptor {
	return fromWorlds ? messages.addServersToInstance : commonMessages.installingContentLabel
}

export function useInstanceBrowseTarget({
	projectType,
	isFromWorlds,
}: InstanceBrowseTargetOptions) {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const route = useRoute()
	const appSettings = useAppSettings()

	const instanceId = computed(() => String(route.query.i ?? ''))

	const instanceQuery = useQuery(
		computed(() => ({
			...instanceDetailQueryOptions(instanceId.value),
			enabled: !!instanceId.value,
		})),
	)
	const instance = computed(() => (instanceId.value ? (instanceQuery.data.value ?? null) : null))

	const linkedProjectId = computed(() => instance.value?.link?.project_id ?? '')
	const linkedProjectQuery = useQuery(
		computed(() => ({
			...instanceLinkedProjectQueryOptions(linkedProjectId.value),
			enabled: !!linkedProjectId.value,
		})),
	)
	const isServerInstance = computed(
		() => !!linkedProjectId.value && linkedProjectQuery.data.value?.minecraft_server != null,
	)

	const installedIdsQuery = useQuery(
		computed(() => installedProjectIdsQueryOptions(instanceId.value, isFromWorlds.value)),
	)

	watch([instanceQuery.error, linkedProjectQuery.error, installedIdsQuery.error], (errors) => {
		const error = errors.find(Boolean)
		if (error) handleError(error)
	})

	const newlyInstalled = ref<string[]>([])
	const installedIds = computed(
		() => new Set([...newlyInstalled.value, ...(installedIdsQuery.data.value ?? [])]),
	)

	function markInstalled(projectIds: string[]) {
		newlyInstalled.value = Array.from(new Set([...newlyInstalled.value, ...projectIds]))
	}

	const hideInstalled = ref(route.query.ai === 'true')
	const hideInstalledModpacks = computed({
		get: () => appSettings.hideInstalledModpacks,
		set: (value: boolean) => {
			appSettings.hideInstalledModpacks = value
			getSettings()
				.then((settings) => {
					settings.hide_installed_modpacks = value
					return setSettings(settings)
				})
				.catch(handleError)
		},
	})

	const hiddenIds = ref(new Set<string>())
	const hiddenIdsInitialized = ref(false)

	function syncHiddenIds() {
		hiddenIds.value = new Set(installedIds.value)
		hiddenIdsInitialized.value = true
	}

	watch(instanceId, () => {
		newlyInstalled.value = []
		hiddenIdsInitialized.value = false
		hideInstalled.value = route.query.ai === 'true'
	})
	watch(
		() => installedIdsQuery.data.value,
		(ids) => {
			if (ids && !hiddenIdsInitialized.value) syncHiddenIds()
		},
		{ immediate: true },
	)

	const hidingInstalled = computed(() =>
		projectType.value === 'modpack' ? hideInstalledModpacks.value : hideInstalled.value,
	)
	watch(hidingInstalled, (hiding) => {
		if (hiding) syncHiddenIds()
	})

	const providedFilters = computed<FilterValue[]>(() => {
		const filters: FilterValue[] = []
		const current = instance.value
		const type = projectType.value

		if (current && type !== 'resourcepack') {
			const isVanillaShader = type === 'shader' && current.loader === 'vanilla'
			if (current.game_version && !isVanillaShader) {
				filters.push({ type: 'game_version', option: current.game_version })
			}
			if (type === 'mod' && MOD_LOADERS.includes(current.loader)) {
				filters.push({ type: 'mod_loader', option: current.loader })
			}
			if (isVanillaShader) filters.push({ type: 'shader_loader', option: 'vanilla' })
			if (isServerInstance.value) filters.push({ type: 'environment', option: 'client' })
		}

		if ((current || type === 'modpack') && hidingInstalled.value) {
			filters.push(...negativeProjectFilters(hiddenIds.value))
		}

		return filters
	})

	const target = computed<BrowseTarget | null>(() => {
		const current = instance.value
		if (!current) return null
		return {
			name: current.name,
			loader: current.loader,
			gameVersion: current.game_version,
			iconSrc: getInstanceIconUrl(current.icon_path),
			tintBy: current.id,
			heading: formatMessage(instanceBrowseHeadingMessage(isFromWorlds.value)),
			warning:
				isServerInstance.value && current.loader !== 'vanilla' && !isFromWorlds.value
					? formatMessage(messages.serverInstanceContentWarning)
					: undefined,
			back: {
				to: `/instance/${encodeURIComponent(current.id)}${isFromWorlds.value ? '/worlds' : ''}`,
				label: formatMessage(messages.backToInstance),
			},
		}
	})

	useAppEvent('instance', async (event) => {
		const listChanged = event.event === 'created' || event.event === 'removed'
		const targetSynced =
			!!instance.value && event.instance_id === instance.value.id && event.event === 'synced'
		if ((listChanged && !instanceId.value) || targetSynced) {
			await installedIdsQuery.refetch()
			if (event.event === 'removed' && projectType.value === 'modpack') syncHiddenIds()
		}
	})

	async function ready() {
		if (!instanceId.value) {
			await settleQuery(installedIdsQuery)
			return
		}
		await Promise.all([settleQuery(instanceQuery), settleQuery(installedIdsQuery)])
		if (linkedProjectId.value) await settleQuery(linkedProjectQuery)
	}

	return {
		instance,
		isServerInstance,
		installedIds,
		markInstalled,
		hideInstalled,
		hideInstalledModpacks,
		hiddenIds,
		syncHiddenIds,
		providedFilters,
		target,
		ready,
	}
}
