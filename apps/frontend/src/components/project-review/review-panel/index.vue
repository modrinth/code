<template>
	<Anchor
		v-if="mode === 'anchored'"
		ref="anchor"
		v-bind="$attrs"
		:anchor-id="id"
		:target="target"
		:as="as"
		:disabled="disabled"
		:show-finding-badge="showFindingBadge"
	>
		<slot />
		<Popover
			v-if="active?.id === id"
			:anchor="active"
			:title-id="title && !hideTitle ? titleId : undefined"
			:label="accessibleTitle"
		>
			<template #title>
				<div v-if="title && !hideTitle" class="flex items-center gap-2">
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
				:key="controlsKey"
				:binding="binding"
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
		tabindex="-1"
		v-bind="$attrs"
		:data-review-panel="id"
		:aria-labelledby="title && !hideTitle ? titleId : undefined"
		:aria-label="hideTitle ? accessibleTitle : undefined"
		class="box-border flex w-full flex-col gap-2.5 text-sm text-primary transition-opacity duration-150"
		:class="inlineActive ? 'opacity-100' : 'opacity-50'"
	>
		<Highlight
			:active="inlineActive"
			:enabled="settings.get(moderationSettings.General.ShowInlinePanelHoverHighlight)"
		>
			<div v-if="title && !hideTitle" class="flex items-center gap-2">
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
				<Controls
					:key="controlsKey"
					:binding="binding"
					:target="target"
					@dropdown-open="setInlineDropdownOpen($event, true)"
					@dropdown-close="setInlineDropdownOpen($event, false)"
				/>
			</template>
			<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noReviewActions) }}</p>
		</Highlight>
	</component>
</template>

<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Tooltip, useVIntl } from '@modrinth/ui'
import { useActiveElement, useElementHover } from '@vueuse/core'
import { computed, nextTick, onScopeDispose, shallowRef, useId, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { injectProjectReviewContext, injectReviewSlot } from '../layout/context'
import { projectReviewMessages as messages } from '../messages'
import Anchor from './anchor.vue'
import { flashReviewElement, injectReviewContext, scrollReviewElement } from './context'
import Controls from './controls.vue'
import Highlight from './highlight.vue'
import Popover from './popover.vue'

defineOptions({ inheritAttrs: false })
const props = withDefaults(
	defineProps<{
		mode: 'anchored' | 'inline'
		target: ReviewTarget
		as?: 'section' | 'div' | 'article'
		disabled?: boolean
		showFindingBadge?: boolean
		hideTitle?: boolean
		interactionScope?: HTMLElement | null
	}>(),
	{ showFindingBadge: true },
)

const id = useId()
const anchor = shallowRef<InstanceType<typeof Anchor> | null>(null)
const layout = injectProjectReviewContext(null)
const slot = injectReviewSlot(null)
const inlinePanel = shallowRef<HTMLElement | null>(null)
const inlineDropdowns = shallowRef(new Set<string>())
const panelHovered = useElementHover(inlinePanel)
const scopeHovered = useElementHover(() => props.interactionScope)
const reviewContext = injectReviewContext()
const { active, activePanelId, registerInlinePanel, registerDestination, setDropdownOpen } =
	reviewContext
const focusedElement = useActiveElement()
const inlineActive = computed(() => activePanelId.value === id)
const { formatMessage } = useVIntl()
const settings = useModerationSettings()
const panels = injectReviewPanels()
const titleId = `${id}-title`
const binding = computed(() => panels.resolve(props.target))
watch(
	() => props.mode,
	(mode, _, onCleanup) => {
		if (mode !== 'inline') return
		onCleanup(
			registerInlinePanel({
				id,
				target: () => props.target,
				element: () => inlinePanel.value,
				available: () => !props.disabled && !!binding.value,
				hovered: () => panelHovered.value,
				scopeHovered: () => scopeHovered.value,
				scopeElement: () => props.interactionScope ?? null,
				focused: () =>
					focusedElement.value !== inlinePanel.value &&
					!!inlinePanel.value?.contains(focusedElement.value ?? null) &&
					!!focusedElement.value?.matches(':focus-visible'),
				dropdownOpen: () => inlineDropdowns.value.size > 0,
			}),
		)
	},
	{ immediate: true },
)
const panel = computed(() => binding.value?.panel)
const controlsKey = computed(() =>
	binding.value ? `${binding.value.projectId}:${binding.value.key}` : undefined,
)
onScopeDispose(
	registerDestination(id, {
		key: () => (!props.disabled ? binding.value?.key : undefined),
		reveal: async () => {
			if (slot) layout?.revealSlot(slot.value)
			await nextTick()
			await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
			await nextTick()
			const element = props.mode === 'inline' ? inlinePanel.value : anchor.value?.element
			if (!element) return
			scrollReviewElement(element)
			if (props.mode === 'anchored') {
				anchor.value?.reveal()
			} else {
				reviewContext.revealInlinePanel(id)
				element.focus({ preventScroll: true })
			}
			flashReviewElement(element)
		},
	}),
)
const title = computed(() => panel.value?.title)
const hint = computed(() => panel.value?.hint)
const guidanceUrl = computed(() => panel.value?.guidanceUrl)
const accessibleTitle = computed(() =>
	formatMessage(messages.reviewSection, { section: title.value ?? hint.value ?? '' }),
)

function setInlineDropdownOpen(key: string, open: boolean) {
	const next = new Set(inlineDropdowns.value)
	if (open) next.add(key)
	else next.delete(key)
	inlineDropdowns.value = next
}

watch(
	[controlsKey, () => props.mode, () => props.disabled],
	() => {
		inlineDropdowns.value = new Set()
	},
	{ flush: 'sync' },
)
</script>
