<template>
	<component
		:is="as"
		ref="element"
		class="relative [&:hover>[data-review-highlight]::before]:opacity-100"
		:data-review-anchor="id"
		:data-review-target="target.kind"
		:data-review-key="'key' in target ? target.key : undefined"
		@pointerover.stop="onPointerOver"
		@pointerleave="leave(id)"
		@focusin="active?.id === id && cancelClose()"
		@focusout="leave(id)"
	>
		<Highlight
			:active="active?.id === id"
			:enabled="settings.get(moderationSettings.General.ShowFloatingPanelFieldHoverHighlight)"
			layer="behind"
		>
			<slot />
			<span
				v-if="selectedFindingCount > 0"
				class="pointer-events-none absolute right-0 top-0 flex h-[18px] min-w-[18px] items-center justify-center rounded-full border border-solid border-highlight-orange bg-highlight-orange px-1 text-xs font-medium leading-none text-orange"
			>
				<span aria-hidden="true">{{ selectedFindingCount }}</span>
				<span class="sr-only">{{
					formatMessage(messages.selectedFindings, { count: selectedFindingCount })
				}}</span>
			</span>
			<button
				v-if="available"
				ref="trigger"
				type="button"
				class="pointer-events-none absolute top-0 z-20 flex size-7 cursor-pointer items-center justify-center rounded-lg border border-solid border-surface-5 bg-surface-3 text-contrast opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow [@media(hover:none)]:pointer-events-auto [@media(hover:none)]:opacity-100"
				:class="selectedFindingCount > 0 ? 'right-6' : 'right-0'"
				:aria-label="label"
				:aria-expanded="active?.id === id"
				:aria-controls="active?.id === id ? panelId : undefined"
				aria-haspopup="dialog"
				@click.stop="openAndFocus"
			>
				<ListBulletedIcon class="size-4" aria-hidden="true" />
			</button>
		</Highlight>
	</component>
</template>

<script setup lang="ts">
import { ListBulletedIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { useVIntl } from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'
import { injectReviewContext } from './context'
import Highlight from './highlight.vue'

const props = withDefaults(
	defineProps<{
		anchorId: string
		target: ReviewTarget
		label: string
		as?: 'section' | 'div' | 'article'
		disabled?: boolean
	}>(),
	{ as: 'div' },
)
const { active, panel, panelId, isAvailable, open, release, leave, cancelClose } =
	injectReviewContext()
const id = props.anchorId
const element = ref<HTMLElement>()
const trigger = ref<HTMLButtonElement>()
const available = computed(() => !props.disabled && isAvailable(props.target))
const { formatMessage } = useVIntl()
const settings = useModerationSettings()
const panels = injectReviewPanels()
const selectedFindingCount = computed(() => {
	const binding = panels.resolve(props.target)
	if (!binding) return 0
	return panels.selectedFindingCount(binding)
})

function show(explicit = false) {
	if (!element.value || !available.value) return
	open(
		{
			id,
			target: props.target,
			element: element.value,
			trigger: trigger.value ?? null,
			available: () => available.value,
		},
		explicit,
	)
}

async function openAndFocus() {
	show(true)
	await nextTick()
	if (active.value?.id === id) panel.value?.focus()
}

function onPointerOver(event: PointerEvent) {
	if (event.pointerType === 'touch') return
	if (active.value?.id === id) {
		cancelClose()
		return
	}
	show()
}

watch(available, (value) => {
	if (!value) release(id)
})
watch([() => props.target.kind, () => ('key' in props.target ? props.target.key : undefined)], () =>
	release(id),
)
onBeforeUnmount(() => release(id))
</script>
