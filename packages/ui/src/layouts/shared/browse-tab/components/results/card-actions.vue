<script setup lang="ts">
import { Button, IconButton } from '#ui/components/base/buttons'

import type { CardAction } from '../../types'

defineProps<{
	actions: CardAction[]
}>()

function buttonType(action: CardAction) {
	if (action.type === 'transparent') return 'quiet'
	if (action.type === 'outlined') return 'outlined'
	return action.color ? 'colored' : 'base'
}

function buttonColor(action: CardAction) {
	const type = buttonType(action)
	return type === 'colored' || type === 'quiet' ? action.color : undefined
}

function buttonClass(action: CardAction) {
	if (action.type !== 'outlined' || !action.color) return undefined
	return {
		brand: '!text-brand [&>svg]:!text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]',
		red: '!text-red [&>svg]:!text-red !shadow-[inset_0_0_0_1px_var(--color-red)]',
		green: '!text-green [&>svg]:!text-green !shadow-[inset_0_0_0_1px_var(--color-green)]',
	}[action.color]
}
</script>

<template>
	<div class="flex gap-2">
		<template v-for="action in actions" :key="action.key">
			<IconButton
				v-if="action.circular"
				v-tooltip="action.tooltip"
				:type="buttonType(action)"
				:color="buttonColor(action)"
				:class="buttonClass(action)"
				:label="action.label || action.tooltip || action.key"
				:disabled="action.disabled"
				@click.stop="action.onClick"
			>
				<component :is="action.icon" :class="action.iconClass" />
			</IconButton>
			<Button
				v-else
				v-tooltip="action.tooltip"
				:type="buttonType(action)"
				:color="buttonColor(action)"
				:class="buttonClass(action)"
				:disabled="action.disabled"
				@click.stop="action.onClick"
			>
				<component :is="action.icon" :class="action.iconClass" />
				{{ action.label }}
			</Button>
		</template>
	</div>
</template>
