<template>
	<Transition name="fade">
		<div
			v-if="isSwitchingAccount"
			class="fixed inset-0 z-[10000] flex items-center justify-center bg-black/80 backdrop-blur"
			role="status"
		>
			<span
				class="flex cursor-default select-none items-center gap-4 text-xl font-semibold text-white"
			>
				<RefreshCwIcon aria-hidden="true" class="h-6 w-6 animate-spin" />
				{{ formatMessage(messages.switchingAccounts) }}
			</span>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { RefreshCwIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'

import { useIsSwitchingAccount } from '~/composables/accounts.ts'

const { formatMessage } = useVIntl()

const isSwitchingAccount = useIsSwitchingAccount()

const messages = defineMessages({
	switchingAccounts: {
		id: 'layout.account-switcher.switching',
		defaultMessage: 'Switching accounts...',
	},
})
</script>

<style scoped>
.fade-enter-active {
	transition: 0.25s ease-in-out;
}

.fade-enter-from {
	opacity: 0;
}
</style>
