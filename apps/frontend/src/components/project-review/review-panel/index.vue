<template>
	<Anchor
		v-if="mode === 'anchored'"
		v-bind="$attrs"
		:anchor-id="id"
		:target="target"
		:trigger-label="accessibleTitle"
		:as="as"
		:disabled="disabled"
		:trigger-placement="triggerPlacement"
	>
		<slot />
		<Popover v-if="active?.id === id" :anchor="active" :title-id="titleId">
			<template #title>
				<div class="flex items-center gap-2">
					<h2 :id="titleId" class="m-0 text-lg font-semibold text-contrast">{{ title }}</h2>
					<Tooltip v-if="hint" :text="hint" :aria-label="hint" class="flex shrink-0 text-secondary">
						<InfoIcon class="size-4" aria-hidden="true" />
					</Tooltip>
				</div>
			</template>
			<Controls :target="target" />
		</Popover>
	</Anchor>
	<component
		:is="as ?? 'section'"
		v-else
		v-bind="$attrs"
		:aria-labelledby="title ? titleId : undefined"
		class="box-border flex w-full flex-col gap-2.5 overflow-y-auto p-1 text-sm text-primary"
	>
		<div v-if="title" class="flex items-center gap-2">
			<h2 :id="titleId" class="m-0 text-sm font-semibold text-contrast">{{ title }}</h2>
			<Tooltip v-if="hint" :text="hint" :aria-label="hint" class="flex shrink-0 text-secondary">
				<InfoIcon class="size-4" aria-hidden="true" />
			</Tooltip>
		</div>
		<template v-if="!disabled">
			<Controls :target="target" />
		</template>
		<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noReviewActions) }}</p>
	</component>
</template>

<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { Tooltip, useVIntl } from '@modrinth/ui'
import { computed, useId } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'
import Anchor from './anchor.vue'
import { injectReviewContext } from './context'
import Controls from './controls.vue'
import Popover from './popover.vue'

defineOptions({ inheritAttrs: false })
const props = defineProps<{
	mode: 'anchored' | 'inline'
	target: ReviewTarget
	as?: 'section' | 'div' | 'article'
	disabled?: boolean
	triggerPlacement?: 'inset' | 'header' | 'overlay'
}>()

const id = useId()
const { formatMessage } = useVIntl()
const { active } = injectReviewContext()
const panels = injectReviewPanels()
const titleId = `${id}-title`
const panel = computed(() => panels.resolve(props.target)?.panel)
const title = computed(() => panel.value?.title)
const accessibleTitle = computed(() =>
	formatMessage(messages.reviewSection, { section: title.value ?? '' }),
)
const hint = computed(() => panel.value?.hint)
</script>
