<script setup lang="ts">
import { Checkbox, injectNotificationManager } from '@icarus/ui'
import { defineMessage, useVIntl } from '@icarus/ui'
import { computed, ref } from 'vue'
import { set_sync_enabled, set_sync_overrides } from '@/helpers/profile'
import { injectInstanceSettings } from '@/providers/instance-settings'

const { formatMessage } = useVIntl()
const { notifyError } = injectNotificationManager()
const { instance } = injectInstanceSettings()

const localEnabled = ref(instance.value.sync_enabled ?? true)
const localOverrides = ref({
	saves: instance.value.sync_overrides?.saves ?? undefined,
	screenshots: instance.value.sync_overrides?.screenshots ?? undefined,
	resourcepacks: instance.value.sync_overrides?.resourcepacks ?? undefined,
	shaderpacks: instance.value.sync_overrides?.shaderpacks ?? undefined,
	schematics: instance.value.sync_overrides?.schematics ?? undefined,
	options_txt: instance.value.sync_overrides?.options_txt ?? undefined,
	servers_dat: instance.value.sync_overrides?.servers_dat ?? undefined,
})

const items = computed(() => [
	{ key: 'saves', label: 'Worlds' },
	{ key: 'screenshots', label: 'Screenshots' },
	{ key: 'resourcepacks', label: 'Resourcepacks' },
	{ key: 'shaderpacks', label: 'Shaderpacks' },
	{ key: 'schematics', label: 'Litematica Schematics' },
	{ key: 'options_txt', label: 'Game Options' },
	{ key: 'servers_dat', label: 'Server List' },
])

async function toggleEnabled(v: boolean) {
	localEnabled.value = v
	try {
		await set_sync_enabled(instance.value.path, v)
	} catch (e) {
		localEnabled.value = !v
		notifyError('Sync error', String(e))
	}
}

async function toggleOverride(key: string, v: boolean) {
	;(localOverrides.value as Record<string, boolean | undefined>)[key] = v
	try {
		await set_sync_overrides(instance.value.path, localOverrides.value)
	} catch (e) {
		notifyError('Sync error', String(e))
	}
}

const messages = {
	title: defineMessage({ id: 'instance.sync.title', defaultMessage: 'Instance Sync' }),
	enable: defineMessage({
		id: 'instance.sync.enable',
		defaultMessage: 'Enable sync for this instance',
	}),
	items: defineMessage({
		id: 'instance.sync.items',
		defaultMessage: 'Items to sync for this instance',
	}),
}
</script>

<template>
	<h2 class="m-0 text-lg font-semibold text-contrast">
		{{ formatMessage(messages.title) }}
	</h2>
	<div class="mt-4 rounded-xl border border-solid border-surface-5 p-3">
		<Checkbox
			:model-value="localEnabled"
			:label="formatMessage(messages.enable)"
			@update:model-value="toggleEnabled(!!$event)"
		/>
	</div>
	<div v-if="localEnabled" class="mt-4 rounded-xl border border-solid border-surface-5 p-3">
		<h3 class="m-0 mb-3 text-sm font-semibold text-secondary">
			{{ formatMessage(messages.items) }}
		</h3>
		<div class="grid gap-2">
		<Checkbox
			v-for="item in items"
			:key="item.key"
			:model-value="(localOverrides as any)[item.key] ?? true"
			:label="item.label"
			@update:model-value="toggleOverride(item.key, !!$event)"
		/>
		</div>
	</div>
</template>
