<script setup lang="ts">
import { LayoutGridIcon, SquarePlusIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	defineMessages,
	ListToolbar,
	type SortField,
	SortMenu,
	type SortMode,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { ScreenshotGroupBy, ScreenshotSortField } from './types'

const props = defineProps<{
	heading?: string
	screenshotCount: number
	showInstanceGrouping: boolean
}>()

const search = defineModel<string>('search', { required: true })
const sort = defineModel<SortMode<ScreenshotSortField>>('sort', { required: true })
const group = defineModel<ScreenshotGroupBy>('group', { required: true })

const emit = defineEmits<{
	newGroup: []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	searchLabel: { id: 'app.screenshots.search.label', defaultMessage: 'Search screenshots' },
	searchPlaceholder: {
		id: 'app.screenshots.search.placeholder',
		defaultMessage: 'Search {count, plural, one {# screenshot} other {# screenshots}}...',
	},
	newGroup: { id: 'app.screenshots.group.new', defaultMessage: 'New group' },
	groupBy: { id: 'app.screenshots.group-by', defaultMessage: 'Group by' },
	sortDate: { id: 'app.screenshots.sort.date', defaultMessage: 'Date taken' },
	sortName: { id: 'app.screenshots.sort.name', defaultMessage: 'Name' },
	groupCustom: { id: 'app.screenshots.group.custom', defaultMessage: 'Custom group' },
	groupInstance: { id: 'app.screenshots.group.instance', defaultMessage: 'Instance' },
	groupDate: { id: 'app.screenshots.group.date', defaultMessage: 'Date' },
	groupNone: { id: 'app.screenshots.group.none', defaultMessage: 'No grouping' },
})

const groupLabels = {
	custom: messages.groupCustom,
	instance: messages.groupInstance,
	date: messages.groupDate,
	none: messages.groupNone,
} satisfies Record<ScreenshotGroupBy, (typeof messages)[keyof typeof messages]>

const sortFields = computed<SortField<ScreenshotSortField>[]>(() => [
	{ value: 'date', label: formatMessage(messages.sortDate), kind: 'date' },
	{ value: 'name', label: formatMessage(messages.sortName), kind: 'alphabetical' },
])

const groupOptions = computed<ButtonMenuOption[]>(() =>
	(Object.keys(groupLabels) as ScreenshotGroupBy[])
		.filter((option) => option !== 'instance' || props.showInstanceGrouping)
		.map((option) => ({
			id: option,
			label: formatMessage(groupLabels[option]),
			selected: option === group.value,
			action: () => {
				group.value = option
			},
		})),
)

const searchPlaceholder = computed(() =>
	formatMessage(messages.searchPlaceholder, { count: props.screenshotCount }),
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex min-h-[42px] flex-wrap items-center justify-between gap-2">
			<h1 v-if="heading" class="m-0 text-2xl font-bold text-contrast">
				{{ heading }}
			</h1>
			<Button type="outlined" size="lg" class="ml-auto" @click="emit('newGroup')">
				<SquarePlusIcon aria-hidden="true" />
				{{ formatMessage(messages.newGroup) }}
			</Button>
		</div>
		<ListToolbar
			v-model:search="search"
			:search-label="formatMessage(messages.searchLabel)"
			:search-placeholder="searchPlaceholder"
		>
			<template #controls>
				<TeleportOverflowMenu
					type="quiet"
					size="lg"
					:label="formatMessage(messages.groupBy)"
					:options="groupOptions"
					:icon-only="false"
					animate-width
				>
					<LayoutGridIcon aria-hidden="true" />
					{{ formatMessage(groupLabels[group]) }}
				</TeleportOverflowMenu>
				<SortMenu v-model="sort" :fields="sortFields" />
			</template>
		</ListToolbar>
	</div>
</template>
