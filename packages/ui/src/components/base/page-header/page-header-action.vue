<template>
	<component
		:is="action.component"
		v-if="'kind' in action && action.kind === 'component'"
		:class="action.class"
		v-bind="action.componentProps"
	/>
	<JoinedButtons
		v-else-if="'joinedActions' in action && action.joinedActions?.length"
		:actions="joinedActions"
		:color="joinedActionColor(action.color)"
		:size="action.size ?? 'large'"
		:disabled="action.disabled"
		:primary-disabled="action.primaryDisabled"
		:dropdown-disabled="action.dropdownDisabled"
		:primary-muted="action.primaryMuted"
	/>
	<ButtonStyled
		v-else
		:color="action.color ?? 'standard'"
		:size="action.size ?? 'large'"
		:type="action.type ?? 'standard'"
		:circular="action.circular ?? action.labelHidden ?? false"
	>
		<TeleportOverflowMenu
			v-if="'menuActions' in action && action.menuActions?.length"
			:options="action.menuActions"
			:tooltip="action.tooltip"
			:aria-label="actionLabel"
			:disabled="action.disabled"
			@open="emit('dismiss-prompt')"
		>
			<PageHeaderActionContent :action="action" />
		</TeleportOverflowMenu>
		<PageHeaderInteractiveWrapper
			v-else
			:to="'to' in action ? action.to : undefined"
			clickable
			:disabled="action.disabled"
			:tooltip="action.tooltip"
			:aria-label="actionLabel"
			base-class=""
			@click="handleClick"
		>
			<PageHeaderActionContent :action="action" />
		</PageHeaderInteractiveWrapper>
	</ButtonStyled>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import ButtonStyled from '../ButtonStyled.vue'
import JoinedButtons, { type JoinedButtonAction } from '../JoinedButtons.vue'
import TeleportOverflowMenu from '../TeleportOverflowMenu.vue'
import PageHeaderActionContent from './page-header-action-content.vue'
import PageHeaderInteractiveWrapper from './page-header-interactive-wrapper.vue'
import type { ButtonColor, JoinedButtonColor, PageHeaderAction } from './types'

const props = defineProps<{
	action: PageHeaderAction
}>()

const emit = defineEmits<{
	'dismiss-prompt': []
}>()

const actionLabel = computed(() => props.action.ariaLabel ?? props.action.tooltip ?? props.action.label)

const joinedActions = computed<JoinedButtonAction[]>(() => {
	if (!('joinedActions' in props.action)) return []

	return props.action.joinedActions.map((joinedAction) => ({
		...joinedAction,
		action: () => {
			emit('dismiss-prompt')
			joinedAction.action()
		},
	}))
})

function handleClick(event: MouseEvent) {
	emit('dismiss-prompt')
	void props.action.onClick?.(event)
}

function joinedActionColor(color?: ButtonColor): JoinedButtonColor | undefined {
	return color === 'medal-promo' ? 'standard' : color
}
</script>
