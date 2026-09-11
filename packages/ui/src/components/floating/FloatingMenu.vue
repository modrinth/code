<script setup lang="ts">
import {
	arrow,
	autoUpdate,
	flip,
	offset,
	type Placement,
	shift,
	useFloating,
} from '@floating-ui/vue'
import { computed, nextTick, onBeforeUnmount, ref, useTemplateRef } from 'vue'

import { registerFloatingMenu } from '../../providers/floating-menu'
import { dismissTooltip } from '../../providers/tooltip'

const SIDES = {
	top: { origin: 'center bottom', arrow: 'bottom', rotate: 45 },
	right: { origin: 'left center', arrow: 'left', rotate: 135 },
	bottom: { origin: 'center top', arrow: 'top', rotate: 225 },
	left: { origin: 'right center', arrow: 'right', rotate: 315 },
} as const

const props = withDefaults(
	defineProps<{
		trigger?: 'click' | 'hover'
		placement?: Placement
		theme?: string
		disabled?: boolean
		bare?: boolean
		panelClass?: string
	}>(),
	{
		trigger: 'click',
		placement: 'bottom',
		theme: 'dropdown',
		disabled: false,
		bare: false,
	},
)

const emit = defineEmits<{
	open: []
	close: []
}>()

const hover = computed(() => props.trigger === 'hover')
const isOpen = ref(false)
const triggerEl = useTemplateRef<HTMLElement>('triggerEl')
const floating = useTemplateRef<HTMLElement>('floating')
const arrowEl = useTemplateRef<HTMLElement>('arrowEl')
const expandOrigin = ref(SIDES.bottom.origin)
const { floatingStyles, middlewareData, placement, x, y, isPositioned } = useFloating(
	triggerEl,
	floating,
	{
		placement: () => props.placement,
		strategy: 'fixed',
		transform: false,
		whileElementsMounted(reference, floatingEl, update) {
			return autoUpdate(reference, floatingEl, () => {
				update()
				updateOrigin()
			})
		},
		open: isOpen,
		middleware: [offset(8), flip(), shift({ padding: 8 }), arrow({ element: arrowEl })],
	},
)

const side = computed(() => (placement.value.split('-')[0] ?? 'bottom') as keyof typeof SIDES)
const expandStyle = computed(() => {
	const horizontal = side.value === 'left' || side.value === 'right'
	return {
		transformOrigin: expandOrigin.value,
		'--floating-expand-origin': expandOrigin.value,
		...(horizontal ? { '--floating-expand-x': '0.3', '--floating-expand-y': '0.8' } : {}),
	}
})
const arrowStyles = computed(() => {
	const { arrow: edge, rotate } = SIDES[side.value]
	const { x, y } = middlewareData.value.arrow ?? {}
	return {
		left: x != null ? `${x}px` : undefined,
		top: y != null ? `${y}px` : undefined,
		[edge]: '-5px',
		transform: `rotate(${rotate}deg)`,
	}
})

let hideTimer: ReturnType<typeof setTimeout> | undefined
let unregister: (() => void) | undefined

function keywordOrigin(p: string) {
	const [s, align] = p.split('-')
	const base = SIDES[(s as keyof typeof SIDES) ?? 'bottom'].origin
	if ((s === 'top' || s === 'bottom') && (align === 'start' || align === 'end')) {
		return base.replace('center', align === 'end' ? 'right' : 'left')
	}
	return base
}

function updateOrigin() {
	const trigger = triggerEl.value
	if (!trigger || !isPositioned.value) {
		return
	}
	const rect = trigger.getBoundingClientRect()
	const horizontal = side.value === 'left' || side.value === 'right'
	const originX = horizontal
		? (side.value === 'right' ? rect.right : rect.left) - x.value
		: rect.left + rect.width / 2 - x.value
	expandOrigin.value = `${originX}px ${rect.top + rect.height / 2 - y.value}px`
}

function listen() {
	document.addEventListener('click', onDocumentClick)
	document.addEventListener('keydown', onKeydown)
}

function unlisten() {
	document.removeEventListener('click', onDocumentClick)
	document.removeEventListener('keydown', onKeydown)
}

function show() {
	clearTimeout(hideTimer)
	if (props.disabled || isOpen.value) {
		return
	}
	dismissTooltip()
	expandOrigin.value = keywordOrigin(props.placement)
	isOpen.value = true
	unregister?.()
	unregister = registerFloatingMenu(close)
	listen()
	emit('open')
	nextTick(updateOrigin)
}

function close() {
	clearTimeout(hideTimer)
	if (!isOpen.value) {
		return
	}
	unlisten()
	unregister?.()
	unregister = undefined
	isOpen.value = false
	emit('close')
}

function hide() {
	if (hover.value) {
		clearTimeout(hideTimer)
		hideTimer = setTimeout(close, 80)
		return
	}
	close()
}

function toggle() {
	if (hover.value || props.disabled) {
		return
	}
	if (isOpen.value) {
		close()
	} else {
		show()
	}
}

function onTriggerClick(event: MouseEvent) {
	if (hover.value) {
		return
	}
	event.stopPropagation()
	toggle()
}

function onDocumentClick(event: MouseEvent) {
	if (
		!triggerEl.value?.contains(event.target as Node) &&
		!floating.value?.contains(event.target as Node)
	) {
		close()
	}
}

function onKeydown(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		close()
	}
}

onBeforeUnmount(() => {
	clearTimeout(hideTimer)
	unregister?.()
	unlisten()
})

defineExpose({ show, hide: close })
</script>
<template>
	<div
		ref="triggerEl"
		class="relative inline-flex"
		:class="`v-popper--theme-${theme}`"
		@click="onTriggerClick"
		@mouseenter="hover && show()"
		@mouseleave="hover && hide()"
		@focusin="hover && show()"
		@focusout="hover && hide()"
	>
		<slot />
		<Teleport to="body">
			<Transition name="floating-expand">
				<div
					v-if="isOpen"
					ref="floating"
					class="isolate z-[100000] overflow-visible"
					:class="`v-popper--theme-${theme}`"
					:style="[floatingStyles, expandStyle]"
					:data-popper-placement="placement"
					@click.stop
					@mouseenter="hover && show()"
					@mouseleave="hover && hide()"
					@focusin="hover && show()"
					@focusout="hover && hide()"
				>
					<div
						:class="[
							bare
								? 'overflow-y-auto'
								: 'v-popper__inner overflow-y-auto text-sm font-medium text-contrast',
							panelClass,
						]"
					>
						<slot name="popper" :hide="close" />
					</div>
					<div
						v-if="!bare"
						ref="arrowEl"
						class="tooltip-arrow pointer-events-none absolute size-2 border-b border-r border-0 border-solid border-surface-5 bg-surface-3"
						:style="arrowStyles"
					/>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>
