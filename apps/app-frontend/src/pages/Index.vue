<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import LibrarySection from '@/components/ui/library/index.vue'
import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { instance_listener } from '@/helpers/events'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()

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
</script>

<template>
	<WelcomeScreen v-if="isReady && !hasCreatedInstance" />
	<div v-else-if="isReady" class="flex flex-col gap-6 p-6">
		<RecentWorldsList v-if="recentInstances?.length > 0" :recent-instances="recentInstances" />
		<LibrarySection :instances="instances" />
	</div>
</template>
