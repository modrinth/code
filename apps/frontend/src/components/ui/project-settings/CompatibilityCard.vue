<template>
	<ServerCompatibilityModal ref="serverCompatibilityModal" />
	<div class="flex items-center justify-between gap-4 rounded-2xl bg-bg-raised p-6">
		<div class="flex flex-col gap-1">
			<h3 class="text-lg font-bold text-contrast">Server compatibility</h3>
			<p class="text-sm text-secondary">
				Select whether your server is vanilla or modded and which versions it supports. The Modrinth
				App uses this when a player joins.
			</p>
		</div>
		<ButtonStyled>
			<button @click="handleSetCompatibility">
				<PlayIcon />
				Set compatibility
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { PlayIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'

import ServerCompatibilityModal from './ServerCompatibilityModal/ServerCompatibilityModal.vue'

const serverCompatibilityModal = useTemplateRef<InstanceType<typeof ServerCompatibilityModal>>(
	'serverCompatibilityModal',
)

function handleSetCompatibility() {
	serverCompatibilityModal.value?.show()
}
</script>
