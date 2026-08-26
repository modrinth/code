<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'

import type { ButtonMenuAction, ButtonMenuLink } from '../types'
import {
	buttonMenuItemClasses,
	buttonMenuTones,
	getButtonMenuItemAttrs,
	isLink,
} from './button-menu'

const trailingActionClasses =
	'button-menu-trailing relative flex size-10 shrink-0 cursor-pointer items-center justify-center border-0 bg-transparent p-0 opacity-0 ' +
	"pointer-events-none before:pointer-events-none before:absolute before:size-8 before:rounded-full before:content-[''] " +
	'focus-visible:outline-none ' +
	'group-hover/button-menu-item:pointer-events-auto group-hover/button-menu-item:opacity-100 ' +
	'focus-visible:pointer-events-auto focus-visible:opacity-100 ' +
	'[&>svg]:relative [&>svg]:z-[1] [&>svg]:size-5'

const props = defineProps<{
	option: ButtonMenuAction | ButtonMenuLink
	submenuItem?: boolean
}>()

const emit = defineEmits<{
	select: [option: ButtonMenuAction | ButtonMenuLink, event: MouseEvent]
	focus: [element: HTMLElement]
}>()

const wrapperElement = ref<HTMLElement | null>(null)
const trailingElement = ref<HTMLElement | null>(null)

const trailingActionStyle = computed(() => {
	const color = props.option.trailingAction?.color
	if (!color) return undefined
	return { '--button-menu-trailing-color': buttonMenuTones[color] }
})

const itemAttrs = computed(() => ({
	...getButtonMenuItemAttrs(props.option),
	'data-button-menu-submenu-item': props.submenuItem || undefined,
	'aria-checked': typeof props.option.selected === 'boolean' ? props.option.selected : undefined,
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
	<div ref="wrapperElement" class="group/button-menu-item flex items-center">
		<RouterLink
			v-if="isLink(props.option) && props.option.to !== undefined && !props.option.disabled"
			v-tooltip="props.option.tooltip"
			v-bind="itemAttrs"
			:to="props.option.to"
			:class="[buttonMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<RadioButtonCheckedIcon
				v-if="props.option.selected === true"
				aria-hidden="true"
				class="!text-brand"
			/>
			<RadioButtonIcon
				v-else-if="props.option.selected === false"
				aria-hidden="true"
				class="!text-secondary"
			/>
			<slot />
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
			:class="[buttonMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<RadioButtonCheckedIcon
				v-if="props.option.selected === true"
				aria-hidden="true"
				class="!text-brand"
			/>
			<RadioButtonIcon
				v-else-if="props.option.selected === false"
				aria-hidden="true"
				class="!text-secondary"
			/>
			<slot />
		</a>

		<button
			v-else
			v-tooltip="props.option.tooltip"
			v-bind="itemAttrs"
			type="button"
			:aria-disabled="props.option.disabled || undefined"
			:class="[buttonMenuItemClasses, 'flex-1']"
			@click="handleClick"
			@keydown="handleKeydown"
			@focus="handleFocus"
		>
			<RadioButtonCheckedIcon
				v-if="props.option.selected === true"
				aria-hidden="true"
				class="!text-brand"
			/>
			<RadioButtonIcon
				v-else-if="props.option.selected === false"
				aria-hidden="true"
				class="!text-secondary"
			/>
			<slot />
		</button>

		<button
			v-if="props.option.trailingAction"
			ref="trailingElement"
			v-tooltip="props.option.trailingAction.label"
			type="button"
			:aria-label="props.option.trailingAction.label"
			:class="trailingActionClasses"
			:style="trailingActionStyle"
			tabindex="-1"
			@click="props.option.trailingAction.action($event)"
			@keydown="handleTrailingKeydown"
		>
			<component :is="props.option.trailingAction.icon" aria-hidden="true" />
		</button>
	</div>
</template>
