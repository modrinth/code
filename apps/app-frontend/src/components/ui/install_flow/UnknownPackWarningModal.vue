<template>
	<UnknownFileWarningModal
		ref="modal"
		:mode="externalFilesInModpack.length > 0 ? 'modpack' : 'mod'"
		:file-name="fileName"
		:external-files-in-modpack="externalFilesInModpack"
		@cancel="reset"
		@continue="proceed"
	/>
</template>

<script setup lang="ts">
import { UnknownFileWarningModal } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { type FeatureFlag, useAppSettings } from '@/composables/use-app-settings.ts'
import { get as getSettings, set as setSettings } from '@/helpers/settings'

const appSettings = useAppSettings()
const skipUnknownPackWarningFeatureFlag = 'skip_unknown_pack_warning' as FeatureFlag

const modal = useTemplateRef('modal')
const onProceed = ref<() => Promise<void>>()
const fileName = ref('')
const externalFilesInModpack = ref<string[]>([])

function show(
	createInstance: () => Promise<void>,
	selectedFileName = '',
	selectedExternalFiles: string[] = [],
) {
	onProceed.value = createInstance
	fileName.value = selectedFileName
	externalFilesInModpack.value = selectedExternalFiles

	if (appSettings.getFeatureFlag(skipUnknownPackWarningFeatureFlag)) {
		void createInstance()
		return
	}

	modal.value?.show()
}

function reset() {
	onProceed.value = undefined
	fileName.value = ''
	externalFilesInModpack.value = []
}

async function proceed(dontShowAgain: boolean) {
	if (dontShowAgain) {
		appSettings.featureFlags[skipUnknownPackWarningFeatureFlag] = true
		const settings = await getSettings()
		settings.feature_flags[skipUnknownPackWarningFeatureFlag] = true
		await setSettings(settings)
	}

	const createInstance = onProceed.value
	reset()
	if (createInstance) void createInstance()
}

defineExpose({ show })
</script>
