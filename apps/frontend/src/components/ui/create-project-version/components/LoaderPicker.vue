<template>
	<div class="space-y-6">
		<div class="flex flex-col gap-2.5">
			<span class="font-semibold text-contrast">Loaders <span class="text-red">*</span></span>

			<Chips
				v-model="loaderGroup"
				:items="groupLabels"
				:never-empty="false"
				:capitalize="true"
				size="small"
			/>

			<div
				class="flex flex-1 flex-col gap-4 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3"
			>
				<div v-if="groupedLoaders[loaderGroup].length" class="flex flex-col gap-1.5">
					<div class="flex flex-wrap gap-2">
						<ButtonStyled
							v-for="loader in groupedLoaders[loaderGroup]"
							:key="`loader-${loader.name}`"
							:color="selectedLoaders.includes(loader.name) ? 'green' : undefined"
							:highlighted="selectedLoaders.includes(loader.name)"
							type="chip"
						>
							<button
								:style="`--_icon: var(--color-platform-${loader.name}); color: var(--color-platform-${loader.name})`"
								@click="toggleLoader(loader.name)"
							>
								<div v-html="loader.icon"></div>
								{{ formatCategory(loader.name) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<span>Select one or more loaders this version supports.</span>
		</div>

		<div v-if="selectedLoaders.length">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Added loaders </span>
				<ButtonStyled type="transparent" size="standard">
					<button @click="onClearAll()">Clear all</button>
				</ButtonStyled>
			</div>
			<div
				class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<span class="font-medium">Selected</span>
				<div class="flex flex-wrap gap-2">
					<template v-if="selectedLoaders.length">
						<template
							v-for="loader in selectedLoaders.map((selectedLoader) =>
								loaders.find((loader) => selectedLoader === loader.name),
							)"
						>
							<ButtonStyled v-if="loader" :key="`loader-${loader.name}`" type="chip">
								<button
									:style="`--_icon: var(--color-platform-${loader.name}); color: var(--color-platform-${loader.name})`"
									@click="toggleLoader(loader.name)"
								>
									<div v-html="loader.icon"></div>
									{{ formatCategory(loader.name) }}
								</button>
							</ButtonStyled>
						</template>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { Chips } from '@modrinth/ui'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { formatCategory } from '@modrinth/utils'

const selectedLoaders = defineModel<string[]>({ default: [] })

const { loaders } = defineProps<{
	loaders: Labrinth.Tags.v2.Loader[]
}>()

const loaderGroup = ref<GroupLabels>('mods')

const toggleLoader = (loader: string) => {
	if (selectedLoaders.value.includes(loader)) {
		selectedLoaders.value = selectedLoaders.value.filter((l) => l !== loader)
	} else {
		selectedLoaders.value = [...selectedLoaders.value, loader]
	}
}

const onClearAll = () => (selectedLoaders.value = [])

type GroupLabels = 'mods' | 'plugins' | 'packs' | 'shaders' | 'other'

const groupLabels: GroupLabels[] = ['mods', 'plugins', 'packs', 'shaders']

function groupLoaders(loaders: Labrinth.Tags.v2.Loader[]) {
	const groups: Record<GroupLabels, Labrinth.Tags.v2.Loader[]> = {
		mods: [],
		plugins: [],
		packs: [],
		shaders: [],
		other: [],
	}

	const MOD_SORT = [
		'fabric',
		'neoforge',
		'forge',
		'quilt',
		'liteloader',
		'rift',
		'ornithe',
		'nilloader',
		'risugami',
		'legacy-fabric',
		'bta-babric',
		'babric',
		'modloader',
		'java-agent',
	]

	const PLUGIN_SORT = [
		'paper',
		'purpur',
		'spigot',
		'bukkit',
		'sponge',
		'folia',
		'bungeecord',
		'velocity',
		'waterfall',
		'geyser',
	]

	const SHADER_SORT = ['optifine', 'iris', 'canvas', 'vanilla']
	const PACKS_SORT = ['minecraft', 'datapack']

	for (const loader of loaders) {
		const name = loader.name.toLowerCase()
		if (PACKS_SORT.includes(name)) groups.packs.push(loader)
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

	return groups
}

const groupedLoaders = computed(() => groupLoaders(loaders))
</script>
