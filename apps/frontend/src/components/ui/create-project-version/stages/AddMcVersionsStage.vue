<template>
	<div class="flex w-full max-w-[496px] flex-col gap-6">
		<div class="space-y-2">
			<span class="font-semibold text-contrast">
				Minecraft versions <span class="text-red">*</span>
			</span>
			<div
				class="flex max-h-60 flex-col gap-3 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div v-for="group in groupedGameVersions" :key="group.key" class="space-y-1.5">
					<span class="text-sm font-semibold">{{ group.key }}</span>
					<div class="flex flex-wrap gap-2 gap-x-1.5">
						<ButtonStyled
							v-for="version in group.versions"
							:key="version"
							:color="selectedVersions.includes(version) ? 'green' : 'standard'"
							:highlighted="selectedVersions.includes(version)"
							type="chip"
							size="small"
						>
							<button class="w-16 !text-contrast" @click="toggleVersion(version)">
								{{ version }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Added versions </span>
				<ButtonStyled type="transparent" size="standard">
					<button @click="clearAllVersions()">Clear all</button>
				</ButtonStyled>
			</div>
			<div
				class="flex flex-col gap-1.5 gap-y-2 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div class="space-y-2">
					<span>Detected</span>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled
							v-for="version in detectedVersions"
							:key="version"
							type="chip"
							size="small"
						>
							<button class="w-20 !text-contrast" @click="toggleVersion(version)">
								{{ version }}
								<XIcon />
							</button>
						</ButtonStyled>
					</div>
				</div>
				<div class="space-y-2">
					<span>Selected</span>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled
							v-for="version in selectedVersions"
							:key="version"
							type="chip"
							size="small"
						>
							<button class="w-20 !text-contrast" @click="toggleVersion(version)">
								{{ version }}
								<XIcon />
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { XIcon } from '@modrinth/assets'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { computed } from 'vue'

const selectedVersions = ref<string[]>([])
const detectedVersions = ['1.18.2', '1.19', '1.19.1']

const generatedState = useGeneratedState()
const gameVersions = generatedState.value.gameVersions

const releaseVersions = gameVersions.filter((v) => v.version_type === 'release')

const groupedGameVersions = computed(() => groupVersions(releaseVersions.map((v) => v.version)))

const toggleVersion = (version: string) => {
	if (selectedVersions.value.includes(version)) {
		selectedVersions.value = selectedVersions.value.filter((v) => v !== version)
	} else {
		selectedVersions.value.push(version)
	}
}

const clearAllVersions = () => {
	selectedVersions.value = []
}

// Create group keys like: "1.20.4" → "1.20"
function groupVersions(versions: string[]) {
	const getGroupKey = (v: string) => v.split('.').slice(0, 2).join('.')
	const groups: Record<string, string[]> = {}

	versions.forEach((version) => {
		const groupKey = getGroupKey(version)
		if (!groups[groupKey]) groups[groupKey] = []
		groups[groupKey].push(version)
	})

	const sortedKeys = Object.keys(groups).sort((a, b) => compareVersions(b, a))
	const result = sortedKeys.map((key) => ({
		key,
		versions: groups[key].sort((a, b) => compareVersions(b, a)),
	}))

	return result
}

function compareVersions(a: string, b: string) {
	const pa = a.split('.').map(Number)
	const pb = b.split('.').map(Number)

	for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
		const na = pa[i] || 0
		const nb = pb[i] || 0
		if (na > nb) return 1
		if (na < nb) return -1
	}
	return 0
}
</script>
