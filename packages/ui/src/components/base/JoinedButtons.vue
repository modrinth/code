<template>
	<div class="joined-buttons">
		<ButtonStyled
			:color="color"
			:size="size"
			:class="{ 'joined-buttons__primary--muted': primaryMuted }"
		>
			<button :disabled="primaryDisabledResolved" @click="handlePrimaryAction">
				<component :is="primaryAction.icon" v-if="primaryAction.icon" aria-hidden="true" />
				{{ primaryAction.label }}
			</button>
		</ButtonStyled>
		<ButtonStyled
			v-if="dropdownActions.length > 0"
			:color="color"
			:size="size"
			class="joined-buttons__dropdown"
		>
			<OverflowMenu
				class="btn-dropdown-animation !w-10"
				:options="dropdownOptions"
				:disabled="dropdownDisabledResolved"
			>
				<DropdownIcon />
				<template v-for="action in dropdownActions" :key="action.id" #[action.id]>
					<component :is="action.icon" v-if="action.icon" aria-hidden="true" />
					{{ action.label }}
				</template>
			</OverflowMenu>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import type { Component } from 'vue'
import { computed } from 'vue'

import { ButtonStyled, OverflowMenu } from '../index'

// TODO: This should be moved to a shared types file.
type Colors = 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'

export interface JoinedButtonAction {
	id: string
	label: string
	icon?: Component
	action: () => void
	color?: Colors
	hoverFilled?: boolean
}

interface Props {
	actions: JoinedButtonAction[]
	color?: Colors
	size?: 'standard' | 'large' | 'small'
	disabled?: boolean
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	color: 'standard',
	size: 'standard',
	disabled: false,
	primaryDisabled: undefined,
	dropdownDisabled: undefined,
	primaryMuted: false,
})

const primaryDisabledResolved = computed(() => props.primaryDisabled ?? props.disabled)
const dropdownDisabledResolved = computed(() => props.dropdownDisabled ?? props.disabled)

const primaryAction = computed(() => props.actions[0])

const dropdownActions = computed(() => props.actions.slice(1))

const colorMap: Record<
	Colors,
	| 'red'
	| 'orange'
	| 'green'
	| 'blue'
	| 'purple'
	| 'highlight'
	| 'primary'
	| 'danger'
	| 'secondary'
	| undefined
> = {
	standard: 'secondary',
	brand: 'primary',
	red: 'red',
	orange: 'orange',
	green: 'green',
	blue: 'blue',
	purple: 'purple',
}

const dropdownOptions = computed(() =>
	dropdownActions.value.map((action) => ({
		id: action.id,
		color: action.color ? colorMap[action.color] : undefined,
		action: action.action,
		hoverFilled: action.hoverFilled ?? true,
	})),
)

function handlePrimaryAction() {
	if (primaryAction.value && !primaryDisabledResolved.value) {
		primaryAction.value.action()
	}
}
</script>

<style scoped>
.joined-buttons {
	display: flex;
	align-items: center;
}

.joined-buttons > :deep(.btn) {
	border-radius: 0;
}

.joined-buttons > :deep(.btn:first-child) {
	border-top-left-radius: var(--radius-md);
	border-bottom-left-radius: var(--radius-md);
}

.joined-buttons > :deep(.btn:last-child) {
	border-top-right-radius: var(--radius-md);
	border-bottom-right-radius: var(--radius-md);
	margin-left: -1px;
}

.joined-buttons > :deep(.btn:not(:last-child)) {
	border-right: none;
}

.btn-dropdown-animation {
	padding: 0.5rem !important;
}

.joined-buttons__primary--muted {
	opacity: 0.6;
}
</style>
