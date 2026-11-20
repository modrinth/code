<script setup lang="ts">
import {
	ArrowDownAZIcon,
	ArrowUpZAIcon,
	ClockArrowDownIcon,
	ClockArrowUpIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import SegmentedControl from '#ui/components/base/SegmentedControl.vue'
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'

import {
	contentSortDefaultDirections,
	type ContentSortDirection,
	type ContentSortField,
	type ContentSortMode,
} from '../composables/content-sort'

const props = defineProps<{ fields: ContentSortField[] }>()

const sortMode = defineModel<ContentSortMode>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	fieldName: {
		id: 'content.page-layout.sort.field.name',
		defaultMessage: 'Name',
	},
	fieldAuthor: {
		id: 'content.page-layout.sort.field.author',
		defaultMessage: 'Author',
	},
	fieldUpdated: {
		id: 'content.page-layout.sort.field.updated',
		defaultMessage: 'Last updated',
	},
	fieldAdded: {
		id: 'content.page-layout.sort.field.added',
		defaultMessage: 'Date added',
	},
	directionAlphabeticalAscending: {
		id: 'content.page-layout.sort.direction.alphabetical-ascending',
		defaultMessage: 'A-Z',
	},
	directionAlphabeticalDescending: {
		id: 'content.page-layout.sort.direction.alphabetical-descending',
		defaultMessage: 'Z-A',
	},
	directionDateDescending: {
		id: 'content.page-layout.sort.direction.date-descending',
		defaultMessage: 'Newest first',
	},
	directionDateAscending: {
		id: 'content.page-layout.sort.direction.date-ascending',
		defaultMessage: 'Oldest first',
	},
	directionLabel: {
		id: 'content.page-layout.sort.direction.label',
		defaultMessage: 'Sort direction',
	},
	sortByLabel: {
		id: 'content.page-layout.sort.current-mode',
		defaultMessage:
			'Sort: {field, select, author {Author} updated {Last updated} added {Date added} other {Name}}, {direction, select, za {Z-A} newest {Newest first} oldest {Oldest first} other {A-Z}}',
	},
})

const fieldLabels: Record<ContentSortField, MessageDescriptor> = {
	name: messages.fieldName,
	author: messages.fieldAuthor,
	updated: messages.fieldUpdated,
	added: messages.fieldAdded,
}

const isDateField = computed(
	() => sortMode.value.field === 'updated' || sortMode.value.field === 'added',
)

const directions = computed<ContentSortDirection[]>(() =>
	isDateField.value ? ['desc', 'asc'] : ['asc', 'desc'],
)

const direction = computed<ContentSortDirection>({
	get: () => sortMode.value.direction,
	set: (value) => (sortMode.value = { ...sortMode.value, direction: value }),
})

function formatDirection(value: ContentSortDirection) {
	if (isDateField.value) {
		return formatMessage(
			value === 'desc' ? messages.directionDateDescending : messages.directionDateAscending,
		)
	}
	return formatMessage(
		value === 'asc'
			? messages.directionAlphabeticalAscending
			: messages.directionAlphabeticalDescending,
	)
}

const triggerIcon = computed(() => {
	const ascending = sortMode.value.direction === 'asc'
	if (isDateField.value) return ascending ? ClockArrowUpIcon : ClockArrowDownIcon
	return ascending ? ArrowDownAZIcon : ArrowUpZAIcon
})

const fieldLabel = computed(() => formatMessage(fieldLabels[sortMode.value.field]))

const directionKey = computed(() => {
	const descending = sortMode.value.direction === 'desc'
	if (isDateField.value) return descending ? 'newest' : 'oldest'
	return descending ? 'za' : 'az'
})

const label = computed(() =>
	formatMessage(messages.sortByLabel, {
		field: sortMode.value.field,
		direction: directionKey.value,
	}),
)

const sortOptions = computed<ButtonMenuOption[]>(() =>
	props.fields.map((field) => ({
		id: field,
		label: formatMessage(fieldLabels[field]),
		selected: field === sortMode.value.field,
		remainOpen: true,
		action: () => {
			if (field === sortMode.value.field) return
			sortMode.value = { field, direction: contentSortDefaultDirections[field] }
		},
	})),
)
</script>

<template>
	<TeleportOverflowMenu
		type="quiet"
		size="lg"
		:label="label"
		:options="sortOptions"
		:icon-only="false"
		animate-width
		animate-panel-width
	>
		<component :is="triggerIcon" aria-hidden="true" />
		{{ fieldLabel }}
		<template #footer>
			<div role="separator" class="mb-2 mt-1 h-px bg-surface-5" />
			<SegmentedControl
				v-model="direction"
				:items="directions"
				:neutral-items="directions"
				:format-label="formatDirection"
				:aria-label="formatMessage(messages.directionLabel)"
				full-width
			/>
		</template>
	</TeleportOverflowMenu>
</template>
