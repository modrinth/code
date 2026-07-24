<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronRightIcon,
	CodeIcon,
	CoffeeIcon,
	InfoIcon,
	MonitorIcon,
	UsersIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	defineMessage,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import type { PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, nextTick, ref, watch } from 'vue'

import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'
import HooksSettings from '@/components/ui/instance_settings/HooksSettings.vue'
import InstallationSettings from '@/components/ui/instance_settings/InstallationSettings.vue'
import JavaSettings from '@/components/ui/instance_settings/JavaSettings.vue'
import SharingSettings from '@/components/ui/instance_settings/SharingSettings.vue'
import WindowSettings from '@/components/ui/instance_settings/WindowSettings.vue'
import { get_project_v3 } from '@/helpers/cache'
import { get_linked_modpack_info } from '@/helpers/instance'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import { provideInstanceSettings } from '@/providers/instance-settings'

import type { GameInstance } from '../../../helpers/types'

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

function hide() {
	tabbedModal.value?.hide()
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

const tabs = computed<TabbedModalTab[]>(() => [
	{
		name: defineMessage({
			id: 'instance.settings.tabs.general',
			defaultMessage: 'General',
		}),
		icon: InfoIcon,
		content: GeneralSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.installation',
			defaultMessage: 'Installation',
		}),
		icon: WrenchIcon,
		content: InstallationSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.sharing',
			defaultMessage: 'Sharing',
		}),
		icon: UsersIcon,
		content: SharingSettings,
		shown: props.instance.shared_instance?.role === 'owner' && !props.instance.quarantined,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.window',
			defaultMessage: 'Window',
		}),
		icon: MonitorIcon,
		content: WindowSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.java',
			defaultMessage: 'Java and memory',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.hooks',
			defaultMessage: 'Launch hooks',
		}),
		icon: CodeIcon,
		content: HooksSettings,
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
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-semibold text-primary">
				<Avatar
					:src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
					size="24px"
					:tint-by="props.instance.id"
				/>
				{{ instance.name }} <ChevronRightIcon />
				<span class="font-extrabold text-contrast">{{
					formatMessage(commonMessages.settingsLabel)
				}}</span>
			</span>
		</template>
	</TabbedModal>
</template>
