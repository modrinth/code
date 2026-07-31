<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { computed, useSlots } from 'vue'

import Button from './Button.vue'
import ButtonGroup from './ButtonGroup.vue'
import TeleportOverflowMenu from './TeleportOverflowMenu.vue'
import type {
	ButtonColor,
	ButtonNativeType,
	ButtonSize,
	ButtonType,
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
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		nativeType?: ButtonNativeType
		disabled?: boolean
		primaryDisabled?: boolean
		menuDisabled?: boolean
		placement?: TeleportPlacement
	}>(),
	{
		groupLabel: undefined,
		type: 'base',
		size: 'md',
		nativeType: 'button',
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
			:type="props.type"
			:color="props.color"
			:size="props.size"
			:native-type="props.nativeType"
			:disabled="props.disabled || props.primaryDisabled"
			@click="emit('click', $event)"
		>
			<slot />
		</Button>

		<TeleportOverflowMenu
			:label="props.menuLabel"
			:options="props.options"
			:type="props.type"
			:color="props.color"
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
