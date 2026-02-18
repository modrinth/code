<template>
	<div class="flex flex-col gap-4">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">Launcher instances</span>
			<button
				class="bg-transparent p-0 text-sm text-secondary hover:text-contrast transition-colors"
				:disabled="totalSelectedCount === 0"
				@click="clearAll"
			>
				Clear all
			</button>
		</div>

		<!-- Search -->
		<StyledInput
			v-model="ctx.importSearchQuery.value"
			:icon="SearchIcon"
			placeholder="Search instance name"
		/>

		<!-- Launcher sections -->
		<div
			v-if="ctx.importLaunchers.value.length > 0"
			class="flex flex-col rounded-xl border border-solid border-surface-4 overflow-hidden"
		>
			<div
				v-for="(launcher, index) in ctx.importLaunchers.value"
				:key="launcher.name"
				class="flex flex-col"
				:class="{ 'border-t border-solid border-surface-4': index > 0 }"
			>
				<!-- Launcher header -->
				<button
					class="flex w-full items-center gap-3 bg-transparent p-3 text-left hover:bg-surface-3 transition-colors"
					@click="toggleLauncherExpanded(launcher.name)"
				>
					<ChevronRightIcon
						class="size-4 shrink-0 text-secondary transition-transform"
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
				<div v-if="expandedLaunchers.has(launcher.name)" class="flex flex-col">
					<template v-for="instance in filteredInstances(launcher)" :key="instance">
						<div
							class="flex items-center gap-3 py-2 pl-10 pr-3 hover:bg-surface-3 transition-colors"
						>
							<Checkbox
								:model-value="isInstanceSelected(launcher.name, instance)"
								@update:model-value="toggleInstance(launcher.name, instance, $event)"
							/>
							<span class="text-sm text-contrast">{{ instance }}</span>
						</div>
					</template>
				</div>
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
			<ButtonStyled type="outlined">
				<button class="w-full !border-surface-4" @click="showAddPath = true">
					Add launcher path
				</button>
			</ButtonStyled>
		</div>
		<div v-else class="flex items-center gap-2">
			<FolderOpenIcon class="size-5 shrink-0 text-secondary" />
			<StyledInput v-model="newLauncherPath" placeholder="Path to launcher..." class="flex-1" />
			<ButtonStyled>
				<button :disabled="!newLauncherPath.trim()" @click="addLauncherPath">Add</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, FolderOpenIcon, SearchIcon } from '@modrinth/assets'
import { computed, onMounted, ref } from 'vue'

import { injectInstanceImport } from '../../../../providers'
import type { ImportableLauncher } from '../../../../providers/instance-import'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Checkbox from '../../../base/Checkbox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const importProvider = injectInstanceImport()

const loading = ref(false)
const expandedLaunchers = ref(new Set<string>())
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
