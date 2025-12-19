<template>
	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">
				Minecraft versions <span class="text-red">*</span>
			</span>

			<Chips
				v-model="versionType"
				:items="['release', 'all']"
				:never-empty="true"
				:capitalize="true"
				size="small"
			/>
		</div>
		<div class="iconified-input w-full">
			<SearchIcon aria-hidden="true" />
			<input v-model="searchQuery" type="text" placeholder="Search versions" />
		</div>
		<div
			class="flex h-72 select-none flex-col gap-3 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3 py-4"
		>
			<div v-for="group in groupedGameVersions" :key="group.key" class="space-y-1.5">
				<span class="font-semibold">{{ group.key }}</span>
				<div class="flex flex-wrap gap-2 gap-x-1.5">
					<ButtonStyled
						v-for="version in group.versions"
						:key="version"
						:color="
							holdingShift && version === anchorVersion
								? 'purple'
								: modelValue.includes(version)
									? 'green'
									: 'standard'
						"
						:highlighted="modelValue.includes(version)"
						type="chip"
					>
						<button
							class="!py-1.5 focus:outline-none"
							:class="[
								versionType === 'all' && !group.isReleaseGroup ? 'w-max' : 'w-16',
								modelValue.includes(version) ? '!text-contrast' : '',
							]"
							@click="() => handleToggleVersion(version)"
							@blur="
								() => {
									if (!holdingShift) anchorVersion = ''
								}
							"
						>
							{{ version }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<span v-if="!filteredVersions.length">No versions found.</span>
		</div>
		<div>Hold shift and click to select range.</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { SearchIcon } from '@modrinth/assets'
import { ButtonStyled, Chips } from '@modrinth/ui'
import { useMagicKeys } from '@vueuse/core'
import { computed, ref } from 'vue'

type GameVersion = Labrinth.Tags.v2.GameVersion

const props = defineProps<{
	modelValue: string[]
	gameVersions: Labrinth.Tags.v2.GameVersion[]
}>()

const emit = defineEmits<{
	(e: 'update:modelValue', value: string[]): void
}>()

const keys = useMagicKeys()
const holdingShift = computed(() => keys.shift.value)

const versionType = ref<string | null>('release')
const searchQuery = ref('')

const filteredVersions = computed(() =>
	props.gameVersions
		.filter((v) => versionType.value === 'all' || v.version_type === versionType.value)
		.filter(searchFilter),
)

const groupedGameVersions = computed(() => groupVersions(filteredVersions.value))

const allVersionsFlat = computed(() => groupedGameVersions.value.flatMap((group) => group.versions))
const anchorVersion = ref<string | null>(null)

const handleToggleVersion = (version: string) => {
	const flat = allVersionsFlat.value

	if (holdingShift.value && anchorVersion.value && flat.length) {
		const anchorIdx = flat.indexOf(anchorVersion.value)
		const targetIdx = flat.indexOf(version)

		if (anchorIdx === -1 || targetIdx === -1) {
			return toggleVersion(version)
		}

		const start = Math.min(anchorIdx, targetIdx)
		const end = Math.max(anchorIdx, targetIdx)
		const range = flat.slice(start, end + 1)

		const isTargetSelected = props.modelValue.includes(version)
		if (isTargetSelected) {
			emit(
				'update:modelValue',
				props.modelValue.filter((v) => !range.includes(v)),
			)
		} else {
			const newVersions = range.filter((v) => !props.modelValue.includes(v))
			emit('update:modelValue', [...props.modelValue, ...newVersions])
		}

		anchorVersion.value = ''
		return
	}

	toggleVersion(version)
	anchorVersion.value = version
}

const toggleVersion = (version: string) => {
	const isSelected = props.modelValue.includes(version)
	const next = isSelected
		? props.modelValue.filter((v) => v !== version)
		: [...props.modelValue, version]

	emit('update:modelValue', next)
}

const DEV_RELEASE_KEY = 'Snapshots'

function groupVersions(gameVersions: GameVersion[]) {
	gameVersions = [...gameVersions].sort(
		(a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
	)

	const getGroupKey = (v: string) => v.split('.').slice(0, 2).join('.')
	const groups: Record<string, string[]> = {}

	let currentGroupKey = getGroupKey(gameVersions.find((v) => v.major)?.version || '')

	gameVersions.forEach((gameVersion) => {
		if (gameVersion.version_type === 'release') {
			currentGroupKey = getGroupKey(gameVersion.version)
			if (!groups[currentGroupKey]) groups[currentGroupKey] = []
			groups[currentGroupKey].push(gameVersion.version)
		} else {
			const key = `${currentGroupKey} ${DEV_RELEASE_KEY}`
			if (!groups[key]) groups[key] = []
			groups[key].push(gameVersion.version)
		}
	})

	const sortedKeys = Object.keys(groups).sort(compareGroupKeys)
	return sortedKeys.map((key) => ({
		key,
		versions: groups[key].sort((a, b) => compareVersions(b, a)),
		isReleaseGroup: !key.includes(DEV_RELEASE_KEY),
	}))
}

const getBaseVersion = (key: string) => key.split(' ')[0]

function compareVersions(a: string, b: string) {
	const pa = a.split('.').map(Number)
	const pb = b.split('.').map(Number)

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

	const isADev = a.includes(DEV_RELEASE_KEY)
	const isBDev = b.includes(DEV_RELEASE_KEY)

	if (isADev && !isBDev) return 1
	if (!isADev && isBDev) return -1

	return 0
}

function searchFilter(gameVersion: Labrinth.Tags.v2.GameVersion) {
	return gameVersion.version.toLowerCase().includes(searchQuery.value.toLowerCase())
}
</script>
