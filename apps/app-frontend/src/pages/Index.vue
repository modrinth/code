<script setup lang="ts">
import { injectNotificationManager } from '@icarus/ui'
import type { SearchResult } from '@icarus/utils'
import dayjs from 'dayjs'
import { computed, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import RowDisplay from '@/components/RowDisplay.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { get_search_results } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events'
import { list } from '@/helpers/profile.js'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])

const taxphobiaModpacks = ref<SearchResult[]>([])
const quickPlayInstances = computed(() =>
	instances.value
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played)))
		.slice(0, 10),
)
const recentInstances = computed(() => instances.value.filter((x) => x.last_played))

const offline = ref<boolean>(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

async function fetchInstances() {
	instances.value = await list().catch(handleError)
}

async function fetchTaxphobiaModpacks() {
	const response = await get_search_results(
		`?facets=[["project_type:modpack"],["author:fraa2a"]]&limit=10&index=downloads`,
	)

	if (response) {
		taxphobiaModpacks.value = response.result.hits
	} else {
		taxphobiaModpacks.value = []
	}
}

await fetchInstances()
await fetchTaxphobiaModpacks()

const unlistenProfile = await profile_listener(
	async (e: { event: string; profile_path_id: string }) => {
		await fetchInstances()

		if (e.event === 'added' || e.event === 'created' || e.event === 'removed') {
			await fetchTaxphobiaModpacks()
		}
	},
)

onUnmounted(() => {
	unlistenProfile()
})
</script>

<template>
	<div class="p-6 flex flex-col gap-2">
		<h1 v-if="recentInstances?.length > 0" class="m-0 text-2xl font-extrabold">Welcome back!</h1>
		<h1 v-else class="m-0 text-2xl font-extrabold">Welcome to Icarus Launcher!</h1>
		<RecentWorldsList :recent-instances="recentInstances" />
		<RowDisplay
			:instances="[
				{
					label: 'Popular Taxphobia Modpacks',
					route: '/taxphobia',
					instances: taxphobiaModpacks,
					downloaded: false,
				},
				{
					label: 'Quick Play',
					route: '/library',
					instance: true,
					instances: quickPlayInstances,
					downloaded: true,
				},
			]"
			:can-paginate="true"
		/>
	</div>
</template>
