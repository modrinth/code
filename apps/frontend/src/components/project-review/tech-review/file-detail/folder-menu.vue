<script setup lang="ts">
import { FloatingMenu } from '@modrinth/ui'
import { onBeforeUnmount, provide, ref } from 'vue'

import { registerFolderMenuChildKey } from '../menus'

const props = withDefaults(defineProps<{ disabled?: boolean; showDelay?: number }>(), {
	disabled: false,
	showDelay: 250,
})
const menu = ref<InstanceType<typeof FloatingMenu>>()
const trigger = ref<HTMLElement>()
const panel = ref<HTMLElement>()
let showTimer: ReturnType<typeof setTimeout> | undefined
let hideTimer: ReturnType<typeof setTimeout> | undefined
let pointerWithin = false
let openChildren = 0
let disposed = false

function clearTimers() {
	clearTimeout(showTimer)
	clearTimeout(hideTimer)
}

function show(delayed = false) {
	clearTimers()
	if (props.disabled) return
	if (delayed) showTimer = setTimeout(() => menu.value?.show(), props.showDelay)
	else menu.value?.show()
}

function hide() {
	clearTimers()
	hideTimer = setTimeout(() => {
		const activeElement = trigger.value?.ownerDocument.activeElement
		const focusWithin =
			!!activeElement &&
			(trigger.value?.contains(activeElement) || panel.value?.contains(activeElement))
		if (!pointerWithin && openChildren === 0 && !focusWithin) menu.value?.hide()
	}, 80)
}

function enter(delayed = false) {
	pointerWithin = true
	show(delayed)
}

function leave() {
	pointerWithin = false
	hide()
}

provide(registerFolderMenuChildKey, () => {
	openChildren++
	clearTimeout(hideTimer)
	return () => {
		openChildren--
		if (!disposed) hide()
	}
})

onBeforeUnmount(() => {
	disposed = true
	clearTimers()
})
</script>

<template>
	<FloatingMenu
		ref="menu"
		trigger="click"
		:disabled="disabled"
		placement="right-start"
		:arrow="false"
		bare
		@close="clearTimers"
	>
		<div
			ref="trigger"
			@mouseenter="enter(true)"
			@mouseleave="leave"
			@focusin="show()"
			@focusout="hide"
			@click.stop="clearTimers"
		>
			<slot />
		</div>
		<template #popper>
			<div
				ref="panel"
				@mouseenter="enter()"
				@mouseleave="leave"
				@focusin="show()"
				@focusout="hide"
			>
				<slot name="popper" />
			</div>
		</template>
	</FloatingMenu>
</template>
