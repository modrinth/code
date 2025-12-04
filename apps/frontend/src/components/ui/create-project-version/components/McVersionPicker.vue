<template>
	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">
				Minecraft versions <span class="text-red">*</span>
			</span>

			<Chips
				v-model="versionType"
				:items="['release', 'all']"
				:never-empty="false"
				:capitalize="true"
				size="small"
			/>
		</div>
		<div class="iconified-input w-full rounded-xl border-[1px] border-solid border-surface-5">
			<SearchIcon aria-hidden="true" />
			<input v-model="searchQuery" type="text" placeholder="Search versions" />
		</div>
		<div
			class="user-select-none flex max-h-72 flex-col gap-3 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3 py-4"
		>
			<div v-for="group in groupedGameVersions" :key="group.key" class="space-y-1.5">
				<span class="font-semibold">{{ group.key }}</span>
				<div class="flex flex-wrap gap-2 gap-x-1.5">
					<ButtonStyled
						v-for="version in group.versions"
						:key="version"
						:color="modelValue.includes(version) ? 'green' : 'standard'"
						:highlighted="modelValue.includes(version)"
						type="chip"
					>
						<button
							class="!text-contrast focus:outline-none"
							:class="versionType === 'all' ? 'w-26' : 'w-16'"
							@click="toggleVersion(version, $event)"
						>
							{{ version }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<span v-if="!filteredVersions.length">No versions found.</span>
		</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { SearchIcon } from '@modrinth/assets'
import { ButtonStyled, Chips } from '@modrinth/ui'
import { computed } from 'vue'

type GameVersion = Labrinth.Tags.v2.GameVersion

const props = defineProps<{
	modelValue: string[]
	gameVersions: Labrinth.Tags.v2.GameVersion[]
}>()

const emit = defineEmits<{
	(e: 'update:modelValue', value: string[]): void
}>()

const versionType = ref<string | null>('release')

const filteredVersions = computed(() =>
	props.gameVersions
		.filter((v) => versionType.value === 'all' || v.version_type === versionType.value)
		.filter(searchFilter),
)

const groupedGameVersions = computed(() => groupVersions(filteredVersions.value))

const toggleVersion = (version: string, event: MouseEvent) => {
	const next = props.modelValue.includes(version)
		? props.modelValue.filter((v) => v !== version)
		: [...props.modelValue, version]
	emit('update:modelValue', next)
}

const DEV_RELEASE_KEY = 'development releases'

function groupVersions(gameVersions: GameVersion[]) {
	gameVersions = [...gameVersions].sort(
		(a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
	)

	const getGroupKey = (v: string) => v.split('.').slice(0, 2).join('.')
	const groups: Record<string, string[]> = {}

	let currentGroupKey = getGroupKey(gameVersions.find((v) => v.major)?.version || '')

	gameVersions.forEach((gameVersions) => {
		if (gameVersions.version_type === 'release') {
			currentGroupKey = getGroupKey(gameVersions.version)
			if (!groups[currentGroupKey]) groups[currentGroupKey] = []
			groups[currentGroupKey].push(gameVersions.version)
		} else {
			if (!groups[`${currentGroupKey} ${DEV_RELEASE_KEY}`])
				groups[`${currentGroupKey} ${DEV_RELEASE_KEY}`] = []
			groups[`${currentGroupKey} ${DEV_RELEASE_KEY}`].push(gameVersions.version)
		}
	})

	const sortedKeys = Object.keys(groups).sort(compareGroupKeys)
	const result = sortedKeys.map((key) => ({
		key,
		versions: groups[key].sort((a, b) => compareVersions(b, a)),
	}))
	return result
}

const getBaseVersion = (key: string) => key.split(' ')[0]

function compareVersions(a: string, b: string) {
	const pa = a.split('.').map(Number)
	const pb = b.split('.').map(Number)

	// checking major first, then minor, then patch
	for (let i = 0; i < 2; i++) {
		const na = pa[i] || 0
		const nb = pb[i] || 0
		if (na > nb) return 1
		if (na < nb) return -1
	}
	return 0
}

function compareGroupKeys(a: string, b: string) {
	const aBase = getBaseVersion(a)
	const bBase = getBaseVersion(b)

	const versionSort = compareVersions(aBase, bBase)
	if (versionSort !== 0) return -versionSort // descending

	const isADev = a.toLowerCase().includes(DEV_RELEASE_KEY)
	const isBDev = b.toLowerCase().includes(DEV_RELEASE_KEY)

	if (isADev && !isBDev) return 1
	if (!isADev && isBDev) return -1

	return 0
}

const searchQuery = ref('')

function searchFilter(gameVersion: Labrinth.Tags.v2.GameVersion) {
	return gameVersion.version.toLowerCase().includes(searchQuery.value.toLowerCase())
}
</script>
