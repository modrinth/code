<script setup lang="ts">
import type { Product } from '@modrinth/blog'
import { ChangelogEntry, NavTabs } from '@modrinth/ui'
import Timeline from '@modrinth/ui/src/components/base/Timeline.vue'
import { useQuery } from '@tanstack/vue-query'

import { resolveChangelogEntries, type AppRelease } from '~/helpers/changelog'

const route = useRoute()

const filter = ref<Product | undefined>(undefined)
const { data: appReleases, suspense: appReleasesSuspense } = useQuery({
	queryKey: ['changelog', 'app-releases'],
	queryFn: () => $fetch<AppRelease[]>('/api/changelog/app-releases'),
})

onServerPrefetch(async () => {
	await appReleasesSuspense().catch(() => {})
})

function updateFilter() {
	if (route.query.filter) {
		filter.value = route.query.filter as Product
	} else {
		filter.value = undefined
	}
}

updateFilter()

watch(
	() => route.query,
	() => updateFilter(),
)

const changelogEntries = computed(() =>
	resolveChangelogEntries(appReleases.value ?? []).filter(
		(entry) => !filter.value || entry.product === filter.value,
	),
)
</script>

<template>
	<NavTabs
		:links="[
			{
				label: 'All',
				href: '',
			},
			{
				label: 'Platform',
				href: 'web',
			},
			{
				label: 'Hosting',
				href: 'hosting',
			},
			{
				label: 'App',
				href: 'app',
			},
		]"
		query="filter"
		class="mb-4"
	/>
	<Timeline fade-out-end>
		<ChangelogEntry
			v-for="(entry, index) in changelogEntries"
			:key="`${entry.product}-${entry.version ?? entry.date.unix()}`"
			:entry="entry"
			:first="index === 0"
			:show-type="filter === undefined"
			has-link
		/>
	</Timeline>
</template>
