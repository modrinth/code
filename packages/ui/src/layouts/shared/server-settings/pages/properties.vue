<template>
	<div class="relative h-screen w-full select-none max-h-[min(70vh,750px)]">
		<div v-if="propsData" class="flex h-full w-full flex-col justify-between gap-4">
			<div class="flex flex-col gap-4">
				<Admonition
					v-if="hasNoProperties"
					type="warning"
					body="Some expected properties are missing from your server.properties - this usually means the server hasn't completed its first startup yet."
				/>
				<div class="flex flex-col gap-2">
					<div class="m-0">
						Edit the Minecraft server properties file here, or use the
						<AutoLink
							class="goto-link !inline-block"
							:to="filesTabLink"
							@click="onFilesTabLinkClick"
						>
							Files tab
						</AutoLink>
						to edit the full file. If you're unsure about a setting, the
						<AutoLink
							class="goto-link !inline-block"
							to="https://minecraft.wiki/w/Server.properties"
							target="_blank"
						>
							Minecraft Wiki
						</AutoLink>
						has more details.
					</div>
				</div>

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
				<div class="flex flex-col gap-3 pb-2">
					<div class="flex flex-col gap-6">
						<!-- Basic Properties -->
						<!-- [&:not(:has(*:not(:empty)))]:hidden is to hide parent if all children are empty -->
						<div
							class="rounded-2xl border border-solid border-surface-5 p-4 pb-2 [&:not(:has(*:not(:empty)))]:hidden"
						>
							<div class="flex w-full flex-col gap-1.5">
								<div v-if="isPropertyVisible('gamemode')" class="flex flex-col gap-2.5 my-1">
									<span class="font-semibold text-contrast">Gamemode</span>
									<Chips
										v-model="combinedGamemode"
										:items="gamemodeItems"
										:format-label="capitalize"
									/>
								</div>

								<div
									v-if="combinedGamemode !== 'hardcore' && isPropertyVisible('difficulty')"
									class="flex flex-col gap-2.5 my-1"
								>
									<span class="font-semibold text-contrast">Difficulty</span>
									<Chips
										v-model="selectedDifficulty"
										:items="difficultyItems"
										:format-label="capitalize"
									/>
								</div>

								<div v-if="isPropertyVisible('max_players')" class="flex flex-col gap-2.5 my-1">
									<span class="font-semibold text-contrast">Max players</span>
									<StyledInput
										id="server-property-max-players"
										:model-value="liveProperties.max_players"
										type="number"
										placeholder="20"
										wrapper-class="w-full max-w-[450px]"
										@update:model-value="liveProperties.max_players = String($event)"
									/>
								</div>

								<div v-if="isPropertyVisible('motd')" class="flex flex-col gap-2.5 my-1">
									<span class="font-semibold text-contrast">MOTD</span>
									<StyledInput
										id="server-property-motd"
										v-model="liveProperties.motd"
										placeholder="A Minecraft Server"
										wrapper-class="w-full max-w-[450px]"
									/>
								</div>

								<div
									v-if="isPropertyVisible('allow_flight')"
									class="flex flex-row items-center justify-between gap-4 h-10"
								>
									<span class="font-semibold text-contrast">Allow flight</span>
									<Toggle
										id="server-property-allow-flight"
										:model-value="liveProperties.allow_flight === 'true'"
										@update:model-value="liveProperties.allow_flight = $event ? 'true' : 'false'"
									/>
								</div>

								<div
									v-if="isPropertyVisible('allow_cheats')"
									class="flex flex-row items-center justify-between gap-4 h-10"
								>
									<span class="font-semibold text-contrast">Allow cheats</span>
									<Toggle
										id="server-property-allow-cheats"
										:model-value="liveProperties.allow_cheats === 'true'"
										@update:model-value="liveProperties.allow_cheats = $event ? 'true' : 'false'"
									/>
								</div>

								<div
									v-if="isPropertyVisible('white_list')"
									class="flex flex-row items-center justify-between gap-4 h-10"
								>
									<span class="font-semibold text-contrast">Enable whitelist</span>
									<Toggle id="server-property-whitelist" v-model="whitelistEnabled" />
								</div>

								<div
									v-if="isPropertyVisible('spawn_protection')"
									class="flex flex-row items-center justify-between gap-4 h-10"
								>
									<span class="font-semibold text-contrast">Enable spawn protection</span>
									<Toggle
										id="server-property-spawn-protection-toggle"
										v-model="spawnProtectionEnabled"
									/>
								</div>

								<div
									v-if="spawnProtectionEnabled && isPropertyVisible('spawn_protection')"
									class="flex items-center justify-between h-10"
								>
									<span class="font-semibold text-contrast">Protection radius</span>
									<StyledInput
										id="server-property-spawn-protection-radius"
										:model-value="liveProperties.spawn_protection"
										type="number"
										wrapper-class="w-full sm:w-[100px]"
										input-class="text-right"
										@update:model-value="liveProperties.spawn_protection = String($event)"
									/>
								</div>
							</div>
						</div>
					</div>
					<!-- Advanced Properties -->
					<Accordion
						v-if="hasVisibleAdvancedProperties"
						overflow-visible
						:force-open="isSearchActive"
						button-class="flex w-full flex-col gap-2 bg-transparent m-0 p-0 border-none"
					>
						<template #title>
							<span class="text-lg font-semibold text-contrast">Advanced properties</span>
						</template>

						<div class="flex flex-col gap-6 pt-4">
							<template v-for="group in advancedGroupedProperties" :key="group.label">
								<div v-if="hasVisibleProperties(group)" class="flex flex-col gap-2.5">
									<h3 class="m-0 text-base font-semibold text-contrast">
										{{ group.label }}
									</h3>
									<div
										class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-5 p-4"
									>
										<template v-for="key in group.properties" :key="key">
											<div
												v-if="isPropertyVisible(key)"
												class="flex flex-row flex-wrap items-center justify-between h-10"
											>
												<span :id="`property-label-${key}`" class="font-semibold text-contrast">
													{{ formatPropertyName(key) }}
												</span>

												<div
													v-if="getPropertyDef(key).type === 'toggle'"
													class="flex w-full justify-end sm:w-[320px]"
												>
													<Toggle
														:id="`server-property-${key}`"
														:model-value="liveProperties[key] === 'true'"
														:aria-labelledby="`property-label-${key}`"
														@update:model-value="liveProperties[key] = $event ? 'true' : 'false'"
													/>
												</div>
												<div
													v-else-if="getPropertyDef(key).type === 'number'"
													class="w-full sm:w-[320px]"
												>
													<StyledInput
														:id="`server-property-${key}`"
														:model-value="liveProperties[key]"
														type="number"
														placeholder="Type here..."
														wrapper-class="w-full"
														:aria-labelledby="`property-label-${key}`"
														@update:model-value="liveProperties[key] = String($event)"
													/>
												</div>
												<div v-else class="flex w-full justify-end sm:w-[320px]">
													<StyledInput
														:id="`server-property-${key}`"
														v-model="liveProperties[key]"
														placeholder="Type here..."
														wrapper-class="w-full"
														:aria-labelledby="`property-label-${key}`"
													/>
												</div>
											</div>
										</template>
									</div>
								</div>
							</template>
							<div>
								All other properties can be edited in server.properties via the
								<AutoLink
									class="goto-link !inline-block"
									:to="filesTabLink"
									@click="onFilesTabLinkClick"
								>
									Files tab </AutoLink
								>.
							</div>
						</div>
					</Accordion>

					<div
						v-if="hasNoResults"
						class="flex flex-col items-center gap-2 py-8 text-center text-secondary"
					>
						<SearchIcon class="size-10" />
						<span class="text-lg font-semibold text-contrast">No properties found</span>
						<span>No properties match "{{ searchInput }}".</span>
					</div>
				</div>
			</div>
		</div>
		<div v-else class="flex h-full w-full items-center justify-center">
			<SpinnerIcon class="animate-spin" />
		</div>

		<SaveBanner
			:is-visible="hasUnsavedChanges || isUpdating"
			:server-id="serverId"
			:is-updating="isUpdating || busyReasons.length > 0"
			restart
			:save="
				async () => {
					await saveProperties()
				}
			"
			:reset="resetProperties"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { SearchIcon, SpinnerIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import Fuse from 'fuse.js'
import { computed, ref, watch } from 'vue'

import { Accordion, Admonition, AutoLink, Chips, StyledInput, Toggle } from '#ui/components'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import { injectServerSettings } from '#ui/layouts/shared/server-settings'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { serverId, worldId, powerState, busyReasons } = injectModrinthServerContext()
const queryClient = useQueryClient()
const filesTabLink = computed(
	() => `/hosting/manage/${encodeURIComponent(serverId)}/files?path=/&editing=server.properties`,
)
const serverSettings = injectServerSettings(null)

const searchInput = ref('')

function onFilesTabLinkClick() {
	serverSettings?.closeModal?.()
}

type PropertyDef = { type: 'toggle' } | { type: 'number' } | { type: 'text' }

const KNOWN_PROPERTIES: Record<string, PropertyDef> = {
	allow_cheats: { type: 'toggle' },
	allow_flight: { type: 'toggle' },
	difficulty: { type: 'text' },
	enforce_whitelist: { type: 'toggle' },
	force_gamemode: { type: 'toggle' },
	gamemode: { type: 'text' },
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

const ADVANCED_GROUPS = [
	{
		label: 'Performance',
		keys: [
			'view_distance',
			'simulation_distance',
			'sync_chunk_writes',
			'max_tick_time',
			'player_idle_timeout',
			'pause_when_empty_seconds',
		],
	},
	{
		label: 'Resource Pack',
		keys: ['resource_pack', 'resource_pack_id', 'resource_pack_sha1', 'require_resource_pack'],
	},
]

type CombinedGamemode = 'survival' | 'creative' | 'hardcore'
const gamemodeItems: CombinedGamemode[] = ['survival', 'creative', 'hardcore']
const difficultyItems = ['peaceful', 'easy', 'normal', 'hard']

function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1)
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
let previousSpawnProtection = '16'

function syncFormFromData() {
	if (!propsData.value) return
	const flat = flattenProperties(propsData.value)
	liveProperties.value = { ...flat }
	originalProperties.value = { ...flat }
	const sp = flat.spawn_protection
	if (sp && sp !== '0') {
		previousSpawnProtection = sp
	}
}

const hasNoProperties = computed(() => Object.keys(liveProperties.value).length === 0)

const hasUnsavedChanges = computed(() =>
	Object.keys(liveProperties.value).some(
		(key) => liveProperties.value[key] !== originalProperties.value[key],
	),
)

watch(
	propsData,
	(newData) => {
		if (newData && !hasUnsavedChanges.value) {
			syncFormFromData()
		}
	},
	{ immediate: true },
)

watch(powerState, () => {
	queryClient.invalidateQueries({ queryKey: queryKey.value })
})

const combinedGamemode = computed<CombinedGamemode>({
	get() {
		if (liveProperties.value.hardcore === 'true') return 'hardcore'
		if (liveProperties.value.gamemode === 'creative') return 'creative'
		return 'survival'
	},
	set(value) {
		if (value === 'hardcore') {
			liveProperties.value.gamemode = 'survival'
			liveProperties.value.hardcore = 'true'
			liveProperties.value.difficulty = 'hard'
		} else {
			liveProperties.value.gamemode = value
			liveProperties.value.hardcore = 'false'
		}
	},
})

const selectedDifficulty = computed({
	get: () => liveProperties.value.difficulty ?? 'normal',
	set: (v: string) => {
		liveProperties.value.difficulty = v
	},
})

const whitelistEnabled = computed({
	get: () => liveProperties.value.white_list === 'true',
	set: (v: boolean) => {
		liveProperties.value.white_list = v ? 'true' : 'false'
		liveProperties.value.enforce_whitelist = v ? 'true' : 'false'
	},
})

const spawnProtectionEnabled = computed({
	get: () => {
		const val = liveProperties.value.spawn_protection
		return val !== undefined && val !== '0'
	},
	set: (enabled: boolean) => {
		if (enabled) {
			liveProperties.value.spawn_protection = previousSpawnProtection || '16'
		} else {
			previousSpawnProtection = liveProperties.value.spawn_protection || '16'
			liveProperties.value.spawn_protection = '0'
		}
	},
})

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

const { mutateAsync: saveProperties, isPending: isUpdating } = useMutation({
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

const advancedGroupedProperties = computed(() =>
	ADVANCED_GROUPS.map((group) => ({
		label: group.label,
		properties: group.keys.filter((key) => key in liveProperties.value),
	})).filter((g) => g.properties.length > 0),
)

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

const isSearchActive = computed(() => !!searchInput.value?.trim())
const hasNoResults = computed(
	() => isSearchActive.value && Object.keys(filteredProperties.value).length === 0,
)

function isPropertyVisible(key: string): boolean {
	if (!isSearchActive.value) return true
	return key in filteredProperties.value
}

function hasVisibleProperties(group: { properties: string[] }): boolean {
	return group.properties.some((key) => isPropertyVisible(key))
}

const hasVisibleAdvancedProperties = computed(() =>
	advancedGroupedProperties.value.some((group) => hasVisibleProperties(group)),
)

function formatPropertyName(name: string): string {
	return name
		.split('_')
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}
</script>
