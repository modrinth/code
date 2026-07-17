<script setup lang="ts">
import {
	moderationSettings,
	type SettingDefinition,
} from "@modrinth/moderation"
import {Combobox, Toggle} from "@modrinth/ui";

const flattenedSettings = Object.entries(moderationSettings)
	.reduce((acc, [group, settings]) => {
		acc[group] = Object.values(settings)
		return acc
	}, {} as { [name: string]: SettingDefinition[] })

onMounted(() => {
	const merged: { [name: string]: SettingDefinition[] } = {}
	const addMergedSettings = (settings: { [name: string]: SettingDefinition[] }) => {
		for (let [groupId, groupSettings] of Object.entries(settings)) {
			const group = (merged[groupId] = merged[groupId] || [])
			group.push(...groupSettings)
		}
	}

	addMergedSettings(flattenedSettings)
	const event = new CustomEvent('request-moderation-settings', {
		detail: {
			addSettings: addMergedSettings
		}
	})
	window.dispatchEvent(event)

	displayedSettings.value = merged
})

const configuredSettings = useModerationSettings()
const displayedSettings = ref<{ [name: string]: SettingDefinition[] }>(flattenedSettings)
</script>

<template>
	<div v-for="[name, page] of Object.entries(displayedSettings)" :key="name" class="universal-card">
		<h2 class="text-2xl">{{ name }}</h2>
		<div class="flex flex-col gap-2">
			<div v-for="setting in page" :key="setting.id"
			     class="flex flex-row flex-wrap items-center justify-between gap-2">
				<label class="flex-1">
					<span class="block font-semibold text-contrast">{{ setting.title }}</span>
					<span class="block text-secondary">{{ setting.description }}</span>
					<span class="block text-secondary">
						Default:
						<span
							class="font-semibold"
							:class="{
							  'text-red': setting.type === 'toggle' && !setting.default,
							  'text-green': setting.type === 'toggle' && setting.default
							}"
						>{{ setting.default }}</span>
					</span>
				</label>

				<Toggle
					v-if="setting.type === 'toggle'"
					:model-value="configuredSettings.get(setting)"
					@update:model-value="(value) => configuredSettings.set(setting, value)"
					class="shrink-0"
				/>
				<Combobox
					v-if="setting.type === 'enum'"
					:model-value="configuredSettings.get(setting)"
					@update:model-value="(value) => configuredSettings.set(setting, value)"
					:options="setting.entries.map(entry => ({ value: entry.value, label: entry.label }))"
					class="!w-1/4"
				/>
				<input
					v-if="setting.type === 'string'"
					type="text"
					:value="configuredSettings.get(setting)"
					@input="(event) => configuredSettings.set(setting, (event.target as HTMLInputElement).value)"
					class="input !w-1/4"
				>
			</div>
		</div>
	</div>
</template>
