<template>
	<div class="relative h-full w-full select-none overflow-y-auto">
		<div v-if="propsData" class="flex h-full w-full flex-col justify-between gap-4 overflow-y-auto">
			<Admonition
				v-if="missingKnownProperties.length > 0"
				type="warning"
				body="Some expected properties are missing from your server.properties - this usually means the server hasn't completed its first startup yet."
			/>
			<div class="card flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<h2 class="m-0 text-lg font-bold text-contrast">Server properties</h2>
					<div class="m-0">
						Edit the Minecraft server properties file. If you're unsure about a specific property,
						the
						<NuxtLink
							class="goto-link !inline-block"
							to="https://minecraft.wiki/w/Server.properties"
							external
						>
							Minecraft Wiki
						</NuxtLink>
						has more detailed information.
					</div>
				</div>
				<div class="flex flex-col gap-4 rounded-2xl bg-table-alternateRow p-4">
					<div class="w-full text-sm">
						<label for="search-server-properties" class="sr-only"> Search server properties </label>
						<StyledInput
							id="search-server-properties"
							v-model="searchInput"
							wrapper-class="w-full"
							type="search"
							:icon="SearchIcon"
							name="search"
							autocomplete="off"
							placeholder="Search server properties..."
						/>
					</div>
					<div
						v-for="(_value, key) in filteredProperties"
						:key="key"
						class="flex flex-row flex-wrap items-center justify-between py-2"
					>
						<span :id="`property-label-${key}`">{{ formatPropertyName(key) }}</span>

						<div
							v-if="getPropertyDef(key).type === 'dropdown'"
							class="mt-2 flex w-full sm:w-[320px] sm:justify-end"
						>
							<Combobox
								:id="`server-property-${key}`"
								v-model="liveProperties[key]"
								:name="formatPropertyName(key)"
								:options="
									(getPropertyDef(key) as DropdownPropertyDef).options.map((v) => ({
										value: v,
										label: formatPropertyName(v),
									}))
								"
								:aria-labelledby="`property-label-${key}`"
								:display-value="formatPropertyName(String(liveProperties[key] ?? 'Select...'))"
							/>
						</div>
						<div v-else-if="getPropertyDef(key).type === 'toggle'" class="flex justify-end">
							<Toggle
								:id="`server-property-${key}`"
								:model-value="liveProperties[key] === 'true'"
								:aria-labelledby="`property-label-${key}`"
								@update:model-value="liveProperties[key] = $event ? 'true' : 'false'"
							/>
						</div>
						<div v-else-if="getPropertyDef(key).type === 'number'" class="mt-2 w-full sm:w-[320px]">
							<StyledInput
								:id="`server-property-${key}`"
								:model-value="liveProperties[key]"
								type="number"
								wrapper-class="w-full"
								:aria-labelledby="`property-label-${key}`"
								@update:model-value="liveProperties[key] = String($event)"
							/>
						</div>
						<div v-else class="mt-2 flex w-full justify-end sm:w-[320px]">
							<StyledInput
								:id="`server-property-${key}`"
								v-model="liveProperties[key]"
								wrapper-class="w-full"
								:aria-labelledby="`property-label-${key}`"
							/>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div v-else class="flex h-full w-full items-center justify-center">
			<SpinnerIcon class="animate-spin" />
		</div>

		<SaveBanner
			:is-visible="hasUnsavedChanges"
			:server-id="serverId"
			:is-updating="isUpdating"
			restart
			:save="() => saveProperties()"
			:reset="resetProperties"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { SearchIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Admonition,
	Combobox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	StyledInput,
	Toggle,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import Fuse from 'fuse.js'
import { computed, ref, watch } from 'vue'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { serverId, worldId, powerState } = injectModrinthServerContext()
const queryClient = useQueryClient()

const searchInput = ref('')

type DropdownPropertyDef = { type: 'dropdown'; options: string[] }
type PropertyDef = { type: 'toggle' } | { type: 'number' } | { type: 'text' } | DropdownPropertyDef

const KNOWN_PROPERTIES: Record<string, PropertyDef> = {
	allow_cheats: { type: 'toggle' },
	allow_flight: { type: 'toggle' },
	difficulty: { type: 'dropdown', options: ['peaceful', 'easy', 'normal', 'hard'] },
	enforce_whitelist: { type: 'toggle' },
	force_gamemode: { type: 'toggle' },
	gamemode: { type: 'dropdown', options: ['survival', 'creative', 'adventure', 'spectator'] },
	generate_structures: { type: 'toggle' },
	generator_settings: { type: 'text' },
	hardcore: { type: 'toggle' },
	level_seed: { type: 'text' },
	level_type: { type: 'text' },
	max_players: { type: 'number' },
	max_tick_time: { type: 'number' },
	motd: { type: 'text' },
	pause_when_empty_seconds: { type: 'number' },
	player_idle_timeout: { type: 'number' },
	require_resource_pack: { type: 'toggle' },
	resource_pack: { type: 'text' },
	resource_pack_id: { type: 'text' },
	resource_pack_sha1: { type: 'text' },
	simulation_distance: { type: 'number' },
	spawn_protection: { type: 'number' },
	sync_chunk_writes: { type: 'toggle' },
	view_distance: { type: 'number' },
	white_list: { type: 'toggle' },
}

function getPropertyDef(key: string): PropertyDef {
	return KNOWN_PROPERTIES[key] ?? { type: 'text' }
}

const queryKey = computed(() => ['servers', 'properties', 'v1', serverId, worldId.value])

const { data: propsData } = useQuery({
	queryKey,
	queryFn: () => client.archon.properties_v1.getProperties(serverId, worldId.value!),
	enabled: computed(() => worldId.value !== null),
})

function flattenProperties(data: Archon.Content.v1.PropertiesFields): Record<string, string> {
	const result: Record<string, string> = {}
	if (data.known) {
		for (const [key, value] of Object.entries(data.known)) {
			if (value != null) result[key] = value
		}
	}
	if (data.custom) {
		for (const [key, value] of Object.entries(data.custom)) {
			if (value != null) result[key] = value
		}
	}
	return result
}

const liveProperties = ref<Record<string, string>>({})
const originalProperties = ref<Record<string, string>>({})

function syncFormFromData() {
	if (!propsData.value) return
	const flat = flattenProperties(propsData.value)
	liveProperties.value = { ...flat }
	originalProperties.value = { ...flat }
}

watch(
	propsData,
	(newData, oldData) => {
		if (newData && !oldData) {
			syncFormFromData()
		}
	},
	{ immediate: true },
)

watch(powerState, () => {
	queryClient.invalidateQueries({ queryKey: queryKey.value })
})

const missingKnownProperties = computed(() =>
	Object.keys(KNOWN_PROPERTIES).filter((key) => !(key in liveProperties.value)),
)

const hasUnsavedChanges = computed(() =>
	Object.keys(liveProperties.value).some(
		(key) => liveProperties.value[key] !== originalProperties.value[key],
	),
)

function buildPatch(): Archon.Content.v1.PatchPropertiesFields {
	const known: Record<string, string> = {}
	const custom: Record<string, string> = {}

	for (const key of Object.keys(liveProperties.value)) {
		if (liveProperties.value[key] === originalProperties.value[key]) continue
		if (key in KNOWN_PROPERTIES) {
			known[key] = liveProperties.value[key]
		} else {
			custom[key] = liveProperties.value[key]
		}
	}

	const patch: Archon.Content.v1.PatchPropertiesFields = {}
	if (Object.keys(known).length > 0) {
		patch.known = known as Archon.Content.v1.KnownPropertiesFields
	}
	if (Object.keys(custom).length > 0) {
		patch.custom = custom
	}
	return patch
}

const { mutate: saveProperties, isPending: isUpdating } = useMutation({
	mutationFn: () =>
		client.archon.properties_v1.patchProperties(serverId, worldId.value!, buildPatch()),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
		syncFormFromData()
		addNotification({
			type: 'success',
			title: 'Server properties updated',
			text: 'Your server properties were successfully changed.',
		})
	},
	onError: (error) => {
		addNotification({
			type: 'error',
			title: 'Failed to update server properties',
			text: error instanceof Error ? error.message : 'An error occurred.',
		})
	},
})

function resetProperties() {
	syncFormFromData()
}

const fuse = computed(() => {
	const entries = Object.entries(liveProperties.value).map(([key, value]) => ({
		key,
		value: String(value),
	}))
	return new Fuse(entries, { keys: ['key', 'value'], threshold: 0.2 })
})

const filteredProperties = computed(() => {
	if (!searchInput.value?.trim()) return liveProperties.value
	const results = fuse.value.search(searchInput.value)
	return Object.fromEntries(results.map(({ item }) => [item.key, liveProperties.value[item.key]]))
})

function formatPropertyName(name: string): string {
	return name
		.split('_')
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}
</script>
