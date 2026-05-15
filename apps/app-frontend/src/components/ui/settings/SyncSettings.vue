<script setup lang="ts">
import { Checkbox } from '@icarus/ui'
import { defineMessage, useVIntl } from '@icarus/ui'
import { ref, watch } from 'vue'
import { get, set } from '@/helpers/settings'
import type { AppSettings } from '@/helpers/settings'

const { formatMessage } = useVIntl()
const localSettings = ref<AppSettings>(await get())

const labels: Record<string, string> = {
	saves: 'Worlds',
	screenshots: 'Screenshots',
	resourcepacks: 'Resourcepacks',
	shaderpacks: 'Shaderpacks',
	schematics: 'Litematica Schematics',
	'options.txt': 'Game Options',
	'servers.dat': 'Server List',
}

const allFolderTargets = ['saves', 'screenshots', 'resourcepacks', 'shaderpacks', 'schematics']
const allFileTargets = ['options.txt', 'servers.dat']

function isEnabled(type: 'folder' | 'file', key: string) {
	return type === 'folder'
		? localSettings.value.sync.folders.includes(key)
		: localSettings.value.sync.files.includes(key)
}

function setEnabled(type: 'folder' | 'file', key: string, enabled: boolean) {
	const list = type === 'folder' ? localSettings.value.sync.folders : localSettings.value.sync.files
	if (enabled && !list.includes(key)) list.push(key)
	if (!enabled) {
		const filtered = list.filter((x) => x !== key)
		if (type === 'folder') localSettings.value.sync.folders = filtered
		else localSettings.value.sync.files = filtered
	}
}

watch(
	localSettings,
	async (newVal) => {
		await set(newVal)
	},
	{ deep: true },
)

const messages = {
	title: defineMessage({ id: 'settings.sync.title', defaultMessage: 'Instance Sync' }),
	description: defineMessage({
		id: 'settings.sync.description',
		defaultMessage:
			'These options are used as defaults for new instances. Existing instances keep their own sync overrides.',
	}),
	enable: defineMessage({ id: 'settings.sync.enable', defaultMessage: 'Enable instance sync' }),
	defaults: defineMessage({
		id: 'settings.sync.defaults',
		defaultMessage: 'Default items for new instances',
	}),
}
</script>

<template>
	<h2 class="m-0 text-lg font-semibold text-contrast">
		{{ formatMessage(messages.title) }}
	</h2>
	<p class="m-0 mt-1">{{ formatMessage(messages.description) }}</p>
	<div class="mt-4 rounded-xl border border-solid border-surface-5 p-3">
		<Checkbox v-model="localSettings.sync.enabled" :label="formatMessage(messages.enable)" />
	</div>
	<div v-if="localSettings.sync.enabled" class="mt-4 rounded-xl border border-solid border-surface-5 p-3">
		<h3 class="m-0 mb-3 text-sm font-semibold text-secondary">
			{{ formatMessage(messages.defaults) }}
		</h3>
		<div class="grid gap-2">
		<Checkbox
			v-for="key in allFolderTargets"
			:key="key"
			:model-value="isEnabled('folder', key)"
			:label="labels[key] ?? key"
			@update:model-value="setEnabled('folder', key, !!$event)"
		/>
		<Checkbox
			v-for="key in allFileTargets"
			:key="key"
			:model-value="isEnabled('file', key)"
			:label="labels[key] ?? key"
			@update:model-value="setEnabled('file', key, !!$event)"
		/>
		</div>
	</div>
</template>
