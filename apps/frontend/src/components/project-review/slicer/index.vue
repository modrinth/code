<template>
	<div v-if="frameSrc" class="relative flex h-full min-h-0 flex-col gap-2">
		<PageLoadingBar :key="target?.key" :loading="loadingFiles" />
		<div v-if="loadFailed && !loadingFiles" role="alert" class="flex items-center gap-2">
			<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
			<Button @click="retry">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<iframe
			:key="frameKey"
			ref="frame"
			:src="frameSrc"
			:title="formatMessage(messages.slicer)"
			:aria-busy="loadingFiles"
			allow="clipboard-read; clipboard-write"
			class="block min-h-0 w-full flex-1 rounded-xl border border-solid border-surface-4 bg-surface-1"
		/>
	</div>
	<p v-else-if="isLoading || versionsQuery.isPending.value" role="status" class="m-0">
		{{ formatMessage(messages.loading) }}
	</p>
	<p v-else class="m-0 text-secondary">{{ formatMessage(messages.emptyVersions) }}</p>
</template>

<script setup lang="ts">
import { Button, PageLoadingBar, useVIntl } from '@modrinth/ui'
import { onScopeDispose, ref, useTemplateRef, watch } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSlicer, type SlicerTarget } from '~/providers/project-review/review-slicer'

import { injectProjectReviewContext, injectReviewPanelVisible } from '../layout/context'
import { projectReviewMessages as messages } from '../messages'
import { useReviewEventListener } from '../shortcuts'
import bridgeSource from './bridge.js?raw'
import {
	forgetInstalledBridge,
	readInstalledBridgeVersion,
	saveInstalledBridgeVersion,
	SLICER_BRIDGE_VERSION,
} from './bridge-storage'

const SLICER_ORIGIN = 'https://slicer.run'
const BRIDGE_SOURCE = 'modrinth-slicer-bridge'
const HANDSHAKE_TIMEOUT_MS = 15_000

const { formatMessage } = useVIntl()
const { isLoading, versionsQuery } = injectProjectReviewPageContext()
const { target, openRequests, loadingFiles } = injectReviewSlicer()
const { openTab } = injectProjectReviewContext()
const panelVisible = injectReviewPanelVisible()
const frame = useTemplateRef<HTMLIFrameElement>('frame')
const bridgeScript = `data:text/javascript;base64,${btoa(
	bridgeSource
		.replaceAll('__MODRINTH_PARENT_ORIGIN__', window.location.origin)
		.replaceAll('__MODRINTH_BRIDGE_VERSION__', String(SLICER_BRIDGE_VERSION)),
)}`

const frameKey = ref(0)
const frameSrc = ref('')
const loadFailed = ref(false)
/** Highest bridge version that has handshaked with the current iframe, 0 when none has. */
let bridgeVersion = 0
let sentKey: string | undefined
let requestId = 0
let handshakeTimer: ReturnType<typeof setTimeout> | undefined

function bridgeInstalled() {
	return (readInstalledBridgeVersion() ?? 0) >= SLICER_BRIDGE_VERSION
}

/**
 * Mounts a fresh slicer instance. All files are loaded through the bridge after it handshakes,
 * so replacing a target can cancel its downloads and serialize workspace updates.
 *
 * The bridge script is only passed along (which makes slicer prompt for an import) when it is not
 * known to be installed. Otherwise slicer loads its saved copies, and if no up to date copy
 * handshakes in time the install record is dropped and slicer is remounted with the script.
 */
function mount(next: SlicerTarget, withScript = !bridgeInstalled()) {
	clearTimeout(handshakeTimer)
	handshakeTimer = undefined
	bridgeVersion = 0
	sentKey = next.key
	requestId++
	loadingFiles.value = true
	loadFailed.value = false
	const params = new URLSearchParams()
	if (withScript) params.set('script', bridgeScript)
	frameSrc.value = `${SLICER_ORIGIN}/?${params}`
	frameKey.value++
	if (!withScript) {
		handshakeTimer = setTimeout(() => {
			handshakeTimer = undefined
			if (bridgeVersion >= SLICER_BRIDGE_VERSION) return
			forgetInstalledBridge()
			if (target.value) mount(target.value, true)
		}, HANDSHAKE_TIMEOUT_MS)
	}
}

function send(next: SlicerTarget) {
	const targetWindow = frame.value?.contentWindow
	if (!targetWindow) return
	sentKey = next.key
	loadingFiles.value = true
	loadFailed.value = false
	targetWindow.postMessage(
		{ source: BRIDGE_SOURCE, type: 'load', id: ++requestId, files: next.files, entry: next.entry },
		SLICER_ORIGIN,
	)
}

function retry() {
	const next = target.value
	if (!next) return
	if (bridgeVersion >= SLICER_BRIDGE_VERSION) send(next)
	else mount(next)
}

/**
 * Slicer starts every saved copy of the bridge. Only a current copy receives targets, since older
 * copies cannot cancel their downloads when the reviewer selects something else.
 * Reparenting the iframe reloads its document, so a bridge of the same version can handshake again.
 */
function onHandshake(version: number) {
	if (version < SLICER_BRIDGE_VERSION || version < bridgeVersion) return
	bridgeVersion = version
	clearTimeout(handshakeTimer)
	handshakeTimer = undefined
	saveInstalledBridgeVersion(version)
	const next = target.value
	if (next) send(next)
}

watch(
	[target, panelVisible],
	([next, visible]) => {
		if (!next || next.key === sentKey) return
		loadFailed.value = false
		if (bridgeVersion) send(next)
		else if (handshakeTimer === undefined && (frameSrc.value || visible)) mount(next)
	},
	{ immediate: true },
)

watch(openRequests, () => openTab('slicer'))

useReviewEventListener('message', (event: MessageEvent) => {
	if (event.origin !== SLICER_ORIGIN || event.source !== frame.value?.contentWindow) return
	const data = event.data
	if (data?.source !== BRIDGE_SOURCE) return
	if (data.type === 'ready') {
		onHandshake(typeof data.version === 'number' ? data.version : 0)
		return
	}
	if (
		data.version !== bridgeVersion ||
		data.version < SLICER_BRIDGE_VERSION ||
		data.id !== requestId ||
		sentKey !== target.value?.key
	) {
		return
	}
	if (data.type === 'loading') {
		loadingFiles.value = true
		loadFailed.value = false
	} else if (data.type === 'loaded' || data.type === 'error') {
		loadingFiles.value = false
		loadFailed.value = data.type === 'error'
	}
})

onScopeDispose(() => {
	clearTimeout(handshakeTimer)
	loadingFiles.value = false
})
</script>
