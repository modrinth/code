<script setup lang="ts">
import { PlayIcon, PlusIcon } from '@modrinth/assets'
import { ContextMenu, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, inject, onActivated, ref } from 'vue'

import LibrarySection from '@/components/ui/library/index.vue'
import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { instanceListQueryOptions } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

defineOptions({
	name: 'LibraryPage',
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const showCreationModal = inject<() => void>('showCreationModal')
const pageOptions = ref<InstanceType<typeof ContextMenu>>()
const appSettings = useAppSettings()

const messages = defineMessages({
	home: {
		id: 'app.navigation.home',
		defaultMessage: 'Home',
	},
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
	libraryActionsLabel: {
		id: 'app.library.actions.label',
		defaultMessage: 'Library actions',
	},
})

const homeBreadcrumb = useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: formatMessage(messages.home),
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})
onActivated(homeBreadcrumb.reset)

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])
if (hasCreatedInstance.value) {
	await instancesQuery.suspense().catch(handleError)
}

const recentInstances = computed(() =>
	instances.value
		.slice()
		.sort((a, b) => dayjs(b.last_played ?? b.created).diff(dayjs(a.last_played ?? a.created))),
)

function openPageContextMenu(event: MouseEvent) {
	if (
		!(event.target instanceof HTMLElement) ||
		!event.target.hasAttribute('data-library-page-background')
	) {
		return
	}

	event.preventDefault()
	event.stopPropagation()
	pageOptions.value?.open(event, [
		{
			id: 'new_instance',
			label: formatMessage(messages.newInstance),
			icon: PlusIcon,
			action: () => showCreationModal?.(),
		},
	])
}
</script>

<template>
	<WelcomeScreen v-if="isReady && !hasCreatedInstance" />
	<div
		v-else-if="isReady"
		data-library-page-background
		class="flex flex-col gap-3 p-6"
		@contextmenu="openPageContextMenu"
	>
		<RecentWorldsList
			v-if="recentInstances?.length > 0 && appSettings.getFeatureFlag('worlds_in_home')"
			:recent-instances="recentInstances"
		/>
		<LibrarySection :instances="instances" />
		<ContextMenu ref="pageOptions" :label="formatMessage(messages.libraryActionsLabel)" />
	</div>
</template>
