<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronRightIcon,
	CodeIcon,
	CoffeeIcon,
	InfoIcon,
	MonitorIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	defineMessage,
	NewModal,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'
import HooksSettings from '@/components/ui/instance_settings/HooksSettings.vue'
import InstallationSettings from '@/components/ui/instance_settings/InstallationSettings.vue'
import JavaSettings from '@/components/ui/instance_settings/JavaSettings.vue'
import WindowSettings from '@/components/ui/instance_settings/WindowSettings.vue'
import { get_project_v3 } from '@/helpers/cache'
import { get_linked_modpack_info } from '@/helpers/profile'

import type { InstanceSettingsTabProps } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()
const emit = defineEmits<{
	unlinked: []
}>()

const isMinecraftServer = ref(false)
const handleUnlinked = () => emit('unlinked')

watch(
	() => props.instance,
	(instance) => {
		isMinecraftServer.value = false
		if (instance.linked_data?.project_id) {
			get_project_v3(instance.linked_data.project_id, 'must_revalidate')
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

const tabs = computed<TabbedModalTab<InstanceSettingsTabProps>[]>(() => [
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

const queryClient = useQueryClient()
const modal = ref()
const tabbedModal = useTemplateRef('tabbedModal')

function show(tabIndex?: number) {
	if (props.instance.linked_data?.project_id) {
		queryClient.prefetchQuery({
			queryKey: ['linkedModpackInfo', props.instance.path],
			queryFn: () => get_linked_modpack_info(props.instance.path, 'stale_while_revalidate'),
		})
	}
	modal.value.show()
	if (tabIndex !== undefined) {
		nextTick(() => tabbedModal.value?.setTab(tabIndex))
	}
}

defineExpose({ show })
</script>
<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-semibold text-primary">
				<Avatar
					:src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
					size="24px"
					:tint-by="props.instance.path"
				/>
				{{ instance.name }} <ChevronRightIcon />
				<span class="font-extrabold text-contrast">{{
					formatMessage(commonMessages.settingsLabel)
				}}</span>
			</span>
		</template>

		<TabbedModal
			ref="tabbedModal"
			:tabs="
				tabs.map((tab) => ({
					...tab,
					props: {
						...props,
						isMinecraftServer,
						onUnlinked: handleUnlinked,
					},
				}))
			"
		/>
	</NewModal>
</template>
