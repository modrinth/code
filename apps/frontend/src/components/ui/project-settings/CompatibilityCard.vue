<template>
	<div class="flex items-start justify-between gap-4">
		<div class="flex flex-col gap-1">
			<div class="text-xl font-semibold text-contrast">Server compatibility</div>
			<div class="text-sm text-secondary">
				Select whether your server is vanilla or modded and which versions it supports. The Modrinth
				App uses this when a player joins.
			</div>
		</div>
		<ButtonStyled>
			<button @click="handleSetCompatibility">
				<PlayIcon />
				Set compatibility
			</button>
		</ButtonStyled>
		<ServerCompatibilityModal ref="serverCompatibilityModal" />
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
