<template>
	<CreationFlowModal
		ref="creationFlowRef"
		type="server-onboarding"
		:available-loaders="serverLoaders"
		:show-snapshot-toggle="true"
		:disable-close="props.initialSetup"
		:is-initial-setup="props.initialSetup"
		@create="onFlowComplete"
		@hide="$emit('hide')"
		@browse-modpacks="navigateToModpacks"
	/>

	<NewModal ref="uploadModal" header="Uploading modpack" :closable="false">
		<div class="flex flex-col gap-4 md:w-[400px]">
			<AppearingProgressBar :max-value="totalBytes" :current-value="uploadedBytes" />
			<p class="m-0 text-sm text-secondary">Please don't close this page while uploading.</p>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { CreationFlowContextValue } from '@modrinth/ui'
import {
	AppearingProgressBar,
	CreationFlowModal,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { ModrinthServersFetchError } from '@modrinth/utils'
import { nextTick, ref, useTemplateRef } from 'vue'

import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'

const { addNotification } = injectNotificationManager()

const serverLoaders = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

const loaderApiNames: Record<string, string> = {
	fabric: 'Fabric',
	neoforge: 'NeoForge',
	forge: 'Forge',
	quilt: 'Quilt',
	paper: 'Paper',
	purpur: 'Purpur',
	vanilla: 'Vanilla',
}

const props = defineProps<{
	server: ModrinthServer
	initialSetup?: boolean
}>()

const emit = defineEmits<{
	reinstall: [any?]
	hide: []
}>()

const creationFlowRef = useTemplateRef<InstanceType<typeof CreationFlowModal>>('creationFlowRef')
const uploadModal = useTemplateRef<InstanceType<typeof NewModal>>('uploadModal')

const uploadedBytes = ref(0)
const totalBytes = ref(0)

async function onFlowComplete(ctx: CreationFlowContextValue) {
	const hardReset = props.initialSetup ? true : ctx.hardReset.value

	try {
		if (ctx.worldType.value === 'modpack' && ctx.modpackFile.value) {
			// .mrpack file upload
			await handleMrpackUpload(ctx.modpackFile.value, hardReset)
		} else if (ctx.worldType.value === 'modpack' && ctx.modpackSelection.value) {
			// Modpack from search
			await props.server.general.reinstall(
				false,
				ctx.modpackSelection.value.projectId,
				ctx.modpackSelection.value.versionId,
				undefined,
				hardReset,
			)

			emitReinstall()
		} else {
			// Custom/Vanilla loader
			const loader = ctx.selectedLoader.value
			const apiLoaderName = loader
				? (loaderApiNames[loader] ?? loader.charAt(0).toUpperCase() + loader.slice(1))
				: 'Vanilla'
			const loaderVersion =
				apiLoaderName === 'Vanilla' ? '' : (ctx.selectedLoaderVersion.value ?? '')

			await props.server.general.reinstall(
				true,
				apiLoaderName,
				ctx.selectedGameVersion.value ?? '',
				loaderVersion,
				hardReset,
			)

			emitReinstall({
				loader: apiLoaderName,
				lVersion: loaderVersion,
				mVersion: ctx.selectedGameVersion.value,
			})
		}

		creationFlowRef.value?.hide()
	} catch (error) {
		if (error instanceof ModrinthServersFetchError && error.statusCode === 429) {
			addNotification({
				title: 'Cannot reinstall server',
				text: 'You are being rate limited. Please try again later.',
				type: 'error',
			})
		} else {
			addNotification({
				title: 'Reinstall Failed',
				text: 'An unexpected error occurred while reinstalling. Please try again later.',
				type: 'error',
			})
		}
	}
}

async function handleMrpackUpload(file: File, hardReset: boolean) {
	uploadedBytes.value = 0
	totalBytes.value = file.size

	creationFlowRef.value?.hide()
	await nextTick()
	uploadModal.value?.show()

	const { onProgress, promise } = props.server.general.reinstallFromMrpack(file, hardReset)

	onProgress(({ loaded, total }) => {
		uploadedBytes.value = loaded
		totalBytes.value = total
	})

	try {
		await promise
		emitReinstall()
	} finally {
		uploadModal.value?.hide()
	}
}

function emitReinstall(args?: { loader: string; lVersion: string; mVersion: string | null }) {
	emit('reinstall', args)
}

function navigateToModpacks() {
	creationFlowRef.value?.hide()
	navigateTo(`/discover/modpacks?sid=${props.server.serverId}`)
}

function show() {
	creationFlowRef.value?.show()
}

function hide() {
	creationFlowRef.value?.hide()
}

defineExpose({ show, hide, ctx: computed(() => creationFlowRef.value?.ctx) })
</script>
