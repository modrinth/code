<template>
	<div v-if="count > 1" class="flex items-center gap-1">
		<template v-if="page > 1">
			<ButtonLink
				v-if="linkFunction"
				v-tooltip="formatMessage(messages.previousPage)"
				:aria-label="formatMessage(messages.previousPage)"
				:href="linkFunction(page - 1)"
				type="quiet"
				class="!w-9 !px-0 !rounded-full"
				@click.prevent="switchPage(page - 1)"
			>
				<ChevronLeftIcon aria-hidden="true" />
			</ButtonLink>
			<IconButton
				v-else
				v-tooltip="formatMessage(messages.previousPage)"
				:label="formatMessage(messages.previousPage)"
				type="quiet"
				@click="switchPage(page - 1)"
			>
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
			<form v-if="item === '-'" class="grid place-content-center" @submit.prevent="goToPage">
				<StyledInput
					v-if="showPageInput === index"
					:ref="focusInput"
					v-model="pageInput"
					v-tooltip="formatMessage(messages.goToPage)"
					type="number"
					:min="1"
					:max="props.count"
					placeholder="..."
					clamp
					class="w-14"
					:aria-label="formatMessage(messages.goToPage)"
					@focusout="showPageInput = undefined"
					@keydown.escape="showPageInput = undefined"
				/>

				<div v-else class="rotate-90">
					<button
						v-tooltip="formatMessage(messages.goToPage)"
						type="button"
						:aria-label="formatMessage(messages.goToPage)"
						class="grid place-content-center"
						@click="openPageInput(index)"
					>
						<EllipsisVerticalIcon aria-hidden="true" />
					</button>
				</div>
			</form>
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
				v-tooltip="formatMessage(messages.nextPage)"
				:aria-label="formatMessage(messages.nextPage)"
				:href="linkFunction(page + 1)"
				type="quiet"
				class="!w-9 !px-0 !rounded-full"
				@click.prevent="switchPage(page + 1)"
			>
				<ChevronRightIcon aria-hidden="true" />
			</ButtonLink>
			<IconButton
				v-else
				v-tooltip="formatMessage(messages.nextPage)"
				:label="formatMessage(messages.nextPage)"
				type="quiet"
				@click="switchPage(page + 1)"
			>
				<ChevronRightIcon aria-hidden="true" />
			</IconButton>
		</template>
	</div>
</template>
<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, EllipsisVerticalIcon } from '@modrinth/assets'
import { type ComponentPublicInstance, computed, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n.ts'

import { Button, ButtonLink, IconButton } from './buttons'
import StyledInput from './StyledInput.vue'

const emit = defineEmits<{
	'switch-page': [page: number]
}>()

const { formatMessage } = useVIntl()

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
const showPageInput = ref<number | undefined>(undefined)
const pageInput = ref<number | undefined>(undefined)

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

function focusInput(element: Element | ComponentPublicInstance | null) {
	if (element && 'focus' in element) {
		const styledInput = element as InstanceType<typeof StyledInput>
		styledInput.focus()
	}
}

function openPageInput(index: number) {
	pageInput.value = undefined
	showPageInput.value = index
}

function goToPage() {
	if (pageInput.value !== undefined && pageInput.value >= 1 && pageInput.value <= props.count) {
		switchPage(pageInput.value)
	}

	showPageInput.value = undefined
	pageInput.value = undefined
}

const messages = defineMessages({
	goToPage: {
		id: 'ui.pagination.go-to-page',
		defaultMessage: 'Go to page',
	},
	previousPage: {
		id: 'ui.pagination.previous-page',
		defaultMessage: 'Previous page',
	},
	nextPage: {
		id: 'ui.pagination.next-page',
		defaultMessage: 'Next page',
	},
})
</script>
