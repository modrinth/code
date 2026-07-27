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
			'Control how advertising partners use cookies to personalize ads and measure performance. Ads fund Modrinth and creator payouts.',
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
		<div class="mt-2 flex flex-col gap-2.5 items-start">
			<ButtonStyled>
				<button class="!shadow-none" @click="manageAdsPreferences">
					<Settings2Icon aria-hidden="true" />
					{{ formatMessage(messages.adsConsentManage) }}
				</button>
			</ButtonStyled>
			<p class="m-0 text-sm">
				{{ formatMessage(messages.adsConsentIntro) }}
			</p>
		</div>
	</div>

	<div class="mt-8 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Share analytics</h2>
			<p class="m-0 mt-1 text-sm">
				Share anonymous usage and analytics data to help improve Modrinth App.
			</p>
		</div>
		<Toggle id="opt-out-analytics" v-model="settings.telemetry" />
	</div>

	<div class="mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Discord activity</h2>
			<p class="m-0 mt-1 text-sm">
				Show Modrinth App as your current activity on Discord. This does not affect Discord activity
				added to instances by mods. Requires an app restart.
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>
