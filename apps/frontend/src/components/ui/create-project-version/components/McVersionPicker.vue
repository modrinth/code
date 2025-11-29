<template>
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
						:color="modelValue.includes(version) ? 'green' : 'standard'"
						:highlighted="modelValue.includes(version)"
						type="chip"
						size="small"
					>
						<button
							class="w-16 !text-contrast focus:outline-none"
							@click="toggleVersion(version, $event)"
						>
							{{ version }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { computed, ref } from 'vue'

const props = defineProps<{
	modelValue: string[]
	gameVersions: any[]
}>()

const emit = defineEmits<{
	(e: 'update:modelValue', value: string[]): void
}>()

const lastClickedVersion = ref<string | null>(null)
const anchorVersion = ref<string | null>(null)

const releaseVersions = computed(() =>
	props.gameVersions.filter((v) => v.version_type === 'release'),
)

const groupedGameVersions = computed(() =>
	groupVersions(releaseVersions.value.map((v) => v.version)),
)

const allVersionsFlat = computed(() => groupedGameVersions.value.flatMap((g) => g.versions))

const toggleVersion = (version: string, event: MouseEvent) => {
	if (event.shiftKey && anchorVersion.value) {
		const anchorIdx = allVersionsFlat.value.indexOf(anchorVersion.value)
		const endIdx = allVersionsFlat.value.indexOf(version)
		const [minIdx, maxIdx] = anchorIdx <= endIdx ? [anchorIdx, endIdx] : [endIdx, anchorIdx]
		const rangeVersions = allVersionsFlat.value.slice(minIdx, maxIdx + 1)

		const next = Array.from(new Set([...props.modelValue, ...rangeVersions])).sort((a, b) =>
			compareVersions(b, a),
		)

		emit('update:modelValue', next)
	} else {
		const next = props.modelValue.includes(version)
			? props.modelValue.filter((v) => v !== version)
			: [...props.modelValue, version]

		emit('update:modelValue', next)
		anchorVersion.value = version
	}

	lastClickedVersion.value = version
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
