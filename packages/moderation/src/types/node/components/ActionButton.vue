<template>
	<!-- A toggle with sub-options gets a hover menu instead of a plain tooltip, so a reason can
	     be picked straight from the hover without first clicking the toggle itself active. It
	     opens on hover but never auto-closes on mouse-leave (`hideTriggers`/`popperHideTriggers`
	     left empty) — a sub-option can itself be a dropdown (e.g. Combobox) whose own listbox
	     teleports elsewhere in the DOM, so moving the cursor towards it would otherwise register
	     as "left the menu" mid-click and close it before a pick lands. Only an outside click
	     (`autoHide`) or opening a *different* menu (forced below, `onMenuShow`) closes it.
	     These are plain component props (not a custom theme) so their values are unambiguous —
	     floating-vue's theme-level config resolution for these specific keys isn't reliable. -->
	<Dropdown
		v-if="$slots.menu"
		ref="dropdownRef"
		theme="dropdown"
		popper-class="checklist-hover-menu-popper"
		placement="bottom-start"
		:show-triggers="['hover']"
		:hide-triggers="[]"
		:popper-show-triggers="['hover']"
		:popper-hide-triggers="[]"
		auto-hide
		@apply-show="onMenuShow"
		@apply-hide="onMenuHide"
	>
		<Button
			:type="color === 'standard' ? 'base' : 'colored'"
			:color="color === 'standard' ? undefined : color"
			:disabled="disabled"
			:size="props.size"
			:interaction="props.interaction"
			:native-type="props.nativeType"
			:loading="props.loading"
			:aria-label="icon ? label : undefined"
			@click="emit('update:modelValue', !modelValue)"
		>
			<component :is="icon" v-if="icon" />
			<template v-else>{{ label }}</template>
		</Button>
		<template #popper>
			<div class="flex max-w-xs flex-col gap-1.5 p-2 text-sm">
				<p v-if="label" class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">
					{{ label }}
				</p>
				<slot name="menu" />
			</div>
		</template>
	</Dropdown>
	<Button
		v-else
		v-tooltip="tooltip"
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		:size="props.size"
		:interaction="props.interaction"
		:native-type="props.nativeType"
		:loading="props.loading"
		:aria-label="icon ? label : undefined"
		@click="emit('update:modelValue', !modelValue)"
	>
		<component :is="icon" v-if="icon" />
		<template v-else>{{ label }}</template>
	</Button>
</template>

<script lang="ts" setup>
import { Button } from '@modrinth/ui'
import type { ButtonProps } from '@modrinth/ui/src/components/base/buttons/types.ts'
import { Dropdown } from 'floating-vue'
import type { Component } from 'vue'
import { computed, ref } from 'vue'

import { clearChecklistMenu, closeOtherChecklistMenus } from './action-button-menu'

const props = defineProps<
	{
		modelValue: boolean
		label?: string
		icon?: Component
		needsAttention?: boolean
		fixActionable?: boolean
		tooltip?: Record<string, unknown>
	} & Omit<ButtonProps, 'type' | 'color'>
>()

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()

const color = computed(() => {
	if (!props.modelValue) return 'standard'
	if (props.needsAttention) return 'orange'
	return props.fixActionable ? 'blue' : 'brand'
})

const dropdownRef = ref<{ hide: () => void } | null>(null)

function hideThisMenu() {
	dropdownRef.value?.hide()
}

function onMenuShow() {
	closeOtherChecklistMenus(hideThisMenu)
}

function onMenuHide() {
	clearChecklistMenu(hideThisMenu)
}
</script>

<style scoped>
/* The popper teleports outside this component, so it has to be reached with :global(). Floating-
   vue's default popper z-index (10000) sits *above* a nested Combobox's own listbox (z-[9999] in
   packages/ui/src/components/base/Combobox.vue), which otherwise renders behind this menu instead
   of on top of it. */
:global(.v-popper__popper.checklist-hover-menu-popper) {
	z-index: 60;
}
</style>
