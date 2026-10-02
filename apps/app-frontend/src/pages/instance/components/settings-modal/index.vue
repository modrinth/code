<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronRightIcon,
	CodeIcon,
	CoffeeIcon,
	InfoIcon,
	Settings2Icon,
	ShieldIcon,
	UsersIcon,
	WindowIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	defineMessage,
	defineMessages,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import type { PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch } from 'vue'

import { get_project_v3 } from '@/helpers/cache'
import { get_linked_modpack_info, getInstanceIconUrl } from '@/helpers/instance'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'

import GeneralSettings from './general-settings.vue'
import HooksSettings from './hooks-settings.vue'
import InstallationSettings from './installation-settings.vue'
import { provideInstanceSettings } from './instance-settings-context.ts'
import JavaSettings from './java-settings.vue'
import SandboxSettings from './sandbox-settings.vue'
import SharingSettings from './sharing-settings.vue'
import SyncedOptionsSettings from './synced-options-settings.vue'
import WindowSettings from './window-settings.vue'

const { formatMessage } = useVIntl()
const queryClient = useQueryClient()

const props = defineProps<{
	instance: GameInstance
	offline?: boolean
}>()
const emit = defineEmits<{
	unlinked: []
}>()

const isMinecraftServer = ref(false)
const handleUnlinked = () => emit('unlinked')

const instanceRef = computed(() => props.instance)
const tabbedModal = ref<InstanceType<typeof TabbedModal> | null>(null)
let onAfterClose: (() => void) | undefined

function hide(callback?: () => void) {
	onAfterClose = callback
	if (!tabbedModal.value?.hide()) onAfterClose = undefined
}

function handleAfterHide() {
	const callback = onAfterClose
	onAfterClose = undefined
	callback?.()
}

provideInstanceSettings({
	instance: instanceRef,
	offline: props.offline,
	isMinecraftServer,
	onUnlinked: handleUnlinked,
	closeModal: hide,
})

watch(
	() => props.instance,
	(instance) => {
		isMinecraftServer.value = false
		if (instance.link?.project_id) {
			get_project_v3(instance.link.project_id, 'must_revalidate')
				.then((project: Labrinth.Projects.v3.Project | undefined) => {
					if (project?.minecraft_server != null) {
						isMinecraftServer.value = true
					}
				})
				.catch(() => {})
		}
	},
	{ immediate: true },
)

const tabCategories = defineMessages({
	instance: {
		id: 'instance.settings.sidebar.label.instance',
		defaultMessage: 'Instance',
	},
	game: {
		id: 'instance.settings.sidebar.label.game',
		defaultMessage: 'Game',
	},
})

const tabs = computed<TabbedModalTab[]>(() => [
	{
		name: defineMessage({
			id: 'instance.settings.tabs.general',
			defaultMessage: 'General',
		}),
		icon: InfoIcon,
		category: tabCategories.instance,
		content: GeneralSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.installation',
			defaultMessage: 'Installation',
		}),
		icon: WrenchIcon,
		category: tabCategories.instance,
		content: InstallationSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.sharing',
			defaultMessage: 'Sharing',
		}),
		category: tabCategories.instance,
		icon: UsersIcon,
		content: SharingSettings,
		shown: props.instance.shared_instance?.role === 'owner' && !props.instance.quarantined,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.sandbox',
			defaultMessage: 'Sandboxing',
		}),
		icon: ShieldIcon,
		category: tabCategories.game,
		content: SandboxSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.java.label',
			defaultMessage: 'Java and memory',
		}),
		icon: CoffeeIcon,
		category: tabCategories.game,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.window.label',
			defaultMessage: 'Window',
		}),
		icon: WindowIcon,
		category: tabCategories.game,
		content: WindowSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.hooks.label',
			defaultMessage: 'Launch hooks',
		}),
		icon: CodeIcon,
		category: tabCategories.game,
		content: HooksSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.syncing',
			defaultMessage: 'Syncing',
		}),
		icon: Settings2Icon,
		category: tabCategories.game,
		content: SyncedOptionsSettings,
	},
])

function getSupportedModpackLoaders() {
	return get_loaders().then((value: PlatformTag[]) =>
		value
			.filter((item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla')
			.sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0)),
	)
}

// Preload
useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'fabric'],
	queryFn: () => get_loader_versions('fabric'),
})
useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'forge'],
	queryFn: () => get_loader_versions('forge'),
})
useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'quilt'],
	queryFn: () => get_loader_versions('quilt'),
})
useQuery({
	queryKey: ['instance-settings', 'loader-versions', 'neo'],
	queryFn: () => get_loader_versions('neo'),
})
useQuery({
	queryKey: ['instance-settings', 'game-versions'],
	queryFn: get_game_versions,
})
useQuery({
	queryKey: ['instance-settings', 'loaders', 'modpack'],
	queryFn: getSupportedModpackLoaders,
})
useQuery({
	queryKey: computed(() => ['linkedModpackInfo', props.instance.id]),
	queryFn: () => get_linked_modpack_info(props.instance.id, 'stale_while_revalidate'),
	enabled: computed(() => !!props.instance.link?.project_id && !props.offline),
})

function show(tabIndex?: number) {
	if (props.instance.link?.project_id) {
		queryClient.prefetchQuery({
			queryKey: ['linkedModpackInfo', props.instance.id],
			queryFn: () => get_linked_modpack_info(props.instance.id, 'stale_while_revalidate'),
		})
	}
	tabbedModal.value?.show()
	if (tabIndex !== undefined) {
		nextTick(() => tabbedModal.value?.setTab(tabIndex))
	}
}

defineExpose({ show, hide })
</script>
<template>
	<TabbedModal
		ref="tabbedModal"
		:tabs="tabs"
		:on-after-hide="handleAfterHide"
		max-width="928px"
		width="928px"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-semibold text-primary">
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					size="24px"
					:tint-by="props.instance.id"
					pad-transparent-corners
				/>
				{{ instance.name }} <ChevronRightIcon />
				<span class="font-extrabold text-contrast">{{
					formatMessage(commonMessages.settingsLabel)
				}}</span>
			</span>
		</template>
	</TabbedModal>
</template>
