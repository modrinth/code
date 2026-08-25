<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'

import { getOverflowMenuItemAttrs, isLink, overflowMenuItemClasses } from './overflow-menu'
import type { OverflowMenuAction, OverflowMenuLink } from './types'

const trailingActionClasses =
	'flex shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-2 text-secondary opacity-0 ' +
	'pointer-events-none hover:bg-surface-5 hover:text-contrast focus-visible:outline-none ' +
	'group-hover/overflow-menu-item:pointer-events-auto group-hover/overflow-menu-item:opacity-100 ' +
	'focus-visible:pointer-events-auto focus-visible:opacity-100 ' +
	'[&>svg]:size-5'

const props = defineProps<{
	option: OverflowMenuAction | OverflowMenuLink
	submenuItem?: boolean
}>()

const emit = defineEmits<{
	select: [option: OverflowMenuAction | OverflowMenuLink, event: MouseEvent]
	focus: [element: HTMLElement]
}>()

const wrapperElement = ref<HTMLElement | null>(null)
const trailingElement = ref<HTMLElement | null>(null)

const itemAttrs = computed(() => ({
	...getOverflowMenuItemAttrs(props.option),
	'data-overflow-submenu-item': props.submenuItem || undefined,
	'aria-current': props.option.selected || undefined,
}))

function handleClick(event: MouseEvent) {
	if (props.option.disabled) {
		event.preventDefault()
		return
	}
	if (!isLink(props.option)) props.option.action(event)
	emit('select', props.option, event)
}

function handleKeydown(event: KeyboardEvent) {
	if (event.key === 'ArrowRight' && props.option.trailingAction) {
		event.preventDefault()
		event.stopPropagation()
		trailingElement.value?.focus()
		return
	}

	// links don't fire on space the way buttons do
	if (event.key !== ' ' || !isLink(props.option)) return
	event.preventDefault()
	if (props.option.disabled) return
	;(event.currentTarget as HTMLElement).click()
}

function focusRow() {
	wrapperElement.value?.querySelector<HTMLElement>('[role="menuitem"]')?.focus()
}

function handleTrailingKeydown(event: KeyboardEvent) {
	if (event.key === 'ArrowLeft') {
		event.preventDefault()
		event.stopPropagation()
		focusRow()
		return
	}

	if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) focusRow() // so the menu still moves from this row
}

function handleFocus(event: FocusEvent) {
	emit('focus', event.currentTarget as HTMLElement)
}
</script>

<template>
	<div ref="wrapperElement" class="group/overflow-menu-item flex items-center">
		<RouterLink
			v-if="isLink(props.option) && props.option.to !== undefined && !props.option.disabled"
			v-tooltip="props.option.tooltip"
			v-bind="itemAttrs"
			:to="props.option.to"
			:class="[overflowMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<slot />
			<CheckIcon v-if="props.option.selected" aria-hidden="true" class="ml-auto !text-green" />
		</RouterLink>

		<a
			v-else-if="isLink(props.option)"
			v-tooltip="props.option.tooltip"
			v-bind="itemAttrs"
			:href="props.option.disabled ? undefined : props.option.href"
			:target="props.option.target"
			:rel="
				props.option.rel ?? (props.option.target === '_blank' ? 'noopener noreferrer' : undefined)
			"
			:download="props.option.download"
			:aria-disabled="props.option.disabled || undefined"
			:class="[overflowMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<slot />
			<CheckIcon v-if="props.option.selected" aria-hidden="true" class="ml-auto !text-green" />
		</a>

		<button
			v-else
			v-tooltip="props.option.tooltip"
			v-bind="itemAttrs"
			type="button"
			:aria-disabled="props.option.disabled || undefined"
			:class="[overflowMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<slot />
			<CheckIcon v-if="props.option.selected" aria-hidden="true" class="ml-auto !text-green" />
		</button>

		<button
			v-if="props.option.trailingAction"
			ref="trailingElement"
			v-tooltip="props.option.trailingAction.label"
			type="button"
			:aria-label="props.option.trailingAction.label"
			:class="trailingActionClasses"
			tabindex="-1"
			@click="props.option.trailingAction.action($event)"
			@keydown="handleTrailingKeydown"
		>
			<component :is="props.option.trailingAction.icon" aria-hidden="true" />
		</button>
	</div>
</template>
