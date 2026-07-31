<template>
	<PopoutMenu
		ref="dropdown"
		v-bind="$attrs"
		:disabled="disabled"
		:dropdown-id="dropdownId"
		:tooltip="tooltip"
		:placement="placement"
	>
		<slot></slot>
		<template #menu>
			<slot name="menu-header" />
			<template v-for="(option, index) in options.filter((x) => x.shown === undefined || x.shown)">
				<div
					v-if="isDivider(option)"
					:key="`divider-${index}`"
					class="h-px mx-[0.625rem] my-2 bg-surface-5"
				></div>
				<ButtonLink
					v-else-if="option.link"
					:key="`option-${option.id}`"
					v-tooltip="option.tooltip"
					class="w-full justify-start whitespace-nowrap"
					type="quiet"
					:color="optionButtonColor(option)"
					:v-close-popper="!option.remainOnClick"
					:href="option.link"
					:download="option.download"
					:target="option.external ? '_blank' : undefined"
					:disabled="option.disabled"
					@click="!option.remainOnClick && close()"
				>
					<template v-if="!$slots[option.id]">
						<component :is="option.icon" v-if="option.icon" class="size-5" />
						{{ option.id }}
					</template>
					<slot :name="option.id"></slot>
				</ButtonLink>
				<Button
					v-else
					:key="`option-${option.id}`"
					v-tooltip="option.tooltip"
					class="w-full justify-start whitespace-nowrap"
					type="quiet"
					:color="optionButtonColor(option)"
					:v-close-popper="!option.remainOnClick"
					:disabled="option.disabled"
					@click="handleOptionAction(option, $event)"
				>
					<template v-if="!$slots[option.id]">
						<component :is="option.icon" v-if="option.icon" class="size-5" />
						{{ option.id }}
					</template>
					<slot :name="option.id"></slot>
				</Button>
			</template>
		</template>
	</PopoutMenu>
</template>

<script setup lang="ts">
import { type Component, type Ref, ref } from 'vue'

import Button from './buttons/Button.vue'
import ButtonLink from './buttons/ButtonLink.vue'
import type { ButtonColor } from './buttons/types'
import PopoutMenu from './PopoutMenu.vue'

interface BaseOption {
	shown?: boolean
}

interface Divider extends BaseOption {
	divider?: boolean
}

interface Item extends BaseOption {
	id: string
	icon?: Component
	action?: (event?: MouseEvent) => void
	link?: string
	download?: string
	external?: boolean
	color?:
		| 'primary'
		| 'danger'
		| 'secondary'
		| 'highlight'
		| 'red'
		| 'orange'
		| 'green'
		| 'blue'
		| 'purple'
	hoverFilled?: boolean
	hoverFilledOnly?: boolean
	remainOnClick?: boolean
	disabled?: boolean
	tooltip?: string
}

export type Option = Divider | Item

withDefaults(
	defineProps<{
		options: Option[]
		disabled?: boolean
		dropdownId?: string
		tooltip?: string
		placement?: string
	}>(),
	{
		options: () => [],
		disabled: false,
		dropdownId: undefined,
		tooltip: undefined,
		placement: 'bottom-end',
	},
)

defineOptions({
	inheritAttrs: false,
})

const dropdown: Ref<InstanceType<typeof PopoutMenu> | null> = ref(null)

const close = () => {
	dropdown.value?.hide()
}

const open = () => {
	dropdown.value?.show()
}

function handleOptionAction(option: Item, event: MouseEvent) {
	option.action?.(event)
	if (!option.remainOnClick) {
		close()
	}
}

function optionButtonColor(option: Item): ButtonColor | undefined {
	switch (option.color) {
		case 'primary':
			return 'brand'
		case 'danger':
		case 'red':
			return 'red'
		case 'orange':
		case 'green':
		case 'blue':
		case 'purple':
			return option.color
		default:
			return undefined
	}
}

function isDivider(option: BaseOption): option is Divider {
	return 'divider' in option
}

defineExpose({ open, close })
</script>

<style lang="scss" scoped>
.btn {
	white-space: nowrap;
	width: 100%;
	box-shadow: none;
	--text-color: var(--color-base);
	--background-color: transparent;
	justify-content: flex-start;
	padding: 0.55rem 0.625rem;
}
</style>
