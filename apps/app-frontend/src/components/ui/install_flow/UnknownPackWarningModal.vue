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

import { get as getSettings, set as setSettings } from '@/helpers/settings'
import { useTheming } from '@/store/state'
import type { FeatureFlag } from '@/store/theme.ts'

const themeStore = useTheming()
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

	if (themeStore.getFeatureFlag(skipUnknownPackWarningFeatureFlag)) {
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
		themeStore.featureFlags[skipUnknownPackWarningFeatureFlag] = true
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
