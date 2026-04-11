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
		:fade="props.initialSetup ? undefined : 'danger'"
		:search-modpacks="searchModpacks"
		:get-project-versions="getProjectVersions"
		@create="onFlowComplete"
		@hide="$emit('hide')"
		@browse-modpacks="$emit('browse-modpacks')"
	/>

	<UploadProgressModal ref="uploadProgressModal" />
</template>

<script setup lang="ts">
import type { Archon, ModrinthApiError } from '@modrinth/api-client'
import { computed, useTemplateRef } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers/api-client'
import { injectModrinthServerContext } from '../../providers/server-context'
import { injectNotificationManager } from '../../providers/web-notifications'
import type { CreationFlowContextValue } from '../flows/creation-flow-modal/creation-flow-context'
import CreationFlowModal from '../flows/creation-flow-modal/index.vue'
import { UploadProgressModal } from '../modal'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	rateLimitTitle: {
		id: 'servers.setup.rate-limit.title',
		defaultMessage: 'Cannot reinstall server',
	},
	rateLimitText: {
		id: 'servers.setup.rate-limit.text',
		defaultMessage: 'You are being rate limited. Please try again later.',
	},
	reinstallFailedTitle: {
		id: 'servers.setup.reinstall-failed.title',
		defaultMessage: 'Reinstall Failed',
	},
	reinstallFailedText: {
		id: 'servers.setup.reinstall-failed.text',
		defaultMessage: 'An unexpected error occurred while reinstalling. Please try again later.',
	},
})

const debug = useDebugLogger('ServerSetupModal')
const client = injectModrinthClient()
const serverContext = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()

const serverLoaders = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

async function searchModpacks(query: string, limit: number = 10) {
	return client.labrinth.projects_v2.search({
		query: query || undefined,
		new_filters:
			'project_types = "modpack" AND (client_side = "optional" OR client_side = "required") AND server_side = "required"',
		limit,
	})
}

async function getProjectVersions(projectId: string) {
	const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
	return versions.map((v) => ({ id: v.id }))
}

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
const uploadProgressModal =
	useTemplateRef<InstanceType<typeof UploadProgressModal>>('uploadProgressModal')

async function onFlowComplete(ctx: CreationFlowContextValue) {
	debug('onFlowComplete:', {
		setupType: ctx.setupType.value,
		hasModpackFile: !!ctx.modpackFile.value,
		modpackSelection: ctx.modpackSelection.value,
		selectedLoader: ctx.selectedLoader.value,
		selectedGameVersion: ctx.selectedGameVersion.value,
		selectedLoaderVersion: ctx.selectedLoaderVersion.value,
		worldId: serverContext.worldId.value,
	})

	try {
		if (ctx.setupType.value === 'modpack' && ctx.modpackFile.value) {
			debug('onFlowComplete: mrpack upload path')
			await handleMrpackUpload(ctx.modpackFile.value, ctx.buildProperties())
		} else if (ctx.setupType.value === 'modpack' && ctx.modpackSelection.value) {
			debug('onFlowComplete: modpack selection path, calling installContent')
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
			debug('onFlowComplete: modpack installContent returned, emitting reinstall')
			emitReinstall()
		} else {
			const loader = ctx.selectedLoader.value
			const loaderVersion =
				!loader || loader === 'vanilla' ? '' : (ctx.selectedLoaderVersion.value ?? '')

			debug('onFlowComplete: bare install path', {
				loader,
				loaderVersion,
				gameVersion: ctx.selectedGameVersion.value,
				apiLoader: toApiLoader(loader ?? 'vanilla'),
			})

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

			debug('onFlowComplete: bare installContent returned, emitting reinstall')
			emitReinstall({
				loader: loader ?? 'vanilla',
				lVersion: loaderVersion,
				mVersion: ctx.selectedGameVersion.value,
			})
		}

		creationFlowRef.value?.hide()
	} catch (error) {
		debug('onFlowComplete: ERROR', error)
		if ((error as ModrinthApiError).statusCode === 429) {
			addNotification({
				title: formatMessage(messages.rateLimitTitle),
				text: formatMessage(messages.rateLimitText),
				type: 'error',
			})
		} else {
			addNotification({
				title: formatMessage(messages.reinstallFailedTitle),
				text: formatMessage(messages.reinstallFailedText),
				type: 'error',
			})
		}
	} finally {
		ctx.loading.value = false
	}
}

async function handleMrpackUpload(file: File, properties: Archon.Content.v1.PropertiesFields) {
	creationFlowRef.value?.hide()
	const handle = client.kyros.content_v1.uploadModpackFile(
		serverContext.worldId.value!,
		file,
		properties,
		{ softOverride: false },
	)
	await uploadProgressModal.value!.track(handle)
	emitReinstall()
}

function emitReinstall(args?: { loader: string; lVersion: string; mVersion: string | null }) {
	debug('emitReinstall:', args)
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
