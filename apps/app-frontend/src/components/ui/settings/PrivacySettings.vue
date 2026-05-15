<script setup lang="ts">
import { Toggle } from '@icarus/ui'
import { ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'

const settings = ref(await get())
</script>

<template>
	<div>
		<div class="flex items-center justify-between gap-4">
			<div>
				<h2 class="m-0 text-lg font-semibold text-contrast">Discord RPC</h2>
				<p class="m-0 mt-1 text-sm">
					Disabling this will cause 'Icarus' to no longer show up as a game or app you are using on your Discord profile.
				</p>
				<p class="m-0 mt-2 text-sm">
					Note: This will not prevent any instance-specific Discord Rich Presence integrations, such
					as those added by mods. (app restart required to take effect)
				</p>
			</div>
			<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
		</div>
	</div>
</template>
