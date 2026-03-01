<script setup lang="ts">
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
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'

import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'
import HooksSettings from '@/components/ui/instance_settings/HooksSettings.vue'
import InstallationSettings from '@/components/ui/instance_settings/InstallationSettings.vue'
import JavaSettings from '@/components/ui/instance_settings/JavaSettings.vue'
import WindowSettings from '@/components/ui/instance_settings/WindowSettings.vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { get_project_v3 } from '@/helpers/cache'

import type { InstanceSettingsTabProps } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const isMinecraftServer = ref(false)

watch(
	() => props.instance,
	(instance) => {
		isMinecraftServer.value = false
		if (instance.linked_data?.project_id) {
			get_project_v3(instance.linked_data.project_id, 'must_revalidate')
				.then((project: any) => {
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

const modal = ref()

function show() {
	modal.value.show()
}

defineExpose({ show })
</script>
<template>
	<ModalWrapper ref="modal">
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
			:tabs="
				tabs.map((tab) => ({ ...tab, props: { ...props, isMinecraftServer: isMinecraftServer } }))
			"
		/>
	</ModalWrapper>
</template>
