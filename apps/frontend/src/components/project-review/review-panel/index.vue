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
		<Popover
			v-if="active?.id === id"
			:anchor="active"
			:title-id="title ? titleId : undefined"
			:label="accessibleTitle"
		>
			<template #title>
				<div v-if="title" class="flex items-center gap-2">
					<h2 :id="titleId" class="m-0 text-lg font-semibold text-contrast">
						{{ title }}
					</h2>
					<Tooltip
						v-if="hint"
						:text="hint"
						:aria-label="guidanceUrl ? undefined : hint"
						class="flex shrink-0 text-secondary"
					>
						<a
							v-if="guidanceUrl"
							:href="guidanceUrl"
							target="_blank"
							rel="noopener noreferrer"
							:aria-label="formatMessage(messages.openReviewGuidance)"
							class="flex text-secondary hover:text-contrast"
						>
							<InfoIcon class="size-4" aria-hidden="true" />
						</a>
						<InfoIcon v-else class="size-4" aria-hidden="true" />
					</Tooltip>
				</div>
			</template>
			<Controls
				:target="target"
				@dropdown-open="setDropdownOpen(id, true)"
				@dropdown-close="setDropdownOpen(id, false)"
			/>
		</Popover>
	</Anchor>
	<component
		:is="as ?? 'section'"
		v-else
		ref="inlinePanel"
		v-bind="$attrs"
		:data-review-panel="id"
		:aria-labelledby="title ? titleId : undefined"
		class="box-border flex w-full flex-col gap-2.5 overflow-y-auto text-sm text-primary opacity-60 transition-opacity duration-150 hover:opacity-100"
	>
		<div v-if="title" class="flex items-center gap-2">
			<h2 :id="titleId" class="m-0 text-sm font-semibold text-contrast">
				{{ title }}
			</h2>
			<Tooltip
				v-if="hint"
				:text="hint"
				:aria-label="guidanceUrl ? undefined : hint"
				class="flex shrink-0 text-secondary"
			>
				<a
					v-if="guidanceUrl"
					:href="guidanceUrl"
					target="_blank"
					rel="noopener noreferrer"
					:aria-label="formatMessage(messages.openReviewGuidance)"
					class="flex text-secondary hover:text-contrast"
				>
					<InfoIcon class="size-4" aria-hidden="true" />
				</a>
				<InfoIcon v-else class="size-4" aria-hidden="true" />
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
import { computed, shallowRef, useId } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'
import Anchor from './anchor.vue'
import { injectReviewContext } from './context'
import Controls from './controls.vue'
import Popover from './popover.vue'
import { useActionKeybinds } from './use-action-keybinds'

defineOptions({ inheritAttrs: false })
const props = defineProps<{
	mode: 'anchored' | 'inline'
	target: ReviewTarget
	as?: 'section' | 'div' | 'article'
	disabled?: boolean
	triggerPlacement?: 'inset' | 'header' | 'overlay' | 'above'
}>()

const id = useId()
const inlinePanel = shallowRef<HTMLElement | null>(null)
useActionKeybinds(inlinePanel)
const { formatMessage } = useVIntl()
const { active, setDropdownOpen } = injectReviewContext()
const panels = injectReviewPanels()
const titleId = `${id}-title`
const panel = computed(() => panels.resolve(props.target)?.panel)
const title = computed(() => panel.value?.title)
const hint = computed(() => panel.value?.hint)
const guidanceUrl = computed(() => panel.value?.guidanceUrl)
const accessibleTitle = computed(() =>
	formatMessage(messages.reviewSection, { section: title.value ?? hint.value ?? '' }),
)
</script>
