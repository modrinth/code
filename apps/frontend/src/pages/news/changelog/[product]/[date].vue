<script setup lang="ts">
import { ChevronLeftIcon } from '@modrinth/assets'
import { ChangelogEntry, Timeline } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import {
	findChangelogEntry,
	resolveChangelogEntries,
	type AppRelease,
} from '~/helpers/changelog'

const route = useRoute()

const { data: appReleases, suspense: appReleasesSuspense } = useQuery({
	queryKey: ['changelog', 'app-releases'],
	queryFn: () => $fetch<AppRelease[]>('/api/changelog/app-releases'),
})

await appReleasesSuspense().catch(() => {})

const changelogEntries = computed(() => resolveChangelogEntries(appReleases.value ?? []))
const changelogEntry = computed(() =>
	route.params.date && route.params.product
		? findChangelogEntry(changelogEntries.value, route.params.product, route.params.date)
		: undefined,
)

const isFirst = computed(() => changelogEntry.value === changelogEntries.value[0])

if (!changelogEntry.value) {
	throw createError({ statusCode: 404, statusMessage: 'Version not found' })
}
</script>

<template>
	<div v-if="changelogEntry">
		<nuxt-link
			:to="`/news/changelog?filter=${changelogEntry.product}`"
			class="mb-4 mt-4 flex w-fit items-center gap-2 text-link"
		>
			<ChevronLeftIcon /> View full changelog
		</nuxt-link>
		<Timeline fade-out-end :fade-out-start="!isFirst">
			<ChangelogEntry :entry="changelogEntry" :first="isFirst" show-type />
		</Timeline>
	</div>
</template>
