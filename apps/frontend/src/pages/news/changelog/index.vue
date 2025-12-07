<script setup lang="ts">
import { ChangelogEntry } from '@modrinth/ui'
import Timeline from '@modrinth/ui/src/components/base/Timeline.vue'
import { getChangelog, type Product } from '@modrinth/utils'

import NavTabs from '~/components/ui/NavTabs.vue'

const route = useRoute()
const router = useRouter()

const filter = ref<Product | undefined>(undefined)
const allChangelogEntries = ref(getChangelog())

function updateFilter() {
	if (route.query.filter) {
		let value = route.query.filter
		if (route.query.filter === 'servers') {
			router.push({ query: { ...route.query, filter: 'hosting' } })
			value = 'hosting'
		}
		filter.value = value as Product
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
	allChangelogEntries.value.filter((x) => !filter.value || x.product === filter.value),
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
				label: 'Website',
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
			:key="entry.date"
			:entry="entry"
			:first="index === 0"
			:show-type="filter === undefined"
			has-link
		/>
	</Timeline>
</template>
