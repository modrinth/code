<template>
	<div class="flex max-h-full flex-col gap-2">
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">Loaders <span class="text-red">*</span></span>
			<ButtonStyled type="transparent" size="standard">
				<button @click="onClearAll()">Clear all</button>
			</ButtonStyled>
		</div>

		<!-- Each group rendered separately when it has loaders -->
		<div
			class="flex flex-1 flex-col gap-4 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3"
		>
			<div v-for="(group, label) in GROUP_LABELS" :key="label">
				<div v-if="groupedLoaders[label].length" class="flex flex-col gap-1.5">
					<span class="text-sm font-medium">{{ group }}</span>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled
							v-for="loader in groupedLoaders[label]"
							:key="`loader-${loader.name}`"
							:color="modelValue.includes(loader.name) ? 'green' : undefined"
							:highlighted="modelValue.includes(loader.name)"
							type="chip"
							size="small"
						>
							<button
								:style="`color: var(--color-platform-${loader.name})`"
								@click="toggleLoader(loader.name)"
							>
								<div v-html="loader.icon"></div>
								{{ formatCategory(loader.name) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { formatCategory } from '@modrinth/utils'

const props = defineProps<{
	modelValue: string[]
	loaders: Labrinth.Tags.v2.Loader[]
}>()

const emit = defineEmits<{
	(e: 'update:modelValue', value: string[]): void
}>()

const toggleLoader = (loader: string) => {
	const next = props.modelValue.includes(loader)
		? props.modelValue.filter((l) => l !== loader)
		: [...props.modelValue, loader]

	emit('update:modelValue', next)
}

const onClearAll = () => emit('update:modelValue', [])

// --- Labels for UI ---
const GROUP_LABELS = {
	mods: 'Mod loaders',
	plugins: 'Plugin loaders',
	shaders: 'Shader loaders',
	resourcepacks: 'Resource Packs',
	datapacks: 'Datapacks',
	other: 'Other',
}

// --- Grouping logic ---
function groupLoaders(loaders: Labrinth.Tags.v2.Loader[]) {
	const groups = {
		mods: [] as Labrinth.Tags.v2.Loader[],
		plugins: [] as Labrinth.Tags.v2.Loader[],
		shaders: [] as Labrinth.Tags.v2.Loader[],
		resourcepacks: [] as Labrinth.Tags.v2.Loader[],
		datapacks: [] as Labrinth.Tags.v2.Loader[],
		other: [] as Labrinth.Tags.v2.Loader[],
	}

	const MOD_SORT = [
		'fabric',
		'neoforge',
		'forge',
		'quilt',
		'bta-babric',
		'babric',
		'legacy-fabric',
		'risugami',
		'liteloader',
		'rift',
		'ornithe',
		'java-agent',
		'nilloader',
		'modloader',
	]

	const PLUGIN_SORT = [
		'paper',
		'purpur',
		'spigot',
		'bukkit',
		'sponge',
		'geyser',
		'bungeecord',
		'velocity',
		'waterfall',
		'folia',
	]

	const SHADER_SORT = ['optifine', 'iris', 'canvas', 'vanilla']
	const RESOURCEPACK_SORT = ['minecraft']
	const DATAPACK_SORT = ['datapack']

	for (const loader of loaders) {
		const name = loader.name.toLowerCase()
		if (DATAPACK_SORT.includes(name)) groups.datapacks.push(loader)
		else if (RESOURCEPACK_SORT.includes(name)) groups.resourcepacks.push(loader)
		else if (SHADER_SORT.includes(name)) groups.shaders.push(loader)
		else if (PLUGIN_SORT.includes(name)) groups.plugins.push(loader)
		else if (MOD_SORT.includes(name)) groups.mods.push(loader)
		else groups.other.push(loader)
	}

	const sortByOrder = (arr: any[], order: string[]) =>
		arr.sort((a, b) => order.indexOf(a.name) - order.indexOf(b.name))

	sortByOrder(groups.mods, MOD_SORT)
	sortByOrder(groups.plugins, PLUGIN_SORT)
	sortByOrder(groups.shaders, SHADER_SORT)
	sortByOrder(groups.resourcepacks, RESOURCEPACK_SORT)
	sortByOrder(groups.datapacks, DATAPACK_SORT)

	return groups
}

const groupedLoaders = computed(() => groupLoaders(props.loaders))
</script>
