<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, inject, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import LibrarySection from '@/components/ui/library/index.vue'
import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { instance_listener } from '@/helpers/events'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const showCreationModal = inject<() => void>('showCreationModal')
const pageOptions = ref<InstanceType<typeof ContextMenu>>()

const messages = defineMessages({
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
})

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

async function fetchInstances() {
	try {
		instances.value = await list()
	} catch (error: unknown) {
		handleError(error)
	}
}

if (hasCreatedInstance.value) {
	await fetchInstances()
}

const unlistenInstance = await instance_listener(fetchInstances)

onUnmounted(() => {
	unlistenInstance()
})

function openPageContextMenu(event: MouseEvent) {
	if (
		!(event.target instanceof HTMLElement) ||
		!event.target.hasAttribute('data-library-page-background')
	) {
		return
	}

	event.preventDefault()
	event.stopPropagation()
	pageOptions.value?.showMenu(event, {}, [{ name: 'new_instance' }])
}

function handlePageOption({ option }: { option: string }) {
	if (option === 'new_instance') {
		showCreationModal?.()
	}
}
</script>

<template>
	<WelcomeScreen v-if="isReady && !hasCreatedInstance" />
	<div
		v-else-if="isReady"
		data-library-page-background
		class="flex flex-col gap-6 p-6"
		@contextmenu="openPageContextMenu"
	>
		<RecentWorldsList v-if="recentInstances?.length > 0" :recent-instances="recentInstances" />
		<LibrarySection :instances="instances" />
		<ContextMenu ref="pageOptions" @option-clicked="handlePageOption">
			<template #new_instance>
				<PlusIcon /> {{ formatMessage(messages.newInstance) }}
			</template>
		</ContextMenu>
	</div>
</template>
