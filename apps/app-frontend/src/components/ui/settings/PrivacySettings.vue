<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { optInAnalytics, optOutAnalytics } from '@/helpers/analytics'
import { get, set } from '@/helpers/settings.ts'

const settings = ref(await get())

watch(
	settings,
	async () => {
		if (settings.value.telemetry) {
			optInAnalytics()
		} else {
			optOutAnalytics()
		}

		await set(settings.value)
	},
	{ deep: true },
)
</script>

<template>
	<div class="flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">Personalized ads</h2>
			<p class="m-0 text-sm">
				Modrinth's ad provider, Aditude, shows ads based on your preferences. By disabling this
				option, you opt out and ads will no longer be shown based on your interests.
			</p>
		</div>
		<Toggle id="personalized-ads" v-model="settings.personalized_ads" />
	</div>

	<div class="mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">Telemetry</h2>
			<p class="m-0 text-sm">
				Modrinth collects anonymized analytics and usage data to improve our user experience and
				customize your experience. By disabling this option, you opt out and your data will no
				longer be collected.
			</p>
		</div>
		<Toggle id="opt-out-analytics" v-model="settings.telemetry" />
	</div>

	<div class="mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">Discord RPC</h2>
			<p class="m-0 text-sm">
				Manages the Discord Rich Presence integration. Disabling this will cause 'Modrinth' to no
				longer show up as a game or app you are using on your Discord profile.
			</p>
			<p class="m-0 mt-2 text-sm">
				Note: This will not prevent any instance-specific Discord Rich Presence integrations, such
				as those added by mods. (app restart required to take effect)
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>
