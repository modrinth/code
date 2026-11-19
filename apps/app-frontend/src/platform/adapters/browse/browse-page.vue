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
	useStickyObserver,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_loader_versions as getLoaderManifest } from '@/helpers/metadata'

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

const setupModal = ref<ServerSetupModalHandle | null>(null)
const targets = useAppBrowseTargets({ setupModal })
const { active, server, target } = targets
const { serverInstall } = server

const [tags] = await Promise.all([loadBrowseTags(handleError), targets.ready()])

const { contextMenuRef } = useAppBrowseManager({ tags, targets })

const stickyHeaderRef = ref<HTMLElement | null>(null)
const { isStuck: isHeaderStuck } = useStickyObserver(stickyHeaderRef, 'BrowseInstallHeader')
</script>

<template>
	<div class="flex flex-col gap-2 p-6">
		<div
			v-if="target"
			ref="stickyHeaderRef"
			class="sticky top-0 z-20 -mx-6 -mt-6 mb-4 rounded-tl-[--radius-xl] border-0 border-b border-solid bg-surface-1 px-6 py-4 border-surface-5"
			:class="[isHeaderStuck ? 'border-t' : '']"
		>
			<BrowseInstallHeader
				:target="target"
				:selection="serverInstall.selection"
				:skip-leave-prompt="appSettings.skipNonEssentialWarnings"
			/>
		</div>
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
