<script setup lang="ts">
import { RefreshCwIcon, Settings2Icon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	injectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import { open_ads_consent_preferences } from '@/helpers/ads.js'
import { get, reset_analytics_installation_id, set } from '@/helpers/settings.ts'

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
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
	telemetryTitle: {
		id: 'app.settings.privacy.telemetry.title',
		defaultMessage: 'Telemetry',
	},
	enableTelemetryTitle: {
		id: 'app.settings.privacy.telemetry.enable.title',
		defaultMessage: 'Enable telemetry',
	},
	telemetryDescription: {
		id: 'app.settings.privacy.telemetry.description',
		defaultMessage:
			"Modrinth collects analytics and usage data, such as which features are used, to help us improve the app. This data isn't linked to your name, email, or Modrinth account. By disabling this option, you opt out and your data will no longer be collected.",
	},
	installationIdTitle: {
		id: 'app.settings.privacy.telemetry.installation-id.title',
		defaultMessage: 'Installation ID',
	},
	installationIdDescription: {
		id: 'app.settings.privacy.telemetry.installation-id.description',
		defaultMessage:
			"The installation ID is a randomly generated identifier for this copy of the Modrinth app. It isn't linked to your name, email, or account.",
	},
	resetInstallationId: {
		id: 'app.settings.privacy.telemetry.reset-installation-id',
		defaultMessage: 'Reset installation ID',
	},
	resetInstallationIdSuccessTitle: {
		id: 'app.settings.privacy.telemetry.reset-installation-id.success.title',
		defaultMessage: 'Installation ID reset',
	},
	discordRichPresenceTitle: {
		id: 'app.settings.privacy.discord-rich-presence.title',
		defaultMessage: 'Discord Rich Presence',
	},
	discordRichPresenceDescription: {
		id: 'app.settings.privacy.discord-rich-presence.description',
		defaultMessage:
			'Show Modrinth App as your current activity on Discord. This does not affect Rich Presence added to instances by mods. Requires an app restart.',
	},
})

async function manageAdsPreferences() {
	await open_ads_consent_preferences().catch(handleError)
}

const resettingInstallationId = ref(false)

async function resetInstallationId() {
	resettingInstallationId.value = true
	try {
		await reset_analytics_installation_id()
		addNotification({
			type: 'success',
			title: formatMessage(messages.resetInstallationIdSuccessTitle),
		})
	} catch (error) {
		handleError(error as Error)
	} finally {
		resettingInstallationId.value = false
	}
}

watch(
	settings,
	async () => {
		try {
			await set(settings.value)
		} catch (error) {
			handleError(error as Error)
		}
	},
	{ deep: true },
)
</script>

<template>
	<div class="flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.discordRichPresenceTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.discordRichPresenceDescription) }}
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>

	<div v-if="adConsentAvailable" class="mt-6">
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.adsConsentTitle) }}
		</h2>
		<div class="mt-2 flex flex-col gap-2.5 items-start">
			<Button @click="manageAdsPreferences">
				<Settings2Icon aria-hidden="true" />
				{{ formatMessage(messages.adsConsentManage) }}
			</Button>
			<div>
				{{ formatMessage(messages.adsConsentIntro) }}
			</div>
		</div>
	</div>

	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.telemetryTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 id="enable-telemetry-label" class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.enableTelemetryTitle) }}
					</h3>
					<p id="enable-telemetry-description" class="m-0 mt-1">
						{{ formatMessage(messages.telemetryDescription) }}
					</p>
				</div>
				<Toggle
					id="opt-out-analytics"
					v-model="settings.telemetry"
					aria-labelledby="enable-telemetry-label"
					aria-describedby="enable-telemetry-description"
				/>
			</div>

			<div class="select-none">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.installationIdTitle) }}
				</h3>
				<p class="m-0 mt-1">
					{{ formatMessage(messages.installationIdDescription) }}
				</p>
				<Button
					class="mt-2.5"
					:disabled="!settings.telemetry || resettingInstallationId"
					@click="resetInstallationId"
				>
					<RefreshCwIcon aria-hidden="true" />
					{{ formatMessage(messages.resetInstallationId) }}
				</Button>
			</div>
		</div>
	</section>
</template>
