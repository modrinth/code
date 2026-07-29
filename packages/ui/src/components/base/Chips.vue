<template>
	<div class="chips" role="radiogroup" :aria-label="ariaLabel">
		<ButtonStyled
			v-for="item in items"
			:key="formatLabel(item)"
			type="chip"
			:size="size"
			:color="selected === item ? 'brand' : 'standard'"
		>
			<button
				v-tooltip="isDisabled(item) ? disabledTooltip : undefined"
				role="radio"
				:aria-checked="selected === item"
				:disabled="isDisabled(item)"
				:class="{ capitalize }"
				@click="toggleItem(item)"
			>
				<CheckIcon v-if="selected === item && !hideCheckmarkIcon" />
				<span>{{ formatLabel(item) }}</span>
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts" generic="T">
import { CheckIcon } from '@modrinth/assets'

import ButtonStyled from './ButtonStyled.vue'

const props = withDefaults(
	defineProps<{
		items: T[]
		formatLabel?: (item: T) => string
		neverEmpty?: boolean
		capitalize?: boolean
		size?: 'standard' | 'small'
		ariaLabel?: string
		disabledItems?: T[]
		disabledTooltip?: string
		hideCheckmarkIcon?: boolean
	}>(),
	{
		neverEmpty: true,
		// Intentional any type, as this default should only be used for primitives (string or number)
		formatLabel: (item) => item.toString(),
		capitalize: true,
		size: 'standard',
	},
)

const selected = defineModel<T | null>()

// If one always has to be selected, default to the first one
if (props.items.length > 0 && props.neverEmpty && !selected.value) {
	selected.value = props.items[0]
}

function isDisabled(item: T): boolean {
	return props.disabledItems?.includes(item) ?? false
}

function toggleItem(item: T) {
	if (isDisabled(item)) return
	if (selected.value === item && !props.neverEmpty) {
		selected.value = null
	} else {
		selected.value = item
	}
}
</script>

<style lang="scss" scoped>
.chips {
	display: flex;
	grid-gap: 0.5rem;
	flex-wrap: wrap;

	:deep(button) {
		transition-property: scale, filter !important;
	}

	.capitalize {
		text-transform: capitalize;
	}
}
</style>
