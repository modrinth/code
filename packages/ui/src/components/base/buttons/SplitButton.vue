<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { computed, useSlots } from 'vue'

import Button from './Button.vue'
import ButtonGroup from './ButtonGroup.vue'
import TeleportOverflowMenu from './TeleportOverflowMenu.vue'
import type {
	ButtonNativeType,
	ButtonSize,
	ButtonTone,
	ButtonVariant,
	OverflowMenuAction,
	OverflowMenuLink,
	OverflowMenuOption,
	TeleportPlacement,
} from './types'

const props = withDefaults(
	defineProps<{
		menuLabel: string
		options: OverflowMenuOption[]
		groupLabel?: string
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		type?: ButtonNativeType
		disabled?: boolean
		primaryDisabled?: boolean
		menuDisabled?: boolean
		placement?: TeleportPlacement
	}>(),
	{
		groupLabel: undefined,
		variant: 'base',
		size: 'default',
		type: 'button',
		disabled: false,
		primaryDisabled: false,
		menuDisabled: false,
		placement: 'bottom-end',
	},
)

const emit = defineEmits<{
	click: [event: MouseEvent]
	select: [option: OverflowMenuAction | OverflowMenuLink]
}>()

const slots = useSlots()
const forwardedSlots = computed(() => Object.keys(slots).filter((name) => name !== 'default'))
</script>

<template>
	<ButtonGroup :label="props.groupLabel">
		<Button
			:variant="props.variant"
			:tone="props.tone"
			:size="props.size"
			:type="props.type"
			:disabled="props.disabled || props.primaryDisabled"
			@click="emit('click', $event)"
		>
			<slot />
		</Button>

		<TeleportOverflowMenu
			:label="props.menuLabel"
			:options="props.options"
			:variant="props.variant"
			:tone="props.tone"
			:size="props.size"
			:disabled="props.disabled || props.menuDisabled"
			:placement="props.placement"
			@select="emit('select', $event)"
		>
			<DropdownIcon aria-hidden="true" />
			<template v-for="slotName in forwardedSlots" #[slotName]="slotProps">
				<slot :name="slotName" v-bind="slotProps" />
			</template>
		</TeleportOverflowMenu>
	</ButtonGroup>
</template>
