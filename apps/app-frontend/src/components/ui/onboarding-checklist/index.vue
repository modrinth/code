<script setup lang="ts">
import { CheckIcon, RadioButtonIcon } from '@modrinth/assets'
import { Accordion, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onUnmounted, ref } from 'vue'

import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

const emit = defineEmits<{
	'create-instance': []
	'login-minecraft': []
	'login-modrinth': []
}>()

const { formatMessage } = useVIntl()
const {
	hasCreatedInstance,
	hasLoggedIntoMinecraft,
	hasLoggedIntoModrinth,
	isReady,
	showChecklist,
} = injectOnboardingChecklist()
const collapsedCornersVisible = ref(false)
let collapseTimer: ReturnType<typeof setTimeout> | undefined

const messages = defineMessages({
	title: {
		id: 'onboarding-checklist.title',
		defaultMessage: 'Getting started',
	},
	createInstance: {
		id: 'onboarding-checklist.create-instance',
		defaultMessage: 'Create first instance',
	},
	loginMinecraft: {
		id: 'onboarding-checklist.login-minecraft',
		defaultMessage: 'Log in to Minecraft',
	},
	loginModrinth: {
		id: 'onboarding-checklist.login-modrinth',
		defaultMessage: 'Log in to Modrinth',
	},
})

const steps = computed(() => [
	{
		id: 'create-instance',
		label: formatMessage(messages.createInstance),
		complete: hasCreatedInstance.value,
		action: () => emit('create-instance'),
	},
	{
		id: 'login-minecraft',
		label: formatMessage(messages.loginMinecraft),
		complete: hasLoggedIntoMinecraft.value,
		action: () => emit('login-minecraft'),
	},
	{
		id: 'login-modrinth',
		label: formatMessage(messages.loginModrinth),
		complete: hasLoggedIntoModrinth.value,
		action: () => emit('login-modrinth'),
	},
])

const accordionButtonClass = computed(
	() =>
		`flex w-full cursor-pointer items-center justify-between rounded-t-2xl border border-solid border-button-border bg-button-bg p-3 text-left text-contrast transition-[filter] hover:brightness-110${collapsedCornersVisible.value ? ' rounded-b-2xl' : ''}`,
)

function handleAccordionOpen() {
	clearTimeout(collapseTimer)
	collapsedCornersVisible.value = false
}

function handleAccordionClose() {
	clearTimeout(collapseTimer)
	collapseTimer = setTimeout(() => {
		collapsedCornersVisible.value = true
	}, 300)
}

onUnmounted(() => clearTimeout(collapseTimer))
</script>

<template>
	<div
		v-if="isReady && showChecklist"
		class="border-0 border-b-[1px] border-solid border-[--brand-gradient-border] p-3"
	>
		<Accordion
			open-by-default
			class="w-full overflow-hidden rounded-2xl text-base"
			:button-class="accordionButtonClass"
			content-class="flex flex-col gap-2 rounded-b-2xl border border-t-0 border-solid border-surface-5 bg-button-bg p-3"
			@on-open="handleAccordionOpen"
			@on-close="handleAccordionClose"
		>
			<template #title>
				<span class="font-semibold leading-6">{{ formatMessage(messages.title) }}</span>
			</template>
			<button
				v-for="step in steps"
				:key="step.id"
				type="button"
				class="flex h-10 w-full items-center gap-2 rounded-xl border border-solid border-button-border bg-button-bg px-4 py-2.5 text-left text-primary shadow-[0_1px_0.5px_rgb(0_0_0_/_12%)] transition-[filter]"
				:class="
					step.complete
						? '!cursor-default opacity-50'
						: 'cursor-pointer hover:brightness-110 active:brightness-90'
				"
				:disabled="step.complete"
				@click="step.action"
			>
				<span
					v-if="step.complete"
					class="flex size-[18px] items-center justify-center rounded-full bg-primary mr-0.5 relative left-px"
				>
					<CheckIcon class="size-3 invert [stroke-width:3] top-px" />
				</span>
				<RadioButtonIcon v-else class="size-5 shrink-0" />
				<span
					class="min-w-0 truncate font-medium leading-5"
					:class="{ 'text-secondary line-through': step.complete }"
				>
					{{ step.label }}
				</span>
			</button>
		</Accordion>
	</div>
</template>
