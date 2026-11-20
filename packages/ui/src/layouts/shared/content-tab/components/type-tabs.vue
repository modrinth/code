<script setup lang="ts">
import { computed } from 'vue'

import ToolbarTabs, { type ToolbarTab } from '#ui/components/base/ToolbarTabs.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonProjectTypeCategoryMessages } from '#ui/utils/common-messages'

import type { ContentType } from '../types'

const props = defineProps<{
	types: ContentType[]
}>()

const active = defineModel<ContentType>({ required: true })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	label: {
		id: 'content.page-layout.type-tabs.label',
		defaultMessage: 'Content types',
	},
})

const tabs = computed<ToolbarTab<ContentType>[]>(() =>
	props.types.map((type) => ({
		value: type,
		label: formatMessage(commonProjectTypeCategoryMessages[type]),
	})),
)
</script>

<template>
	<ToolbarTabs v-model="active" :tabs="tabs" :label="formatMessage(messages.label)" />
</template>
