<template>
	<component
		:is="as"
		ref="element"
		class="review-anchor relative"
		:class="{
			'!pr-10': available && triggerPlacement === 'inset',
			'review-anchor-header': available && triggerPlacement === 'header',
		}"
		:data-review-anchor="id"
		:data-review-target="target.kind"
		:data-review-key="'key' in target ? target.key : undefined"
		@pointerover.stop="onPointerOver"
		@pointerleave="leave(id)"
	>
		<slot />
		<button
			v-if="available"
			ref="trigger"
			type="button"
			class="review-trigger absolute right-0.5 top-0.5 flex cursor-pointer items-center gap-1 rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-1 text-xs text-secondary hover:text-contrast focus-visible:opacity-100"
			:class="{ 'review-trigger-active': active?.id === id }"
			:aria-label="formatMessage(messages.reviewSection, { section: label })"
			:aria-expanded="active?.id === id"
			:aria-controls="active?.id === id ? panelId : undefined"
			aria-haspopup="dialog"
			@click.stop="show(true)"
		>
			<ListBulletedIcon class="size-3.5" aria-hidden="true" />
		</button>
	</component>
</template>

<script setup lang="ts">
import { ListBulletedIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'

import { projectReviewMessages as messages } from '../messages'
import { injectReviewContext } from './context'

const props = withDefaults(
	defineProps<{
		anchorId: string
		target: ReviewTarget
		label: string
		as?: 'section' | 'div' | 'article'
		disabled?: boolean
		triggerPlacement?: 'inset' | 'header' | 'overlay'
	}>(),
	{ as: 'div', triggerPlacement: 'inset' },
)
const { formatMessage } = useVIntl()
const { active, panelId, isAvailable, open, release, leave, cancelClose } = injectReviewContext()
const id = props.anchorId
const element = ref<HTMLElement>()
const trigger = ref<HTMLButtonElement>()
const available = computed(() => !props.disabled && isAvailable(props.target))

function show(explicit = false) {
	if (!element.value || !available.value) return
	open(
		{
			id,
			target: props.target,
			label: props.label,
			element: element.value,
			trigger: trigger.value ?? null,
			available: () => available.value,
		},
		explicit,
	)
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
watch(
	[
		() => props.target.kind,
		() => ('key' in props.target ? props.target.key : undefined),
		() => props.label,
	],
	() => release(id),
)
onBeforeUnmount(() => release(id))
</script>

<style scoped>
.review-anchor-header :slotted([data-review-header]) {
	padding-inline-end: 3rem;
}

.review-trigger {
	opacity: 0;
	transition: opacity 120ms ease;
}
.review-anchor:hover > .review-trigger,
.review-anchor:focus-within > .review-trigger,
.review-trigger-active {
	opacity: 1;
}
@media (hover: none) {
	.review-trigger {
		opacity: 1;
	}
}
</style>
