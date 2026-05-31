<script setup lang="ts">
import { Tooltip } from 'floating-vue'
import { type Component, useId } from 'vue'

import { type MessageDescriptor, useVIntl } from '#ui/composables/i18n.ts'
import { ExternalIcon } from '@modrinth/assets'
import AutoLink from '../base/AutoLink.vue'

const { formatMessage } = useVIntl()

defineProps<{
	icon: Component
	name: MessageDescriptor
	about: MessageDescriptor[]
	values?: Record<string, unknown>
	link?: {
		href: string
		message: MessageDescriptor
	}
}>()

const baseId = useId()
</script>
<template>
	<Tooltip theme="tooltip" :triggers="['hover', 'focus']" :aria-id="`${baseId}-${name.id}`">
		<AutoLink
			:to="link?.href"
			class="rounded-2xl flex"
			:class="{
				'hover:bg-surface-4 focus:bg-surface-4': !!link,
			}"
			target="_blank"
			tabindex="0"
		>
			<component :is="icon" class="size-full p-0.5" />
		</AutoLink>
		<template #popper>
			<div class="flex flex-col max-w-[22rem] leading-tight gap-0.5">
				<span class="text-contrast mb-1">{{ formatMessage(name, values) }}</span>
				<span v-for="message of about" :key="message.id" class="text-primary">
					{{ formatMessage(message, values) }}
				</span>
				<span v-if="link" class="text-secondary text-xs opacity-80">
					{{ formatMessage(link.message, values) }} <ExternalIcon />
				</span>
			</div>
		</template>
	</Tooltip>
</template>
