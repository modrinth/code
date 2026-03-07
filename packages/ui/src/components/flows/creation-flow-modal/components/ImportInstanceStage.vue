<template>
	<div class="flex flex-col gap-4">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">Launcher instances</span>
			<ButtonStyled
				type="transparent"
				size="standard"
				:class="{ invisible: totalSelectedCount === 0 }"
			>
				<button @click="clearAll">Clear all</button>
			</ButtonStyled>
		</div>

		<!-- Search -->
		<StyledInput
			v-model="ctx.importSearchQuery.value"
			:icon="SearchIcon"
			placeholder="Search instance name"
		/>

		<!-- Launcher sections -->
		<div v-if="ctx.importLaunchers.value.length > 0" class="flex flex-col gap-2">
			<div
				v-for="launcher in visibleLaunchers"
				:key="launcher.name"
				class="flex flex-col rounded-[20px] border border-solid border-surface-4 shadow-sm overflow-clip"
			>
				<!-- Launcher header -->
				<button
					class="flex w-full cursor-pointer items-center gap-3 border-none bg-surface-3 p-3 text-left transition-colors"
					@click="toggleLauncherExpanded(launcher.name)"
				>
					<ChevronRightIcon
						class="size-5 shrink-0 text-secondary transition-transform"
						:class="{ 'rotate-90': expandedLaunchers.has(launcher.name) }"
					/>
					<Checkbox
						:model-value="getLauncherCheckState(launcher)"
						:indeterminate="getLauncherIndeterminate(launcher)"
						@update:model-value="toggleLauncherAll(launcher, $event)"
						@click.stop
					/>
					<span class="font-semibold text-contrast">{{ launcher.name }}</span>
				</button>

				<!-- Instance list (expanded) -->
				<Collapsible :collapsed="!expandedLaunchers.has(launcher.name)">
					<div class="flex flex-col">
						<template v-for="(instance, i) in filteredInstances(launcher)" :key="instance">
							<div
								class="flex items-center gap-3 border-0 border-t border-solid border-surface-4 py-3 pr-3"
								:class="i % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
								style="padding-left: 2.75rem"
							>
								<Checkbox
									:model-value="isInstanceSelected(launcher.name, instance)"
									@update:model-value="toggleInstance(launcher.name, instance, $event)"
								/>
								<span class="text-sm">{{ instance }}</span>
							</div>
						</template>
					</div>
				</Collapsible>
			</div>
		</div>

		<!-- Empty state -->
		<div v-else-if="!loading" class="flex items-center justify-center py-8 text-secondary text-sm">
			No launcher instances detected
		</div>

		<!-- Loading state -->
		<div v-if="loading" class="flex items-center justify-center py-8 text-secondary text-sm">
			Detecting launcher instances...
		</div>

		<!-- Add launcher path -->
		<div v-if="!showAddPath">
			<ButtonStyled>
				<button class="w-full !shadow-none" @click="showAddPath = true">Add launcher path</button>
			</ButtonStyled>
		</div>
		<div v-else class="flex items-center gap-2">
			<ButtonStyled icon-only
				><button class="!shadow-none" @click="browseForLauncherPath">
					<FolderSearchIcon class="size-5" /></button
			></ButtonStyled>
			<StyledInput v-model="newLauncherPath" placeholder="Path to launcher..." class="flex-1" />
			<ButtonStyled>
				<button class="!shadow-none" :disabled="!newLauncherPath.trim()" @click="addLauncherPath">
					Add
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, FolderSearchIcon, SearchIcon } from '@modrinth/assets'
import { computed, onMounted, ref, watch } from 'vue'

import { injectInstanceImport } from '../../../../providers'
import type { ImportableLauncher } from '../../../../providers/instance-import'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Checkbox from '../../../base/Checkbox.vue'
import Collapsible from '../../../base/Collapsible.vue'
import StyledInput from '../../../base/StyledInput.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const importProvider = injectInstanceImport()

const loading = ref(false)
const expandedLaunchers = ref(new Set<string>())
const expandedBeforeSearch = ref<Set<string> | null>(null)
const showAddPath = ref(false)
const newLauncherPath = ref('')

// Load detected launchers on mount
onMounted(async () => {
	if (ctx.importLaunchers.value.length > 0) return // Already loaded

	loading.value = true
	try {
		ctx.importLaunchers.value = await importProvider.getDetectedLaunchers()
		// Auto-expand launchers that have instances
		for (const launcher of ctx.importLaunchers.value) {
			if (launcher.instances.length > 0) {
				expandedLaunchers.value.add(launcher.name)
			}
		}
	} catch {
		ctx.importLaunchers.value = []
	}
	loading.value = false
})

// Filter instances by search query
function filteredInstances(launcher: ImportableLauncher): string[] {
	const query = ctx.importSearchQuery.value.toLowerCase().trim()
	if (!query) return launcher.instances
	return launcher.instances.filter((name) => name.toLowerCase().includes(query))
}

// Hide launchers with no matching instances when searching
const visibleLaunchers = computed(() => {
	const query = ctx.importSearchQuery.value.toLowerCase().trim()
	if (!query) return ctx.importLaunchers.value
	return ctx.importLaunchers.value.filter((launcher) => filteredInstances(launcher).length > 0)
})

// Auto-expand launchers with matching results when searching
watch(
	() => ctx.importSearchQuery.value,
	(query) => {
		const trimmed = query.trim()
		if (trimmed) {
			// Save current state before search overrides it
			if (!expandedBeforeSearch.value) {
				expandedBeforeSearch.value = new Set(expandedLaunchers.value)
			}
			// Expand all launchers that have matching instances
			const newExpanded = new Set(expandedLaunchers.value)
			for (const launcher of ctx.importLaunchers.value) {
				if (filteredInstances(launcher).length > 0) {
					newExpanded.add(launcher.name)
				}
			}
			expandedLaunchers.value = newExpanded
		} else if (expandedBeforeSearch.value) {
			// Restore pre-search state
			expandedLaunchers.value = expandedBeforeSearch.value
			expandedBeforeSearch.value = null
		}
	},
)

// Selection helpers
function isInstanceSelected(launcherName: string, instance: string): boolean {
	return ctx.importSelectedInstances.value[launcherName]?.has(instance) ?? false
}

function toggleInstance(launcherName: string, instance: string, selected: boolean) {
	if (!ctx.importSelectedInstances.value[launcherName]) {
		ctx.importSelectedInstances.value[launcherName] = new Set()
	}
	if (selected) {
		ctx.importSelectedInstances.value[launcherName].add(instance)
	} else {
		ctx.importSelectedInstances.value[launcherName].delete(instance)
	}
	// Trigger reactivity
	ctx.importSelectedInstances.value = { ...ctx.importSelectedInstances.value }
}

function getLauncherCheckState(launcher: ImportableLauncher): boolean {
	const set = ctx.importSelectedInstances.value[launcher.name]
	if (!set || set.size === 0) return false
	const visible = filteredInstances(launcher)
	return visible.length > 0 && visible.every((i) => set.has(i))
}

function getLauncherIndeterminate(launcher: ImportableLauncher): boolean {
	const set = ctx.importSelectedInstances.value[launcher.name]
	if (!set || set.size === 0) return false
	const visible = filteredInstances(launcher)
	const selectedVisible = visible.filter((i) => set.has(i))
	return selectedVisible.length > 0 && selectedVisible.length < visible.length
}

function toggleLauncherAll(launcher: ImportableLauncher, selected: boolean) {
	if (!ctx.importSelectedInstances.value[launcher.name]) {
		ctx.importSelectedInstances.value[launcher.name] = new Set()
	}
	const visible = filteredInstances(launcher)
	for (const instance of visible) {
		if (selected) {
			ctx.importSelectedInstances.value[launcher.name].add(instance)
		} else {
			ctx.importSelectedInstances.value[launcher.name].delete(instance)
		}
	}
	// Trigger reactivity
	ctx.importSelectedInstances.value = { ...ctx.importSelectedInstances.value }
}

function toggleLauncherExpanded(name: string) {
	if (expandedLaunchers.value.has(name)) {
		expandedLaunchers.value.delete(name)
	} else {
		expandedLaunchers.value.add(name)
	}
	expandedLaunchers.value = new Set(expandedLaunchers.value)
}

const totalSelectedCount = computed(() => {
	let count = 0
	for (const set of Object.values(ctx.importSelectedInstances.value)) {
		count += set.size
	}
	return count
})

function clearAll() {
	ctx.importSelectedInstances.value = {}
}

async function browseForLauncherPath() {
	const path = await importProvider.selectDirectory()
	if (path) {
		newLauncherPath.value = path
	}
}

async function addLauncherPath() {
	const path = newLauncherPath.value.trim()
	if (!path) return

	try {
		const instances = await importProvider.getImportableInstances('Custom', path)
		const launcher: ImportableLauncher = {
			name: `Custom (${path.split(/[\\/]/).pop() || path})`,
			path,
			instances,
		}
		ctx.importLaunchers.value = [...ctx.importLaunchers.value, launcher]
		expandedLaunchers.value.add(launcher.name)
		expandedLaunchers.value = new Set(expandedLaunchers.value)
	} catch {
		// Failed to load — still add with empty instances
		const launcher: ImportableLauncher = {
			name: `Custom (${path.split(/[\\/]/).pop() || path})`,
			path,
			instances: [],
		}
		ctx.importLaunchers.value = [...ctx.importLaunchers.value, launcher]
	}

	newLauncherPath.value = ''
	showAddPath.value = false
}
</script>
