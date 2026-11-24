<script setup lang="ts">
import { PlayIcon, PlusIcon } from '@modrinth/assets'
import {
	ContextMenu,
	defineMessages,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useSessionStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { injectInstanceCreation } from '@/features/instances/instance-creation-context'
import { instanceListQueryOptions } from '@/features/instances/queries'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { prefetchSharedInstanceLibrary } from '@/features/library/shared-instances/queries'
import { useSharedInstanceLibrarySync } from '@/features/library/shared-instances/use-shared-instance-library-sync'
import { useUninstalledSharedInstances } from '@/features/library/shared-instances/use-uninstalled-shared-instances'
import { traceStartupStep } from '@/helpers/startup-debug'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'

import LibrarySection from './components/LibrarySection.vue'
import RecentWorldsList from './components/recent-worlds-list/index.vue'
import WelcomeScreen from './components/WelcomeScreen.vue'

defineOptions({
	name: 'LibraryPage',
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const instanceCreation = injectInstanceCreation()
const pageOptions = ref<InstanceType<typeof ContextMenu>>()
const appSettings = useAppSettings()
const { hiddenIds: hiddenInstanceIds } = useHiddenInstances()
const auth = injectAuth()
const modrinthClient = injectModrinthClient()
const queryClient = useQueryClient()
const scrollTop = useSessionStorage('library-scroll-top', 0)

const getViewport = () => document.querySelector('.app-viewport')

onBeforeRouteLeave(() => {
	scrollTop.value = getViewport()?.scrollTop ?? 0
})

function restoreScroll() {
	requestAnimationFrame(() => getViewport()?.scrollTo(0, scrollTop.value))
}

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

useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: formatMessage(messages.home),
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})

watch(
	() => (isReady.value && !hasCreatedInstance.value ? auth.user.value?.id : null),
	(userId) => {
		if (!userId) return
		prefetchSharedInstanceLibrary(queryClient, modrinthClient, userId).catch((error) =>
			console.warn('Failed to prefetch shared instance library:', error),
		)
	},
	{ immediate: true },
)

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])
const uninstalledSharedInstances = useUninstalledSharedInstances(instances)
const { sharedInstances } = uninstalledSharedInstances
useSharedInstanceLibrarySync(uninstalledSharedInstances)

if (hasCreatedInstance.value) {
	await traceStartupStep('Load library instances', () => instancesQuery.suspense()).catch(
		handleError,
	)
}

const showWelcomeScreen = computed(
	() =>
		isReady.value &&
		!hasCreatedInstance.value &&
		!sharedInstances.value.some((instance) => !hiddenInstanceIds.value.has(instance.id)),
)

const recentInstances = computed(() =>
	instances.value
		.filter((instance) => !instance.launch_blocked && !hiddenInstanceIds.value.has(instance.id))
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
			action: () => instanceCreation.showCreationModal(),
		},
	])
}
</script>

<template>
	<WelcomeScreen v-if="showWelcomeScreen" />
	<div
		v-else-if="isReady"
		data-library-page-background
		class="flex flex-col gap-3 p-6"
		@contextmenu="openPageContextMenu"
	>
		<RecentWorldsList
			v-if="recentInstances.length > 0 && appSettings.showJumpIn"
			:recent-instances="recentInstances"
		/>
		<LibrarySection
			:instances="instances"
			:shared-instances="sharedInstances"
			@ready="restoreScroll"
		/>
		<ContextMenu ref="pageOptions" :label="formatMessage(messages.libraryActionsLabel)" />
	</div>
</template>
