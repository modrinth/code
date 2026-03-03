<template>
	<CreationFlowModal
		ref="creationFlowRef"
		type="reset-server"
		:available-loaders="serverLoaders"
		:show-snapshot-toggle="true"
		:disable-close="props.initialSetup"
		:is-initial-setup="props.initialSetup"
		:initial-loader="initialLoader"
		:initial-game-version="initialGameVersion"
		@create="onFlowComplete"
		@hide="$emit('hide')"
		@browse-modpacks="$emit('browse-modpacks')"
	/>

	<NewModal ref="uploadModal" header="Uploading modpack" :closable="false">
		<div class="flex flex-col gap-4 md:w-[400px]">
			<AppearingProgressBar :max-value="totalBytes" :current-value="uploadedBytes" />
			<p class="m-0 text-sm text-secondary">Please don't close this page while uploading.</p>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon, ModrinthApiError } from '@modrinth/api-client'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import { injectModrinthClient } from '../../providers/api-client'
import { injectModrinthServerContext } from '../../providers/server-context'
import { injectNotificationManager } from '../../providers/web-notifications'
import { AppearingProgressBar } from '../base'
import type { CreationFlowContextValue } from '../flows/creation-flow-modal/creation-flow-context'
import CreationFlowModal from '../flows/creation-flow-modal/index.vue'
import { NewModal } from '../modal'

const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()

const serverLoaders = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

const props = defineProps<{
	initialSetup?: boolean
}>()

const emit = defineEmits<{
	reinstall: [args?: { loader: string; lVersion: string; mVersion: string | null }]
	hide: []
	'browse-modpacks': []
}>()

const initialLoader = computed(() => {
	const loader = serverContext.server.value.loader
	if (!loader || loader === 'Vanilla') return undefined
	return loader.toLowerCase()
})

const initialGameVersion = computed(() => serverContext.server.value.mc_version ?? undefined)

const creationFlowRef = useTemplateRef<InstanceType<typeof CreationFlowModal>>('creationFlowRef')
const uploadModal = useTemplateRef<InstanceType<typeof NewModal>>('uploadModal')

const uploadedBytes = ref(0)
const totalBytes = ref(0)

async function onFlowComplete(ctx: CreationFlowContextValue) {
	try {
		if (ctx.setupType.value === 'modpack' && ctx.modpackFile.value) {
			await handleMrpackUpload(ctx.modpackFile.value)
		} else if (ctx.setupType.value === 'modpack' && ctx.modpackSelection.value) {
			await client.archon.content_v1.installContent(
				serverContext.serverId,
				serverContext.worldId.value!,
				{
					content_variant: 'modpack',
					spec: {
						platform: 'modrinth',
						project_id: ctx.modpackSelection.value.projectId,
						version_id: ctx.modpackSelection.value.versionId,
					},
					soft_override: false,
					properties: ctx.buildProperties(),
				},
			)

			emitReinstall()
		} else {
			const loader = ctx.selectedLoader.value
			const loaderVersion =
				!loader || loader === 'vanilla' ? '' : (ctx.selectedLoaderVersion.value ?? '')

			await client.archon.content_v1.installContent(
				serverContext.serverId,
				serverContext.worldId.value!,
				{
					content_variant: 'bare',
					loader: toApiLoader(loader ?? 'vanilla'),
					version: loaderVersion,
					game_version: ctx.selectedGameVersion.value ?? undefined,
					soft_override: false,
					properties: ctx.buildProperties(),
				},
			)

			emitReinstall({
				loader: loader ?? 'vanilla',
				lVersion: loaderVersion,
				mVersion: ctx.selectedGameVersion.value,
			})
		}

		creationFlowRef.value?.hide()
	} catch (error) {
		if ((error as ModrinthApiError).statusCode === 429) {
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
	} finally {
		ctx.loading.value = false
	}
}

async function handleMrpackUpload(file: File) {
	uploadedBytes.value = 0
	totalBytes.value = file.size

	creationFlowRef.value?.hide()
	await nextTick()
	uploadModal.value?.show()

	try {
		const handle = await client.archon.servers_v0.reinstallFromMrpack(
			serverContext.serverId,
			file,
			true,
		)

		handle.onProgress(({ loaded, total }) => {
			uploadedBytes.value = loaded
			totalBytes.value = total
		})

		await handle.promise
		emitReinstall()
	} finally {
		uploadModal.value?.hide()
	}
}

function emitReinstall(args?: { loader: string; lVersion: string; mVersion: string | null }) {
	emit('reinstall', args)
}

function show() {
	creationFlowRef.value?.show()
}

function hide() {
	creationFlowRef.value?.hide()
}

defineExpose({ show, hide, ctx: computed(() => creationFlowRef.value?.ctx) })
</script>
