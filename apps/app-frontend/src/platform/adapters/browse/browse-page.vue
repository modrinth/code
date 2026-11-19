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
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_loader_versions as getLoaderManifest } from '@/helpers/metadata'
import InstancePageHeaderMetadataItems from '@/pages/instance/components/page-header/instance-page-header-metadata-items.vue'

import StickyBrowseHeader from './sticky-browse-header.vue'
import { loadBrowseTags, useAppBrowseManager } from './use-app-browse-manager'
import { useAppBrowseTargets } from './use-app-browse-targets'

const messages = defineMessages({
	projectActionsLabel: {
		id: 'app.browse.project-actions.label',
		defaultMessage: 'Project actions',
	},
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const appSettings = useAppSettings()
const queryClient = useQueryClient()

const setupModal = ref<ServerSetupModalHandle | null>(null)
const targets = useAppBrowseTargets({ setupModal })
const { active, server, target } = targets
const { instance } = targets.instanceTarget
const { serverInstall } = server

const [tags] = await Promise.all([loadBrowseTags(queryClient, handleError), targets.ready()])

const { contextMenuRef } = useAppBrowseManager({ tags, targets })
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
					<InstancePageHeaderMetadataItems
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
