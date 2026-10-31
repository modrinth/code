<script setup lang="ts">
import {
	defineMessage,
	defineMessages,
	injectNotificationManager,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, useId, watch } from 'vue'

import SandboxSettingsForm from '@/features/settings/instance-defaults/sandbox-settings-form.vue'
import { edit } from '@/helpers/instance'
import { type AppSettings, get } from '@/helpers/settings.ts'
import type { SandboxSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const id = useId()

const { instance } = injectInstanceSettings()

const globalSettings = (await get().catch(handleError)) as AppSettings

function copySettings(value: SandboxSettings): SandboxSettings {
	return { ...value, folders: [...value.folders] }
}

const customSettings = ref(!!instance.value.sandbox)
const sandbox = ref(copySettings(instance.value.sandbox ?? globalSettings.sandbox))

watch(customSettings, (enabled) => {
	if (!enabled) {
		sandbox.value = copySettings(globalSettings.sandbox)
	}
})

watch(
	[customSettings, sandbox],
	async () => {
		await edit(instance.value.id, {
			sandbox: customSettings.value ? copySettings(sandbox.value) : null,
		}).catch(handleError)
	},
	{ deep: true },
)

const description = defineMessage({
	id: 'instance.settings.tabs.sandbox.default-settings-description',
	defaultMessage:
		'Your instance uses the default sandbox permissions unless you override them here.',
})
const messages = defineMessages({
	customDescription: {
		id: 'app.settings.sandbox.custom-description',
		defaultMessage: 'Override app sandbox settings for this instance.',
	},
	custom: {
		id: 'app.settings.sandbox.custom',
		defaultMessage: 'Custom sandbox settings',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro tab="sandbox" :description="formatMessage(description)" />
		<div
			class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
		>
			<div class="flex min-w-0 flex-col gap-0.5">
				<label :for="`${id}-custom`" class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.custom) }}
				</label>
				<p :id="`${id}-custom-description`" class="m-0">
					{{ formatMessage(messages.customDescription) }}
				</p>
			</div>
			<Toggle
				:id="`${id}-custom`"
				v-model="customSettings"
				:aria-describedby="`${id}-custom-description`"
			/>
		</div>
		<SandboxSettingsForm v-model="sandbox" instance-settings :disabled="!customSettings" />
	</div>
</template>
