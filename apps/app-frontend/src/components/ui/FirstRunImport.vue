<template>
	<div
		class="fixed inset-0 z-[190] flex min-h-screen bg-bg text-contrast"
		data-tauri-drag-region
	>
		<div class="m-auto flex h-full w-full max-w-[1180px] flex-col px-6 py-8 md:px-10">
			<header class="flex shrink-0 items-center justify-between gap-4" data-tauri-drag-region>
				<div class="flex items-center gap-3">
					<div
						class="flex size-12 items-center justify-center rounded-2xl bg-surface-2 shadow-lg"
					>
						<IcarusIcon class="size-8" />
					</div>
					<div>
						<p class="m-0 text-sm font-semibold text-secondary">Welcome to Icarus Launcher</p>
						<h1 class="m-0 text-2xl font-extrabold text-contrast md:text-3xl">
							Do you want to import your instances?
						</h1>
					</div>
				</div>
				<ButtonStyled type="transparent">
					<button :disabled="busy" @click="skip">
						Skip
					</button>
				</ButtonStyled>
			</header>

			<section class="grid min-h-0 flex-1 grid-cols-1 gap-8 py-8 lg:grid-cols-[360px_1fr]">
				<aside
					class="flex min-h-0 flex-col justify-between gap-6 rounded-[24px] border border-solid border-surface-4 bg-surface-1 p-6"
					data-tauri-drag-region
				>
					<div>
						<p class="m-0 text-base leading-7 text-secondary">
							Bring your Minecraft instances from another launcher into Icarus. Pick the
							instances you want to import, or skip this step and do it later.
						</p>
						<div class="mt-6 grid grid-cols-3 gap-3">
							<div
								v-for="launcher in launchers"
								:key="launcher.name"
								class="flex aspect-square items-center justify-center rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4"
								:class="{ 'opacity-45': !detectedLauncherNames.has(launcher.name) }"
								:title="launcher.label"
							>
								<component
									:is="launcher.component"
									v-if="launcher.component"
									class="max-h-full max-w-full"
								/>
								<img
									v-else
									:src="launcher.icon"
									:alt="`${launcher.label} logo`"
									class="max-h-full max-w-full object-contain"
									draggable="false"
								/>
							</div>
						</div>
					</div>
					<div class="rounded-2xl bg-surface-2 p-4">
						<p class="m-0 text-sm font-semibold text-contrast">{{ totalSelectedCount }} selected</p>
						<p class="m-0 mt-1 text-sm text-secondary">
							Selected instances will be copied into your Icarus library.
						</p>
					</div>
				</aside>

				<main
					class="flex min-h-0 flex-col rounded-[24px] border border-solid border-surface-4 bg-bg-raised shadow-xl"
				>
					<div class="flex shrink-0 items-center justify-between gap-4 border-0 border-b border-solid border-surface-4 p-5">
						<div>
							<h2 class="m-0 text-lg font-bold text-contrast">Importable instances</h2>
							<p class="m-0 mt-1 text-sm text-secondary">
								{{ loading ? 'Scanning installed launchers...' : scanSummary }}
							</p>
						</div>
						<ButtonStyled type="outlined">
							<button :disabled="loading || busy" @click="loadLaunchers">
								<RefreshCwIcon :class="{ 'animate-spin': loading }" />
								Rescan
							</button>
						</ButtonStyled>
					</div>

					<div class="min-h-0 flex-1 overflow-y-auto p-5">
						<div v-if="loading" class="flex h-full items-center justify-center text-secondary">
							<SpinnerIcon class="mr-3 size-5 animate-spin" />
							Scanning installed launchers...
						</div>

						<div
							v-else-if="detectedLaunchers.length === 0"
							class="flex h-full flex-col items-center justify-center text-center"
						>
							<h3 class="m-0 text-xl font-bold text-contrast">No importable instances found</h3>
							<p class="m-0 mt-2 max-w-[420px] text-sm leading-6 text-secondary">
								You can skip this step now and import instances later from the create instance
								flow.
							</p>
						</div>

						<div v-else class="flex flex-col gap-3">
							<section
								v-for="launcher in detectedLaunchers"
								:key="launcher.name"
								class="overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-1"
							>
								<button
									class="flex w-full cursor-pointer items-center gap-4 border-none bg-surface-2 p-4 text-left text-contrast"
									@click="toggleLauncher(launcher.name)"
								>
									<div class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-surface-3 p-2">
										<component
											:is="getLauncherMeta(launcher.name)?.component"
											v-if="getLauncherMeta(launcher.name)?.component"
											class="max-h-full max-w-full"
										/>
										<img
											v-else
											:src="getLauncherMeta(launcher.name)?.icon"
											:alt="`${launcher.name} logo`"
											class="max-h-full max-w-full object-contain"
											draggable="false"
										/>
									</div>
									<div class="min-w-0 flex-1">
										<p class="m-0 truncate font-bold">{{ getLauncherMeta(launcher.name)?.label ?? launcher.name }}</p>
										<p class="m-0 mt-1 truncate text-sm text-secondary">
											{{ launcher.instances.length }} importable
											{{ launcher.instances.length === 1 ? 'instance' : 'instances' }}
										</p>
									</div>
									<Checkbox
										:model-value="isLauncherSelected(launcher)"
										:indeterminate="isLauncherIndeterminate(launcher)"
										@update:model-value="setLauncherSelected(launcher, $event)"
										@click.stop
									/>
								</button>
								<div class="grid grid-cols-1 gap-2 p-3 md:grid-cols-2">
									<button
										v-for="instance in launcher.instances"
										:key="instance"
										class="flex min-h-[48px] items-center gap-3 rounded-xl border border-solid p-3 text-left transition-colors"
										:class="
											isInstanceSelected(launcher.name, instance)
												? 'border-brand bg-brand-highlight text-contrast'
												: 'border-surface-4 bg-surface-2 text-contrast hover:bg-surface-3'
										"
										@click="toggleInstance(launcher.name, instance)"
									>
										<Checkbox
											:model-value="isInstanceSelected(launcher.name, instance)"
											@update:model-value="setInstanceSelected(launcher.name, instance, $event)"
											@click.stop
										/>
										<span class="min-w-0 truncate text-sm font-semibold">{{ instance }}</span>
									</button>
								</div>
							</section>
						</div>
					</div>

					<footer class="flex shrink-0 items-center justify-end gap-3 border-0 border-t border-solid border-surface-4 p-5">
						<ButtonStyled type="transparent">
							<button :disabled="busy" @click="skip">
								Skip
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button :disabled="totalSelectedCount === 0 || busy" @click="importSelected">
								<SpinnerIcon v-if="busy" class="animate-spin" />
								<DownloadIcon v-else />
								Import
							</button>
						</ButtonStyled>
					</footer>
				</main>
			</section>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CurseForgeIcon,
	DownloadIcon,
	IcarusIcon,
	RefreshCwIcon,
	SpinnerIcon,
} from '@icarus/assets'
import { ButtonStyled, Checkbox, injectInstanceImport, injectNotificationManager } from '@icarus/ui'
import { computed, onMounted, ref, type Component } from 'vue'

import ATLauncherIcon from '@/assets/external/atlauncher.svg?url'
import GDLauncherIcon from '@/assets/external/gdlauncher.png?url'
import MultiMCIcon from '@/assets/external/multimc.webp?url'
import PrismIcon from '@/assets/external/prism.svg?url'

import type { ImportableLauncher } from '@icarus/ui'

const emit = defineEmits<{
	complete: []
}>()

interface LauncherMeta {
	name: string
	label: string
	icon?: string
	component?: Component
}

const launchers: LauncherMeta[] = [
	{ name: 'ModrinthApp', label: 'Modrinth App', component: IcarusIcon },
	{ name: 'PrismLauncher', label: 'Prism Launcher', icon: PrismIcon },
	{ name: 'MultiMC', label: 'MultiMC', icon: MultiMCIcon },
	{ name: 'Curseforge', label: 'CurseForge', component: CurseForgeIcon },
	{ name: 'ATLauncher', label: 'ATLauncher', icon: ATLauncherIcon },
	{ name: 'GDLauncher', label: 'GDLauncher', icon: GDLauncherIcon },
]

const importProvider = injectInstanceImport()
const { handleError } = injectNotificationManager()

const loading = ref(false)
const busy = ref(false)
const detectedLaunchers = ref<ImportableLauncher[]>([])
const selectedInstances = ref<Record<string, Set<string>>>({})

const detectedLauncherNames = computed(() => new Set(detectedLaunchers.value.map((x) => x.name)))

const totalSelectedCount = computed(() => {
	let count = 0
	for (const instances of Object.values(selectedInstances.value)) {
		count += instances.size
	}
	return count
})

const scanSummary = computed(() => {
	const count = detectedLaunchers.value.length
	if (count === 0) return 'No launchers with importable instances were found.'
	return `${count} ${count === 1 ? 'launcher' : 'launchers'} found.`
})

onMounted(loadLaunchers)

async function loadLaunchers() {
	loading.value = true
	selectedInstances.value = {}

	try {
		detectedLaunchers.value = await importProvider.getDetectedLaunchers()
	} catch (err) {
		handleError(err as Error)
		detectedLaunchers.value = []
	}

	loading.value = false
}

function getLauncherMeta(name: string) {
	return launchers.find((launcher) => launcher.name === name)
}

function ensureLauncherSelection(launcherName: string) {
	if (!selectedInstances.value[launcherName]) {
		selectedInstances.value[launcherName] = new Set()
	}
}

function setInstanceSelected(launcherName: string, instance: string, selected: boolean) {
	ensureLauncherSelection(launcherName)

	if (selected) {
		selectedInstances.value[launcherName].add(instance)
	} else {
		selectedInstances.value[launcherName].delete(instance)
	}

	selectedInstances.value = { ...selectedInstances.value }
}

function toggleInstance(launcherName: string, instance: string) {
	setInstanceSelected(launcherName, instance, !isInstanceSelected(launcherName, instance))
}

function isInstanceSelected(launcherName: string, instance: string) {
	return selectedInstances.value[launcherName]?.has(instance) ?? false
}

function setLauncherSelected(launcher: ImportableLauncher, selected: boolean) {
	ensureLauncherSelection(launcher.name)

	for (const instance of launcher.instances) {
		if (selected) {
			selectedInstances.value[launcher.name].add(instance)
		} else {
			selectedInstances.value[launcher.name].delete(instance)
		}
	}

	selectedInstances.value = { ...selectedInstances.value }
}

function toggleLauncher(launcherName: string) {
	const launcher = detectedLaunchers.value.find((x) => x.name === launcherName)
	if (!launcher) return

	setLauncherSelected(launcher, !isLauncherSelected(launcher))
}

function isLauncherSelected(launcher: ImportableLauncher) {
	const selected = selectedInstances.value[launcher.name]
	return !!selected && launcher.instances.length > 0 && launcher.instances.every((x) => selected.has(x))
}

function isLauncherIndeterminate(launcher: ImportableLauncher) {
	const selected = selectedInstances.value[launcher.name]
	if (!selected || selected.size === 0) return false

	const selectedCount = launcher.instances.filter((x) => selected.has(x)).length
	return selectedCount > 0 && selectedCount < launcher.instances.length
}

async function importSelected() {
	if (totalSelectedCount.value === 0 || busy.value) return

	busy.value = true

	try {
		await importProvider.importInstances(
			Object.entries(selectedInstances.value)
				.map(([launcherName, instances]) => {
					const launcher = detectedLaunchers.value.find((x) => x.name === launcherName)
					if (!launcher) return null

					return {
						launcher: launcher.name,
						path: launcher.path,
						instanceNames: Array.from(instances),
					}
				})
				.filter((selection): selection is {
					launcher: string
					path: string
					instanceNames: string[]
				} => !!selection && selection.instanceNames.length > 0),
		)
		emit('complete')
	} catch (err) {
		handleError(err as Error)
	}

	busy.value = false
}

function skip() {
	if (busy.value) return
	emit('complete')
}
</script>
