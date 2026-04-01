<script setup lang="ts">
import { getChangelog, type Product } from '@modrinth/blog'
import { ChangelogEntry, NavTabs } from '@modrinth/ui'
import Timeline from '@modrinth/ui/src/components/base/Timeline.vue'

const route = useRoute()

const filter = ref<Product | undefined>(undefined)
const allChangelogEntries = ref(getChangelog())

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
	/>
	<Timeline
		fade-out-end
		class="mx-4 rounded-[4px] border border-solid border-[#b5b5b5] bg-[#b5b5b5] p-3"
	>
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
