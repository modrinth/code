<script setup lang="ts">
import { ListTodoIcon, MessagesSquareIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	projectPublishingColors,
	type ProjectPublishingState,
	projectPublishingTones,
} from '~/providers/project-publishing'

type DrawerView = 'moderation' | 'thread'

const props = defineProps<{
	state: ProjectPublishingState
	remaining: number
	controls: string
}>()
const emit = defineEmits<{ open: [target: DrawerView] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	openChecklist: {
		id: 'project.settings.review.open-checklist',
		defaultMessage:
			'Open review checklist ({count, plural, one {# item remaining} other {# items remaining}})',
	},
	openThread: {
		id: 'project.settings.review.open-thread',
		defaultMessage: 'Open moderation thread',
	},
})
const launcherColor = computed(() => projectPublishingColors[projectPublishingTones[props.state]])

const checklistTrigger = ref<InstanceType<typeof IconButton>>()
const threadTrigger = ref<InstanceType<typeof IconButton>>()

function focusTrigger(target: DrawerView) {
	const trigger = target === 'thread' ? threadTrigger : checklistTrigger
	trigger.value?.element?.focus()
}

defineExpose({ focusTrigger })
</script>

<template>
	<div
		class="drawer-launcher fixed right-0 top-[47.5%] z-30 flex -translate-y-1/2 flex-col rounded-l-2xl border border-r-0 border-solid p-1 shadow-lg"
		:style="{ '--launcher-color': launcherColor ? `var(--color-${launcherColor})` : undefined }"
	>
		<IconButton
			ref="checklistTrigger"
			class="group/checklist-trigger"
			type="quiet"
			:color="launcherColor"
			:interaction="launcherColor ? 'filled' : 'surface'"
			size="xl"
			:circular="false"
			:label="formatMessage(messages.openChecklist, { count: remaining })"
			:aria-controls="controls"
			aria-expanded="false"
			@click="emit('open', 'moderation')"
		>
			<ListTodoIcon aria-hidden="true" />
			<span
				aria-hidden="true"
				class="pointer-events-none absolute -right-0 -top-0 flex aspect-square h-4 min-w-4 items-center justify-center rounded-full text-[10px] leading-none group-hover/checklist-trigger:brightness-[115%]"
				:class="
					launcherColor
						? 'bg-[--launcher-color] text-[var(--color-accent-contrast)]'
						: 'bg-surface-4 text-contrast'
				"
			>
				{{ remaining }}
			</span>
		</IconButton>
		<IconButton
			ref="threadTrigger"
			type="quiet"
			:color="launcherColor"
			:interaction="launcherColor ? 'filled' : 'surface'"
			size="xl"
			:circular="false"
			:label="formatMessage(messages.openThread)"
			:aria-controls="controls"
			aria-expanded="false"
			@click="emit('open', 'thread')"
		>
			<MessagesSquareIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>

<style scoped>
.drawer-launcher {
	--hover-brightness: 1;
	background-color: color-mix(
		in srgb,
		var(--launcher-color, var(--surface-2)) 14%,
		var(--surface-2)
	);
	border-color: color-mix(in srgb, var(--launcher-color, var(--surface-4)) 35%, var(--surface-4));
}
</style>
