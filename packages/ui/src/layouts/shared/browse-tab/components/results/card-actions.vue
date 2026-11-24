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
	return buttonType(action) === 'base' ? undefined : action.color
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
				:label="action.label || action.tooltip || action.key"
				:disabled="action.disabled"
				@click.stop="action.onClick"
			>
				<component :is="action.icon" :class="action.iconClass" aria-hidden="true" />
			</IconButton>
			<Button
				v-else
				v-tooltip="action.tooltip"
				:type="buttonType(action)"
				:color="buttonColor(action)"
				:disabled="action.disabled"
				@click.stop="action.onClick"
			>
				<component :is="action.icon" :class="action.iconClass" aria-hidden="true" />
				{{ action.label }}
			</Button>
		</template>
	</div>
</template>
