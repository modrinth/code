<template>
	<Transition name="fade">
		<div
			v-if="show"
			class="account-switch-overlay fixed inset-0 z-[10000] flex items-center justify-center backdrop-blur"
			role="status"
		>
			<span
				class="flex cursor-default select-none items-center gap-4 text-xl font-semibold text-contrast"
			>
				<RefreshCwIcon aria-hidden="true" class="h-6 w-6 animate-spin" />
				{{ formatMessage(messages.switchingAccounts) }}
			</span>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { RefreshCwIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{
	show: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	switchingAccounts: {
		id: 'layout.account-switcher.switching',
		defaultMessage: 'Switching accounts...',
	},
})
</script>

<style scoped>
.account-switch-overlay {
	background-color: color-mix(in srgb, var(--color-bg) 82%, transparent);
}

.fade-enter-active {
	transition: 0.25s ease-in-out;
}

.fade-enter-from {
	opacity: 0;
}
</style>
