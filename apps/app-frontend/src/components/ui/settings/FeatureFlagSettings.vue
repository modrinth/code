<script setup lang="ts">
import { ButtonStyled, defineMessages, Toggle, useVIntl } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { useTheming } from '@/store/state'
import { DEFAULT_FEATURE_FLAGS, type FeatureFlag } from '@/store/theme.ts'

const themeStore = useTheming()
const { formatMessage } = useVIntl()

const settings = ref(await getSettings())
const options = ref<FeatureFlag[]>(Object.keys(DEFAULT_FEATURE_FLAGS))

const featureFlagMessages = defineMessages({
	project_background: {
		id: 'app.settings.feature-flags.project-background',
		defaultMessage: 'Project background',
	},
	page_path: {
		id: 'app.settings.feature-flags.page-path',
		defaultMessage: 'Page path',
	},
	worlds_tab: {
		id: 'app.settings.feature-flags.worlds-tab',
		defaultMessage: 'Worlds tab',
	},
	worlds_in_home: {
		id: 'app.settings.feature-flags.worlds-in-home',
		defaultMessage: 'Worlds on Home',
	},
	server_project_qa: {
		id: 'app.settings.feature-flags.server-project-qa',
		defaultMessage: 'Server project QA',
	},
	show_version_environment_column: {
		id: 'app.settings.feature-flags.show-version-environment-column',
		defaultMessage: 'Show version environment column',
	},
	server_ram_as_bytes_always_on: {
		id: 'app.settings.feature-flags.server-ram-as-bytes-always-on',
		defaultMessage: 'Always show server RAM in bytes',
	},
	always_show_app_controls: {
		id: 'app.settings.feature-flags.always-show-app-controls',
		defaultMessage: 'Always show app controls',
	},
	skip_non_essential_warnings: {
		id: 'app.settings.feature-flags.skip-non-essential-warnings',
		defaultMessage: 'Skip non-essential warnings',
	},
	skip_unknown_pack_warning: {
		id: 'app.settings.feature-flags.skip-unknown-pack-warning',
		defaultMessage: 'Skip unknown pack warning',
	},
	pride_fundraiser: {
		id: 'app.settings.feature-flags.pride-fundraiser',
		defaultMessage: 'Pride fundraiser',
	},
	i18n_debug: {
		id: 'app.settings.feature-flags.i18n-debug',
		defaultMessage: 'Internationalization debug',
	},
	show_instance_play_time: {
		id: 'app.settings.feature-flags.show-instance-play-time',
		defaultMessage: 'Show instance play time',
	},
	advanced_filters_collapsed: {
		id: 'app.settings.feature-flags.advanced-filters-collapsed',
		defaultMessage: 'Collapse advanced filters',
	},
})

const messages = defineMessages({
	resetToDefault: {
		id: 'app.settings.feature-flags.reset-to-default',
		defaultMessage: 'Reset to default',
	},
})

function setFeatureFlag(key: FeatureFlag, value: boolean) {
	themeStore.featureFlags[key] = value
	settings.value.feature_flags[key] = value
}

watch(
	settings,
	async () => {
		await setSettings(settings.value)
	},
	{ deep: true },
)
</script>
<template>
	<div class="flex flex-col gap-2.5 min-w-[600px]">
		<div v-for="option in options" :key="option" class="flex items-center justify-between">
			<div>
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(featureFlagMessages[option]) }}
				</h2>
			</div>
			<div class="flex items-center gap-2">
				<ButtonStyled type="transparent">
					<button
						:disabled="themeStore.getFeatureFlag(option) === DEFAULT_FEATURE_FLAGS[option]"
						@click="setFeatureFlag(option, DEFAULT_FEATURE_FLAGS[option])"
					>
						{{ formatMessage(messages.resetToDefault) }}
					</button>
				</ButtonStyled>
				<Toggle
					:id="`feature-flag-${option}`"
					:model-value="themeStore.getFeatureFlag(option)"
					@update:model-value="() => setFeatureFlag(option, !themeStore.getFeatureFlag(option))"
				/>
			</div>
		</div>
	</div>
</template>
