<script setup lang="ts">
import { FloatingMenu } from '@modrinth/ui'
import { inject, onBeforeUnmount, provide, ref, watch } from 'vue'

import { folderMenuLockKey, registerFolderMenuChildKey } from '../menus'

const props = withDefaults(defineProps<{ disabled?: boolean; showDelay?: number }>(), {
	disabled: false,
	showDelay: 250,
})
const menu = ref<InstanceType<typeof FloatingMenu>>()
const trigger = ref<HTMLElement>()
const panel = ref<HTMLElement>()
const lockedMenu = inject(folderMenuLockKey, ref<symbol>())
const menuId = Symbol('FolderMenu')
let showTimer: ReturnType<typeof setTimeout> | undefined
let hideTimer: ReturnType<typeof setTimeout> | undefined
let pointerWithin = false
const childMenus = new Set<(event: MouseEvent) => void>()
let disposed = false

function clearTimers() {
	clearTimeout(showTimer)
	clearTimeout(hideTimer)
}

function show(delayed = false) {
	clearTimers()
	if (!canShow()) return
	if (delayed) {
		showTimer = setTimeout(() => {
			if (canShow()) menu.value?.show()
		}, props.showDelay)
	} else menu.value?.show()
}

function canShow() {
	return !props.disabled && (!lockedMenu.value || lockedMenu.value === menuId)
}

watch(lockedMenu, (owner) => {
	if (owner && owner !== menuId) {
		clearTimers()
		menu.value?.hide()
	}
})

function hide() {
	clearTimers()
	hideTimer = setTimeout(() => {
		const activeElement = trigger.value?.ownerDocument.activeElement
		const focusWithin =
			!!activeElement &&
			(trigger.value?.contains(activeElement) || panel.value?.contains(activeElement))
		if (!pointerWithin && childMenus.size === 0 && !focusWithin) menu.value?.hide()
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

function onPanelClick(event: MouseEvent) {
	for (const dismiss of [...childMenus]) dismiss(event)
}

provide(registerFolderMenuChildKey, (dismiss) => {
	childMenus.add(dismiss)
	lockedMenu.value = menuId
	clearTimeout(hideTimer)
	return () => {
		childMenus.delete(dismiss)
		if (childMenus.size === 0 && lockedMenu.value === menuId) lockedMenu.value = undefined
		if (!disposed) hide()
	}
})

onBeforeUnmount(() => {
	disposed = true
	if (lockedMenu.value === menuId) lockedMenu.value = undefined
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
				@click.capture="onPanelClick"
			>
				<slot name="popper" />
			</div>
		</template>
	</FloatingMenu>
</template>
