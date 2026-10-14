<template>
	<iframe
		v-if="frameSrc"
		:key="frameKey"
		ref="frame"
		:src="frameSrc"
		:title="formatMessage(messages.slicer)"
		allow="clipboard-read; clipboard-write"
		class="h-full w-full rounded-xl border border-solid border-surface-4 bg-surface-1"
	/>
	<p v-else-if="isLoading || versionsQuery.isPending.value" role="status" class="m-0">
		{{ formatMessage(messages.loading) }}
	</p>
	<p v-else class="m-0 text-secondary">{{ formatMessage(messages.emptyVersions) }}</p>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { onScopeDispose, ref, useTemplateRef, watch } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSlicer, type SlicerTarget } from '~/providers/project-review/review-slicer'

import { injectReviewPanelVisible } from '../layout/context'
import { projectReviewMessages as messages } from '../messages'
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
const { target } = injectReviewSlicer()
const panelVisible = injectReviewPanelVisible()
const frame = useTemplateRef<HTMLIFrameElement>('frame')
const bridgeScript = `data:text/javascript;base64,${btoa(
	bridgeSource
		.replaceAll('__MODRINTH_PARENT_ORIGIN__', window.location.origin)
		.replaceAll('__MODRINTH_BRIDGE_VERSION__', String(SLICER_BRIDGE_VERSION)),
)}`

const frameKey = ref(0)
const frameSrc = ref('')
let bridgeReady = false
let mountedWithScript = false
let sentKey: string | undefined
let handshakeTimer: ReturnType<typeof setTimeout> | undefined

function bridgeInstalled() {
	return (readInstalledBridgeVersion() ?? 0) >= SLICER_BRIDGE_VERSION
}

/**
 * Mounts a fresh slicer instance. Files are passed through slicer's `url` parameters so they load
 * even without the bridge; later targets are sent over the bridge once it handshakes.
 *
 * The bridge script is only passed along (which makes slicer prompt for an import) when it is not
 * known to be installed. Otherwise slicer loads its saved copy, and if that copy never handshakes
 * the install record is dropped and slicer is remounted with the script.
 */
function mount(next: SlicerTarget, withScript = !bridgeInstalled()) {
	clearTimeout(handshakeTimer)
	handshakeTimer = undefined
	bridgeReady = false
	mountedWithScript = withScript
	sentKey = next.key
	const params = new URLSearchParams(next.files.map(({ url }) => ['url', url]))
	if (withScript) params.set('script', bridgeScript)
	frameSrc.value = `${SLICER_ORIGIN}/?${params}`
	frameKey.value++
	if (!withScript) {
		handshakeTimer = setTimeout(() => {
			handshakeTimer = undefined
			forgetInstalledBridge()
			if (target.value) mount(target.value, true)
		}, HANDSHAKE_TIMEOUT_MS)
	}
}

function send(next: SlicerTarget) {
	const targetWindow = frame.value?.contentWindow
	if (!targetWindow) return
	sentKey = next.key
	targetWindow.postMessage(
		{ source: BRIDGE_SOURCE, type: 'load', id: next.key, files: next.files },
		SLICER_ORIGIN,
	)
}

function onHandshake(version: number) {
	clearTimeout(handshakeTimer)
	handshakeTimer = undefined
	if (version >= SLICER_BRIDGE_VERSION) {
		saveInstalledBridgeVersion(version)
	} else if (!mountedWithScript && target.value) {
		mount(target.value, true)
		return
	}
	bridgeReady = true
	if (target.value && target.value.key !== sentKey) send(target.value)
}

watch(
	[target, panelVisible],
	([next, visible]) => {
		if (!next || next.key === sentKey) return
		if (bridgeReady) send(next)
		else if (handshakeTimer === undefined && (frameSrc.value || visible)) mount(next)
	},
	{ immediate: true },
)

useEventListener('message', (event: MessageEvent) => {
	if (event.origin !== SLICER_ORIGIN || event.source !== frame.value?.contentWindow) return
	const data = event.data
	if (data?.source !== BRIDGE_SOURCE || data.type !== 'ready') return
	onHandshake(typeof data.version === 'number' ? data.version : 0)
})

onScopeDispose(() => clearTimeout(handshakeTimer))
</script>
