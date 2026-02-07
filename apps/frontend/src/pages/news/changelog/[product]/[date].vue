<script setup lang="ts">
import { ChevronLeftIcon } from '@modrinth/assets'
import { getChangelog } from '@modrinth/blog'
import { ChangelogEntry, Timeline } from '@modrinth/ui'

const route = useRoute()

const changelogEntry = computed(() =>
	route.params.date
		? getChangelog().find((x) => {
				if (x.product === route.params.product) {
					if (x.version && x.version === route.params.date) {
						return x
					} else if (x.date.unix() === Number(route.params.date as string)) {
						return x
					}
				}
				return undefined
			})
		: undefined,
)

const isFirst = computed(() => changelogEntry.value?.date === getChangelog()[0].date)

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
