<template>
	<div v-if="count > 1" class="flex items-center gap-1">
		<template v-if="page > 1">
			<ButtonLink
				v-if="linkFunction"
				aria-label="Previous Page"
				:href="linkFunction(page - 1)"
				type="quiet"
				class="!w-9 !px-0 !rounded-full"
				@click.prevent="switchPage(page - 1)"
			>
				<ChevronLeftIcon aria-hidden="true" />
			</ButtonLink>
			<IconButton v-else label="Previous Page" type="quiet" @click="switchPage(page - 1)">
				<ChevronLeftIcon aria-hidden="true" />
			</IconButton>
		</template>
		<div
			v-for="(item, index) in pages"
			:key="'page-' + item + '-' + index"
			:class="{
				'page-number': page !== item,
				shrink: item !== '-' && item > 99,
			}"
			class="page-number-container"
		>
			<div v-if="item === '-'" class="rotate-90 grid place-content-center">
				<EllipsisVerticalIcon />
			</div>
			<template v-else>
				<ButtonLink
					v-if="linkFunction"
					:href="linkFunction(item)"
					type="quiet"
					:color="page === item ? 'brand' : undefined"
					:interaction="page === item ? 'filled' : undefined"
					:aria-current="page === item ? 'page' : undefined"
					:class="['!min-w-9 !rounded-full', page === item ? '!bg-brand-highlight' : '']"
					@click.prevent="page !== item ? switchPage(item) : null"
				>
					{{ item }}
				</ButtonLink>
				<Button
					v-else
					type="quiet"
					:color="page === item ? 'brand' : undefined"
					:interaction="page === item ? 'filled' : undefined"
					:aria-current="page === item ? 'page' : undefined"
					:class="['!min-w-9 !rounded-full', page === item ? '!bg-brand-highlight' : '']"
					@click="page !== item ? switchPage(item) : null"
				>
					{{ item }}
				</Button>
			</template>
		</div>

		<template v-if="page !== pages[pages.length - 1]">
			<ButtonLink
				v-if="linkFunction"
				aria-label="Next Page"
				:href="linkFunction(page + 1)"
				type="quiet"
				class="!w-9 !px-0 !rounded-full"
				@click.prevent="switchPage(page + 1)"
			>
				<ChevronRightIcon aria-hidden="true" />
			</ButtonLink>
			<IconButton v-else label="Next Page" type="quiet" @click="switchPage(page + 1)">
				<ChevronRightIcon aria-hidden="true" />
			</IconButton>
		</template>
	</div>
</template>
<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, EllipsisVerticalIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { Button, ButtonLink, IconButton } from './buttons'

const emit = defineEmits<{
	'switch-page': [page: number]
}>()

const props = withDefaults(
	defineProps<{
		page: number
		count: number
		linkFunction?: (page: number) => string | undefined
	}>(),
	{
		page: 1,
		count: 1,
	},
)

const pages = computed(() => {
	const pages: ('-' | number)[] = []

	const first = 1
	const last = props.count
	const current = props.page
	const prev = current - 1
	const next = current + 1
	const gap = '-'

	if (prev > first) {
		pages.push(first)
	}
	if (prev > first + 1) {
		pages.push(gap)
	}
	if (prev >= first) {
		pages.push(prev)
	}
	pages.push(current)
	if (next <= last) {
		pages.push(next)
	}
	if (next < last - 1) {
		pages.push(gap)
	}
	if (next < last) {
		pages.push(last)
	}

	return pages
})

function switchPage(newPage: number) {
	emit('switch-page', Math.min(Math.max(newPage, 1), props.count))
}
</script>
