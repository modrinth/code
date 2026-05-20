<script setup lang="ts">
import { injectNotificationManager, Slider, StyledInput, Toggle } from '@modrinth/ui'
import { ref, watch } from 'vue'

import useMemorySlider from '@/composables/useMemorySlider'
import { get, set } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = fetchSettings.custom_env_vars.map((x) => x.join('=')).join(' ')

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
	</div>
</template>
