<template>
	<div
		v-if="level === 'survey'"
		class="flex items-center gap-2 border-2 border-solid border-brand-purple bg-bg-purple p-4 rounded-2xl"
	>
		<span class="text-contrast font-bold">Survey ID:</span> <CopyCode :text="message" />
	</div>
	<Admonition v-else :type="NOTICE_TYPE[level]">
		<template #header>
			<template v-if="!hideDefaultTitle">
				{{ formatMessage(heading) }}
			</template>
			<template v-if="title">
				<template v-if="hideDefaultTitle">
					{{ title.substring(1) }}
				</template>
				<template v-else> - {{ title }}</template>
			</template>
		</template>
		<template #actions>
			<ButtonStyled v-if="dismissable" :color="NOTICE_TYPE_BTN[level]">
				<button
					v-tooltip="formatMessage(messages.dismiss)"
					@click="() => (preview ? {} : emit('dismiss'))"
				>
					<XIcon /> Dismiss
				</button>
			</ButtonStyled>
		</template>
		<div v-if="message" class="markdown-body" v-html="renderString(message)" />
	</Admonition>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { renderString } from '@modrinth/utils'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

import Admonition from './Admonition.vue'
import ButtonStyled from './ButtonStyled.vue'
import CopyCode from './CopyCode.vue'

const { formatMessage } = useVIntl()
const emit = defineEmits<{
	(e: 'dismiss'): void
}>()

const props = withDefaults(
	defineProps<{
		level: string
		message: string
		dismissable: boolean
		preview?: boolean
		title?: string
	}>(),
	{
		preview: false,
		title: undefined,
	},
)

const hideDefaultTitle = computed(
	() => props.title && props.title.length > 1 && props.title.startsWith('\\'),
)

const messages = defineMessages({
	info: {
		id: 'servers.notice.heading.info',
		defaultMessage: 'Info',
	},
	attention: {
		id: 'servers.notice.heading.attention',
		defaultMessage: 'Attention',
	},
	dismiss: {
		id: 'servers.notice.dismiss',
		defaultMessage: 'Dismiss',
	},
})

const NOTICE_HEADINGS: Record<string, MessageDescriptor> = {
	info: messages.info,
	warn: messages.attention,
	critical: messages.attention,
}

const NOTICE_TYPE: Record<string, 'info' | 'warning' | 'critical'> = {
	info: 'info',
	warn: 'warning',
	critical: 'critical',
}

const NOTICE_TYPE_BTN: Record<string, 'blue' | 'orange' | 'red'> = {
	info: 'blue',
	warn: 'orange',
	critical: 'red',
}

const heading = computed(() => NOTICE_HEADINGS[props.level] ?? messages.info)
</script>
<style scoped lang="scss">
.markdown-body > *:first-child {
	margin-top: 0;
}
</style>
