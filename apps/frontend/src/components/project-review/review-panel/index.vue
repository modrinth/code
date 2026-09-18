<template>
	<Anchor
		v-if="mode === 'anchored'"
		v-bind="$attrs"
		:anchor-id="id"
		:target="target"
		:label="label"
		:as="as"
		:disabled="disabled"
		:trigger-placement="triggerPlacement"
	>
		<slot />
		<Popover v-if="active?.id === id" :anchor="active">
			<Controls :target="target" />
		</Popover>
	</Anchor>
	<component
		:is="as ?? 'section'"
		v-else
		v-bind="$attrs"
		:aria-label="formatMessage(messages.reviewSection, { section: label })"
		class="flex w-full flex-col gap-2.5 overflow-y-auto text-sm text-primary"
	>
		<Controls v-if="!disabled" :target="target" />
		<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noReviewActions) }}</p>
	</component>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { useId } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'

import { projectReviewMessages as messages } from '../messages'
import Anchor from './anchor.vue'
import { injectReviewContext } from './context'
import Controls from './controls.vue'
import Popover from './popover.vue'

defineOptions({ inheritAttrs: false })
defineProps<{
	mode: 'anchored' | 'inline'
	target: ReviewTarget
	label: string
	as?: 'section' | 'div' | 'article'
	disabled?: boolean
	triggerPlacement?: 'inset' | 'header' | 'overlay'
}>()

const id = useId()
const { formatMessage } = useVIntl()
const { active } = injectReviewContext()
</script>
