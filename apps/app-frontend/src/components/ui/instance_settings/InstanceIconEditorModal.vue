<script setup lang="ts">
import type { IconConfig } from '@modrinth/api-client'
import { getIconSymbolAsset, IconEditorModal, injectNotificationManager } from '@modrinth/ui'
import { ref } from 'vue'

import { toError } from '@/helpers/errors'
import {
	cache_generated_icon,
	edit_generated_icon,
	edit_generated_icon_if_empty,
	get_recent_icon_configs,
} from '@/helpers/instance'

const props = defineProps<{
	instanceId?: string
	config?: IconConfig | null
}>()

const emit = defineEmits<{
	saved: [iconPath: string, config: IconConfig]
}>()

const { handleError } = injectNotificationManager()
const editor = ref<InstanceType<typeof IconEditorModal> | null>(null)

async function loadSymbolBytes(asset: string): Promise<number[]> {
	const response = await fetch(asset)
	if (!response.ok) throw new Error('Failed to load the icon symbol.')
	return Array.from(new Uint8Array(await response.arrayBuffer()))
}

async function save(config: IconConfig, asset: string): Promise<string> {
	const symbolBytes = await loadSymbolBytes(asset)
	return props.instanceId
		? edit_generated_icon(props.instanceId, config, symbolBytes)
		: cache_generated_icon(config, symbolBytes, true)
}

function onSaved(result: string | void, config: IconConfig) {
	if (result) emit('saved', result, config)
}

function show() {
	editor.value?.show()
}

function hide() {
	editor.value?.hide()
}

async function randomizeAndSave() {
	try {
		const selection = editor.value?.randomize()
		if (!selection) return null
		const iconPath = await cache_generated_icon(
			selection.config,
			await loadSymbolBytes(selection.symbolAsset),
		)
		return { iconPath, config: selection.config }
	} catch (error) {
		handleError(toError(error))
		return null
	}
}

async function applyGeneratedIcon(instanceId: string, config: IconConfig) {
	const asset = getIconSymbolAsset(config.symbol)
	if (!asset) return false
	try {
		return await edit_generated_icon_if_empty(instanceId, config, await loadSymbolBytes(asset))
	} catch (error) {
		handleError(toError(error))
		return false
	}
}

defineExpose({ show, hide, randomize: randomizeAndSave, randomizeAndSave, applyGeneratedIcon })
</script>

<template>
	<IconEditorModal
		ref="editor"
		:config="config"
		:load-recents="get_recent_icon_configs"
		:save="save"
		@saved="onSaved"
	/>
</template>
