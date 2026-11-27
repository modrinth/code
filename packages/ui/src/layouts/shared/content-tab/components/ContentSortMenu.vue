<script setup lang="ts">
import { computed } from 'vue'

import SortMenu, { type SortField, type SortKind } from '#ui/components/base/SortMenu.vue'
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'

import type { ContentSortField, ContentSortMode } from '../composables/content-sort'

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
})

const fieldLabels: Record<ContentSortField, MessageDescriptor> = {
	name: messages.fieldName,
	author: messages.fieldAuthor,
	updated: messages.fieldUpdated,
	added: messages.fieldAdded,
}

const fieldKinds: Record<ContentSortField, SortKind> = {
	name: 'alphabetical',
	author: 'alphabetical',
	updated: 'date',
	added: 'date',
}

const sortFields = computed<SortField<ContentSortField>[]>(() =>
	props.fields.map((field) => ({
		value: field,
		label: formatMessage(fieldLabels[field]),
		kind: fieldKinds[field],
	})),
)
</script>

<template>
	<SortMenu v-model="sortMode" :fields="sortFields" />
</template>
