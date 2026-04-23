<script setup lang="ts">
import {
	FileIcon,
	FolderSearchIcon,
	LinkIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import { injectNotificationManager, Slider, StyledInput, Toggle } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { ref, watch } from 'vue'

import useMemorySlider from '@/composables/useMemorySlider'
import { get, set } from '@/helpers/settings.ts'
import type { FileLinkKind } from '@/helpers/types'

const { handleError } = injectNotificationManager()

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = fetchSettings.custom_env_vars.map((x) => x.join('=')).join(' ')
fetchSettings.file_links = fetchSettings.file_links ?? []

const settings = ref(fetchSettings)

const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
		setSettings.custom_env_vars = setSettings.envVars
			.trim()
			.split(/\s+/)
			.filter(Boolean)
			.map((x) => x.split('=').filter(Boolean))
		setSettings.file_links = setSettings.file_links.map(
			(fileLink: { path: string; target: string; kind: FileLinkKind }) => ({
				path: fileLink.path.trim(),
				target: fileLink.target.trim(),
				kind: fileLink.kind,
			}),
		).filter((fileLink: { path: string; target: string }) => fileLink.path && fileLink.target)

		if (!setSettings.hooks.pre_launch) {
			setSettings.hooks.pre_launch = null
		}
		if (!setSettings.hooks.wrapper) {
			setSettings.hooks.wrapper = null
		}
		if (!setSettings.hooks.post_exit) {
			setSettings.hooks.post_exit = null
		}

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)

function addFileLink() {
	settings.value.file_links.push({
		path: '',
		target: '',
		kind: 'directory',
	})
}

function removeFileLink(index: number) {
	settings.value.file_links.splice(index, 1)
}

async function browseTarget(index: number) {
	const fileLink = settings.value.file_links[index]
	if (!fileLink) return

	const result = await open({
		multiple: false,
		directory: fileLink.kind === 'directory',
		title: fileLink.kind === 'directory' ? 'Select a folder to link' : 'Select a file to link',
	})

	if (typeof result === 'string') {
		fileLink.target = result
	}
}
</script>

<template>
	<div>
		<div class="flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h3 class="m-0 text-lg font-semibold text-contrast">Fullscreen</h3>
					<p class="m-0 leading-tight">
						Overwrites the options.txt file to start in full screen when launched.
					</p>
				</div>

				<Toggle id="fullscreen" v-model="settings.force_fullscreen" />
			</div>

			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h3 class="m-0 text-lg font-semibold text-contrast">Width</h3>
					<p class="m-0 leading-tight">The width of the game window when launched.</p>
				</div>

				<StyledInput
					id="width"
					v-model="settings.game_resolution[0]"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					placeholder="Enter width..."
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h3 class="m-0 text-lg font-semibold text-contrast">Height</h3>
					<p class="m-0 leading-tight">The height of the game window when launched.</p>
				</div>

				<StyledInput
					id="height"
					v-model="settings.game_resolution[1]"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					placeholder="Enter height..."
				/>
			</div>
		</div>

		<hr class="my-6 bg-button-border border-none h-[1px]" />

		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">Memory allocated</h2>
				<Slider
					id="max-memory"
					v-model="settings.memory.maximum"
					:min="512"
					:max="maxMemory"
					:step="64"
					:snap-points="snapPoints"
					:snap-range="512"
					unit="MB"
				/>
				<p class="m-0 mt-1 leading-tight">The memory allocated to each instance when it is ran.</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">Java arguments</h2>
				<StyledInput
					id="java-args"
					v-model="settings.launchArgs"
					autocomplete="off"
					type="text"
					placeholder="Enter java arguments..."
					wrapper-class="w-full"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">Environmental variables</h2>
				<StyledInput
					id="env-vars"
					v-model="settings.envVars"
					autocomplete="off"
					type="text"
					placeholder="Enter environmental variables..."
					wrapper-class="w-full"
				/>
			</div>
		</div>

		<hr class="my-6 bg-button-border border-none h-[1px]" />

		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">Pre launch hook</h3>
				<StyledInput
					id="pre-launch"
					v-model="settings.hooks.pre_launch"
					autocomplete="off"
					type="text"
					placeholder="Enter pre-launch command..."
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">Ran before the instance is launched.</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">Wrapper hook</h3>
				<StyledInput
					id="wrapper"
					v-model="settings.hooks.wrapper"
					autocomplete="off"
					type="text"
					placeholder="Enter wrapper command..."
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">Wrapper command for launching Minecraft.</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">Post exit hook</h3>
				<StyledInput
					id="post-exit"
					v-model="settings.hooks.post_exit"
					autocomplete="off"
					type="text"
					placeholder="Enter post-exit command..."
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">Ran after the game closes.</p>
			</div>
		</div>

		<hr class="my-6 bg-button-border border-none h-[1px]" />

		<div class="flex flex-col gap-2.5">
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">File links</h2>
					<p class="m-0 leading-tight">
						Default links are applied when you create a new instance. Use relative paths like
						screenshots or options.txt.
					</p>
				</div>

				<button class="iconified-button brand-button" @click="addFileLink">
					<PlusIcon />
				</button>
			</div>

			<div
				v-if="settings.file_links.length === 0"
				class="rounded-2xl border border-solid border-button-border p-4"
			>
				<div class="flex items-center gap-3 text-secondary">
					<LinkIcon class="size-5" />
					<span>No default file links configured yet.</span>
				</div>
			</div>

			<div v-for="(fileLink, index) in settings.file_links" :key="index" class="file-link-card">
				<div class="flex items-center justify-between gap-2">
					<div class="flex items-center gap-2 text-contrast">
						<LinkIcon class="size-4" />
						<span class="font-semibold">{{ fileLink.path || 'New file link' }}</span>
					</div>
					<button class="iconified-button danger-button" @click="removeFileLink(index)">
						<TrashIcon />
					</button>
				</div>

				<div class="grid gap-4 md:grid-cols-2">
					<div class="flex flex-col gap-2">
						<label class="text-sm font-medium text-secondary">Path in instance</label>
						<StyledInput
							v-model="fileLink.path"
							autocomplete="off"
							type="text"
							placeholder="screenshots"
							wrapper-class="w-full"
						/>
					</div>

					<div class="flex flex-col gap-2">
						<label class="text-sm font-medium text-secondary">Target on this device</label>
						<div class="flex gap-2">
							<StyledInput
								v-model="fileLink.target"
								autocomplete="off"
								type="text"
								placeholder="Select an existing file or folder"
								wrapper-class="min-w-0 flex-1"
							/>
							<button class="iconified-button" @click="browseTarget(index)">
								<FolderSearchIcon />
							</button>
						</div>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<button
						class="btn"
						:class="{ 'btn-brand': fileLink.kind === 'directory' }"
						@click="fileLink.kind = 'directory'"
					>
						<FolderSearchIcon class="size-4" />
						Folder
					</button>
					<button
						class="btn"
						:class="{ 'btn-brand': fileLink.kind === 'file' }"
						@click="fileLink.kind = 'file'"
					>
						<FileIcon class="size-4" />
						File
					</button>
				</div>
			</div>
		</div>
	</div>
</template>

<style scoped lang="scss">
.file-link-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	padding: 1rem;
	border: 1px solid var(--color-button-border);
	border-radius: 1rem;
	background: var(--color-bg-raised);
}

.iconified-button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 2.5rem;
	height: 2.5rem;
	border: 1px solid var(--color-button-border);
	border-radius: 9999px;
	background: transparent;
	color: var(--color-contrast);
	cursor: pointer;
}

.brand-button {
	background: var(--color-brand);
	border-color: var(--color-brand);
	color: var(--color-accent-contrast);
}

.danger-button {
	color: var(--color-red);
}

.btn {
	display: inline-flex;
	align-items: center;
	gap: 0.5rem;
	height: 2.25rem;
	padding: 0 0.875rem;
	border: 1px solid var(--color-button-border);
	border-radius: 9999px;
	background: var(--color-button-bg);
	color: var(--color-button-text);
	cursor: pointer;
}

.btn-brand {
	background: var(--color-brand);
	border-color: var(--color-brand);
	color: var(--color-accent-contrast);
}
</style>
