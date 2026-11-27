<script setup lang="ts">
import { ExternalIcon, GlobeIcon } from '@modrinth/assets'
import type { ServerSetupModalHandle } from '@modrinth/ui'
import {
	BrowseInstallHeader,
	BrowsePageLayout,
	BrowseSidebar,
	ContextMenu,
	CreationFlowModal,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import InstanceMetadataItems from '@/features/instances/InstanceMetadataItems.vue'
import { get_loader_versions as getLoaderManifest } from '@/helpers/metadata'

import StickyBrowseHeader from '../StickyBrowseHeader.vue'
import { useAppBrowseManager, useBrowseTags } from '../use-app-browse-manager'
import { useAppBrowseTargets } from '../use-app-browse-targets'

const messages = defineMessages({
	projectActionsLabel: {
		id: 'app.browse.project-actions.label',
		defaultMessage: 'Project actions',
	},
})

const { formatMessage } = useVIntl()
const appSettings = useAppSettings()

const setupModal = ref<ServerSetupModalHandle | null>(null)
const browseTags = useBrowseTags()
const targets = useAppBrowseTargets({ setupModal })
const { active, server, target } = targets
const { instance } = targets.instanceTarget
const { serverInstall } = server

await Promise.all([browseTags.ready(), targets.ready()])

const { contextMenuRef } = useAppBrowseManager({ tags: browseTags.tags, targets })
</script>

<template>
	<div class="flex flex-col gap-2 p-6">
		<StickyBrowseHeader v-if="target" class="mb-2">
			<BrowseInstallHeader
				:target="target"
				:selection="serverInstall.selection"
				:skip-leave-prompt="appSettings.skipNonEssentialWarnings"
			>
				<template v-if="!server.isServerContext.value && instance" #metadata>
					<InstanceMetadataItems
						:instance="instance"
						:show-instance-play-time="appSettings.showPlayTime"
					/>
				</template>
			</BrowseInstallHeader>
		</StickyBrowseHeader>
		<BrowsePageLayout dense>
			<template #after>
				<ContextMenu ref="contextMenuRef" :label="formatMessage(messages.projectActionsLabel)">
					<template #open_link="{ option }">
						<GlobeIcon /> {{ option.label }} <ExternalIcon />
					</template>
				</ContextMenu>
			</template>
		</BrowsePageLayout>
		<CreationFlowModal
			v-if="server.isServerContext.value && serverInstall.from.value !== 'onboarding'"
			ref="setupModal"
			:type="serverInstall.from.value === 'reset-server' ? 'reset-server' : 'server-onboarding'"
			:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
			:show-snapshot-toggle="true"
			:on-back="serverInstall.onFlowBack"
			:get-project-versions="server.getServerProjectVersions"
			:get-loader-manifest="getLoaderManifest"
			@hide="serverInstall.onFlowHide"
			@browse-modpacks="() => {}"
			@create="serverInstall.onFlowCreate"
		/>
		<Teleport v-if="active" to="#sidebar-teleport-target">
			<BrowseSidebar dense />
		</Teleport>
	</div>
</template>
