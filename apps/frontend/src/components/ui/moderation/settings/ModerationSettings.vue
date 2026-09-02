<script setup lang="ts">
import { moderationSettings, type SettingDefinition } from '@modrinth/moderation'
import { Button, Combobox, Input, Toggle } from '@modrinth/ui'

const localhostHostname = 'localhost:3000'

const flattenedSettings = Object.entries(moderationSettings).reduce(
	(acc, [group, settings]) => {
		acc[group] = Object.values(settings)
		return acc
	},
	{} as { [name: string]: SettingDefinition[] },
)

onMounted(() => {
	const merged: { [name: string]: SettingDefinition[] } = {}
	const addMergedSettings = (settings: { [name: string]: SettingDefinition[] }) => {
		for (const [groupId, groupSettings] of Object.entries(settings)) {
			const group = (merged[groupId] = merged[groupId] || [])
			group.push(...groupSettings)
		}
	}

	addMergedSettings(flattenedSettings)
	const event = new CustomEvent('request-moderation-settings', {
		detail: {
			addSettings: addMergedSettings,
		},
	})
	window.dispatchEvent(event)

	displayedSettings.value = merged
})

const configuredSettings = useModerationSettings()
const displayedSettings = ref<{ [name: string]: SettingDefinition[] }>(flattenedSettings)
</script>

<template>
	<div v-for="[name, page] in Object.entries(displayedSettings)" :key="name" class="universal-card">
		<h2 class="text-2xl">{{ name }}</h2>
		<div class="flex flex-col gap-3">
			<div
				v-for="setting in page"
				:key="setting.id"
				class="flex flex-row flex-wrap items-center justify-between gap-2"
			>
				<label class="flex-1">
					<span class="mb-1 block font-semibold text-contrast">{{ setting.title }}</span>
					<span class="block text-secondary">{{ setting.description }}</span>
					<span class="mt-1 block text-secondary">
						Default:
						<span
							class="font-medium"
							:class="{
								'text-red': setting.type === 'toggle' && !setting.default,
								'text-green': setting.type === 'toggle' && setting.default,
								'italic opacity-50': setting.default === null,
							}"
							>{{ setting.default ?? '(none)' }}
						</span>
					</span>
				</label>

				<Toggle
					v-if="setting.type === 'toggle'"
					:model-value="configuredSettings.get(setting)"
					class="shrink-0"
					@update:model-value="(value) => configuredSettings.set(setting, value)"
				/>
				<Combobox
					v-if="setting.type === 'enum'"
					:model-value="configuredSettings.get(setting)"
					:options="setting.entries.map((entry) => ({ value: entry.value, label: entry.label }))"
					class="!w-1/4"
					@update:model-value="(value) => configuredSettings.set(setting, value)"
				/>
				<div v-if="setting.type === 'string'" class="flex !w-1/4 flex-col items-start gap-2">
					<Input
						type="text"
						:model-value="configuredSettings.get(setting) ?? ''"
						@update:model-value="(value) => configuredSettings.set(setting, String(value ?? ''))"
					/>
					<Button
						v-if="setting.id === 'alternative-hostname'"
						type="outlined"
						size="sm"
						class="shrink-0"
						:disabled="configuredSettings.get(setting) === localhostHostname"
						@click="configuredSettings.set(setting, localhostHostname)"
					>
						Set to {{ localhostHostname }}
					</Button>
				</div>
			</div>
		</div>
	</div>
</template>
