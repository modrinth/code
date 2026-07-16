<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'
import { RouterLink } from 'vue-router'

import IconButton from './IconButton.vue'
import type {
	ButtonSize,
	ButtonTone,
	ButtonVariant,
	OverflowMenuAction,
	OverflowMenuLink,
	OverflowMenuOption,
	TeleportPlacement,
} from './types'
import { useAnchoredTeleport } from './useAnchoredTeleport'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		label: string
		options: OverflowMenuOption[]
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		disabled?: boolean
		placement?: TeleportPlacement
	}>(),
	{
		variant: 'base',
		size: 'default',
		disabled: false,
		placement: 'bottom-end',
	},
)

const emit = defineEmits<{
	select: [option: OverflowMenuAction | OverflowMenuLink]
	open: []
	close: []
}>()

const triggerButton = ref<InstanceType<typeof IconButton> | null>(null)
const triggerElement = computed(() => triggerButton.value?.element ?? null)
const panelElement = ref<HTMLElement | null>(null)
const resolvedPlacement = computed(() => props.placement)
const menuId = `button-overflow-${useId()}`
const selectedIndex = ref(-1)
const typeahead = ref('')
let typeaheadTimer: ReturnType<typeof setTimeout> | undefined

const visibleOptions = computed(() => props.options.filter((option) => option.shown !== false))
const interactiveOptions = computed(() =>
	visibleOptions.value.filter(
		(option): option is OverflowMenuAction | OverflowMenuLink =>
			option.type !== 'divider' && !option.disabled,
	),
)

const { isOpen, panelStyle, open, close } = useAnchoredTeleport(
	triggerElement,
	panelElement,
	resolvedPlacement,
)

const menuItemClasses =
	'flex min-h-10 w-full items-center gap-2 rounded-[10px] border-0 bg-transparent px-3 py-2 text-left text-base font-semibold leading-5 text-contrast no-underline ' +
	'cursor-pointer whitespace-nowrap hover:bg-surface-4 focus-visible:bg-surface-4 focus-visible:outline-none ' +
	'disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 [&[aria-disabled=true]]:pointer-events-none [&[aria-disabled=true]]:opacity-50 ' +
	'[&>svg]:size-5 [&>svg]:shrink-0 [&>svg]:text-primary'

function isDivider(
	option: OverflowMenuOption,
): option is Extract<OverflowMenuOption, { type: 'divider' }> {
	return option.type === 'divider'
}

function isLink(option: OverflowMenuOption): option is OverflowMenuLink {
	return option.type === 'link'
}

function getMenuItems() {
	if (!panelElement.value) return []
	return Array.from(
		panelElement.value.querySelectorAll<HTMLElement>(
			'[role="menuitem"]:not([disabled]):not([aria-disabled="true"])',
		),
	)
}

function focusItem(index: number) {
	const items = getMenuItems()
	if (items.length === 0) return
	selectedIndex.value = (index + items.length) % items.length
	items[selectedIndex.value]?.focus()
}

async function openMenu(position: 'first' | 'last' = 'first') {
	if (props.disabled || isOpen.value) return
	await open()
	emit('open')
	await nextTick()
	focusItem(position === 'first' ? 0 : getMenuItems().length - 1)
}

function closeMenu(restoreFocus = false) {
	if (!isOpen.value) return
	selectedIndex.value = -1
	close(restoreFocus)
}

async function toggleMenu() {
	if (isOpen.value) closeMenu()
	else await openMenu()
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
	event.preventDefault()
	openMenu(event.key === 'ArrowDown' ? 'first' : 'last')
}

function handleAction(option: OverflowMenuAction, event: MouseEvent) {
	if (option.disabled) return
	option.action(event)
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

function handleLink(option: OverflowMenuLink, event: MouseEvent) {
	if (option.disabled) {
		event.preventDefault()
		return
	}
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

function handleMenuKeydown(event: KeyboardEvent) {
	const items = getMenuItems()
	if (items.length === 0) return

	switch (event.key) {
		case 'ArrowDown':
			event.preventDefault()
			focusItem(selectedIndex.value + 1)
			break
		case 'ArrowUp':
			event.preventDefault()
			focusItem(selectedIndex.value - 1)
			break
		case 'Home':
			event.preventDefault()
			focusItem(0)
			break
		case 'End':
			event.preventDefault()
			focusItem(items.length - 1)
			break
		case 'Escape':
			event.preventDefault()
			closeMenu(true)
			break
		case 'Tab':
			closeMenu()
			break
		default: {
			if (event.key.length !== 1 || event.ctrlKey || event.metaKey || event.altKey) return
			typeahead.value += event.key.toLocaleLowerCase()
			const match = interactiveOptions.value.findIndex((option) =>
				option.label.toLocaleLowerCase().startsWith(typeahead.value),
			)
			if (match >= 0) focusItem(match)
			if (typeaheadTimer) clearTimeout(typeaheadTimer)
			typeaheadTimer = setTimeout(() => {
				typeahead.value = ''
			}, 500)
		}
	}
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
	if (!openState && typeaheadTimer) {
		clearTimeout(typeaheadTimer)
		typeaheadTimer = undefined
		typeahead.value = ''
	}
})

defineExpose({ open: openMenu, close: closeMenu })
</script>

<template>
	<IconButton
		ref="triggerButton"
		v-bind="$attrs"
		:label="props.label"
		:variant="props.variant"
		:tone="props.tone"
		:size="props.size"
		:disabled="props.disabled"
		:aria-expanded="isOpen"
		:aria-controls="menuId"
		aria-haspopup="menu"
		@click="toggleMenu"
		@keydown="handleTriggerKeydown"
	>
		<slot />
	</IconButton>

	<Teleport to="body">
		<Transition
			enter-active-class="transition duration-125 ease-out"
			enter-from-class="scale-95 opacity-0"
			enter-to-class="scale-100 opacity-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="scale-100 opacity-100"
			leave-to-class="scale-95 opacity-0"
		>
			<div
				v-if="isOpen"
				:id="menuId"
				ref="panelElement"
				class="fixed isolate z-[9999] flex min-w-48 flex-col gap-1 rounded-[14px] bg-surface-3 p-2 shadow-lg ring-1 ring-surface-5"
				:style="panelStyle"
				role="menu"
				:aria-label="props.label"
				@keydown="handleMenuKeydown"
			>
				<template v-for="(option, index) in visibleOptions" :key="option.id ?? `divider-${index}`">
					<div v-if="isDivider(option)" role="separator" class="my-1 h-px bg-surface-5" />

					<RouterLink
						v-else-if="isLink(option) && option.to !== undefined && !option.disabled"
						v-tooltip="option.tooltip"
						:to="option.to"
						:class="[menuItemClasses, option.tone === 'red' ? 'text-red [&>svg]:text-red' : '']"
						role="menuitem"
						@click="handleLink(option, $event)"
						@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
					>
						<slot :name="option.id" :option="option">
							<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
							{{ option.label }}
						</slot>
					</RouterLink>

					<a
						v-else-if="isLink(option)"
						v-tooltip="option.tooltip"
						:href="option.disabled ? undefined : option.href"
						:target="option.target"
						:rel="option.rel ?? (option.target === '_blank' ? 'noopener noreferrer' : undefined)"
						:download="option.download"
						:aria-disabled="option.disabled || undefined"
						:tabindex="option.disabled ? -1 : undefined"
						:class="[menuItemClasses, option.tone === 'red' ? 'text-red [&>svg]:text-red' : '']"
						role="menuitem"
						@click="handleLink(option, $event)"
						@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
					>
						<slot :name="option.id" :option="option">
							<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
							{{ option.label }}
						</slot>
					</a>

					<button
						v-else
						v-tooltip="option.tooltip"
						type="button"
						:disabled="option.disabled"
						:class="[menuItemClasses, option.tone === 'red' ? 'text-red [&>svg]:text-red' : '']"
						role="menuitem"
						@click="handleAction(option, $event)"
						@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
					>
						<slot :name="option.id" :option="option">
							<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
							{{ option.label }}
						</slot>
					</button>
				</template>
			</div>
		</Transition>
	</Teleport>
</template>
