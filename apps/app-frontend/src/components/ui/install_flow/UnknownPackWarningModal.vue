<template>
	<UnknownFileWarningModal
		ref="modal"
		:mode="externalFilesInModpack.length > 0 ? 'modpack' : 'mod'"
		:file-name="fileName"
		:external-files-in-modpack="externalFilesInModpack"
		:offer-sandbox="offerSandbox"
		@cancel="reset"
		@continue="proceed"
		@sandbox="proceedSandboxed"
	/>
</template>

<script setup lang="ts">
import { injectNotificationManager, UnknownFileWarningModal } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { enableInstallJobSandbox } from '@/features/sandbox/enable-instance-sandbox'
import { useOfferSandbox } from '@/features/sandbox/use-offer-sandbox'
import type { InstallJobSnapshot } from '@/helpers/install'
import { get as getSettings, set as setSettings } from '@/helpers/settings'

type CreateInstance = () => Promise<InstallJobSnapshot | void>

const appSettings = useAppSettings()
const offerSandbox = useOfferSandbox()
const { handleError } = injectNotificationManager()

const modal = useTemplateRef('modal')
const onProceed = ref<CreateInstance>()
const fileName = ref('')
const externalFilesInModpack = ref<string[]>([])

function show(
	createInstance: CreateInstance,
	selectedFileName = '',
	selectedExternalFiles: string[] = [],
) {
	onProceed.value = createInstance
	fileName.value = selectedFileName
	externalFilesInModpack.value = selectedExternalFiles

	if (!appSettings.warnOnUnknownModpacks) {
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
		appSettings.warnOnUnknownModpacks = false
		const settings = await getSettings()
		settings.warn_on_unknown_modpacks = false
		await setSettings(settings)
	}

	const createInstance = onProceed.value
	reset()
	if (createInstance) void createInstance()
}

async function proceedSandboxed() {
	const createInstance = onProceed.value
	reset()
	const job = await createInstance?.()
	if (job) await enableInstallJobSandbox(job).catch(handleError)
}

defineExpose({ show })
</script>
