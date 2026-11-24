<script setup lang="ts" generic="T extends string">
import {
	ArrowDownAZIcon,
	ArrowDownWideNarrowIcon,
	ArrowUpNarrowWideIcon,
	ArrowUpZAIcon,
	ClockArrowDownIcon,
	ClockArrowUpIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import ToolbarTabs, { type ToolbarTab } from '#ui/components/base/ToolbarTabs.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

export type SortDirection = 'asc' | 'desc'

export type SortKind = 'alphabetical' | 'date' | 'numeric'

export interface SortField<F extends string = string> {
	value: F
	label: string
	kind: SortKind
}

export interface SortMode<F extends string = string> {
	field: F
	direction: SortDirection
}

const props = defineProps<{ fields: SortField<T>[] }>()

const sortMode = defineModel<SortMode<T>>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	alphabeticalAscending: {
		id: 'content.page-layout.sort.direction.alphabetical-ascending',
		defaultMessage: 'A-Z',
	},
	alphabeticalDescending: {
		id: 'content.page-layout.sort.direction.alphabetical-descending',
		defaultMessage: 'Z-A',
	},
	dateDescending: {
		id: 'content.page-layout.sort.direction.date-descending',
		defaultMessage: 'Newest first',
	},
	dateAscending: {
		id: 'content.page-layout.sort.direction.date-ascending',
		defaultMessage: 'Oldest first',
	},
	numericDescending: {
		id: 'sort-menu.direction.numeric-descending',
		defaultMessage: 'Highest first',
	},
	numericAscending: {
		id: 'sort-menu.direction.numeric-ascending',
		defaultMessage: 'Lowest first',
	},
	directionLabel: {
		id: 'content.page-layout.sort.direction.label',
		defaultMessage: 'Sort direction',
	},
	currentMode: {
		id: 'sort-menu.current-mode',
		defaultMessage: 'Sort: {field}, {direction}',
	},
})

const directionMessages = {
	alphabetical: { asc: messages.alphabeticalAscending, desc: messages.alphabeticalDescending },
	date: { asc: messages.dateAscending, desc: messages.dateDescending },
	numeric: { asc: messages.numericAscending, desc: messages.numericDescending },
}

const directionIcons = {
	alphabetical: { asc: ArrowDownAZIcon, desc: ArrowUpZAIcon },
	date: { asc: ClockArrowUpIcon, desc: ClockArrowDownIcon },
	numeric: { asc: ArrowUpNarrowWideIcon, desc: ArrowDownWideNarrowIcon },
}

function defaultDirection(kind: SortKind): SortDirection {
	return kind === 'alphabetical' ? 'asc' : 'desc'
}

const activeField = computed(
	() => props.fields.find((field) => field.value === sortMode.value.field) ?? props.fields[0],
)

const activeKind = computed(() => activeField.value?.kind ?? 'alphabetical')

function directionTabsFor(kind: SortKind): ToolbarTab<SortDirection>[] {
	const first = defaultDirection(kind)
	const second: SortDirection = first === 'asc' ? 'desc' : 'asc'
	return [first, second].map((value) => ({
		value,
		label: formatMessage(directionMessages[kind][value]),
	}))
}

const directionTabs = computed(() => directionTabsFor(activeKind.value))

const sizerKinds = computed(() => [...new Set(props.fields.map((field) => field.kind))])

const direction = computed<SortDirection>({
	get: () => sortMode.value.direction,
	set: (value) => (sortMode.value = { ...sortMode.value, direction: value }),
})

const triggerIcon = computed(() => directionIcons[activeKind.value][sortMode.value.direction])

const label = computed(() =>
	formatMessage(messages.currentMode, {
		field: activeField.value?.label ?? '',
		direction: formatMessage(directionMessages[activeKind.value][sortMode.value.direction]),
	}),
)

const sortOptions = computed<ButtonMenuOption[]>(() =>
	props.fields.map((field) => ({
		id: field.value,
		label: field.label,
		selected: field.value === sortMode.value.field,
		remainOpen: true,
		action: () => {
			if (field.value === sortMode.value.field) return
			sortMode.value = { field: field.value, direction: defaultDirection(field.kind) }
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
	>
		<component :is="triggerIcon" aria-hidden="true" />
		{{ activeField?.label }}
		<template #footer>
			<div role="separator" class="mb-2 mt-1 h-px bg-surface-5" />
			<div class="grid">
				<ToolbarTabs
					v-model="direction"
					:tabs="directionTabs"
					:label="formatMessage(messages.directionLabel)"
					full-width
					class="[grid-area:1/1]"
				/>
				<ToolbarTabs
					v-for="kind in sizerKinds"
					:key="kind"
					:model-value="defaultDirection(kind)"
					:tabs="directionTabsFor(kind)"
					:label="formatMessage(messages.directionLabel)"
					full-width
					aria-hidden="true"
					inert
					class="invisible [grid-area:1/1]"
				/>
			</div>
		</template>
	</TeleportOverflowMenu>
</template>
