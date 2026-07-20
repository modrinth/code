<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	injectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import { open_ads_consent_preferences } from '@/helpers/ads.js'
import { optInAnalytics, optOutAnalytics } from '@/helpers/analytics'
import { get, set } from '@/helpers/settings.ts'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { adConsentAvailable } = injectPageContext()
const settings = ref(await get())

const messages = defineMessages({
	adsConsentTitle: {
		id: 'app.ads-consent.title',
		defaultMessage: 'Your privacy and how ads support Modrinth',
	},
	adsConsentIntro: {
		id: 'app.settings.privacy.ads-consent.intro',
		defaultMessage:
			'Ads make Modrinth possible and fund creator payouts. Our partners may store or access cookies in the app to personalize ads and measure performance. You can opt out or manage your preferences below.',
	},
	adsConsentManage: {
		id: 'app.ads-consent.manage',
		defaultMessage: 'Manage preferences',
	},
})

async function manageAdsPreferences() {
	await open_ads_consent_preferences().catch(handleError)
}

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
	<div v-if="adConsentAvailable">
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.adsConsentTitle) }}
		</h2>
		<div class="mt-1 flex flex-col gap-2.5 items-start">
			<div class="flex flex-col gap-1 items-start">
				<div class="text-sm">
					{{ formatMessage(messages.adsConsentIntro) }}
				</div>
			</div>
			<ButtonStyled>
				<button class="!shadow-none" @click="manageAdsPreferences">
					<Settings2Icon aria-hidden="true" />
					{{ formatMessage(messages.adsConsentManage) }}
				</button>
			</ButtonStyled>
		</div>
	</div>

	<div class="mt-8 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Telemetry</h2>
			<p class="m-0 mt-1 text-sm">
				Modrinth collects anonymized analytics and usage data to improve our user experience and
				customize your experience. By disabling this option, you opt out and your data will no
				longer be collected.
			</p>
		</div>
		<Toggle id="opt-out-analytics" v-model="settings.telemetry" />
	</div>

	<div class="mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Discord RPC</h2>
			<p class="m-0 mt-1 text-sm">
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
