<script setup lang="ts">
import {
	defineMessages,
	injectNotificationManager,
	Slider,
	StyledInput,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import useMemorySlider from '@/composables/useMemorySlider'
import { get, set } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	fullscreenTitle: {
		id: 'app.settings.default-instance-options.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'app.settings.default-instance-options.fullscreen.description',
		defaultMessage: 'Start instances in fullscreen by updating their options.txt file.',
	},
	widthTitle: {
		id: 'app.settings.default-instance-options.width.title',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'app.settings.default-instance-options.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'app.settings.default-instance-options.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: {
		id: 'app.settings.default-instance-options.height.title',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'app.settings.default-instance-options.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'app.settings.default-instance-options.height.placeholder',
		defaultMessage: 'Enter height...',
	},
	memoryAllocationTitle: {
		id: 'app.settings.default-instance-options.memory-allocation.title',
		defaultMessage: 'Memory allocation',
	},
	memoryAllocationDescription: {
		id: 'app.settings.default-instance-options.memory-allocation.description',
		defaultMessage: 'Maximum memory available to each instance.',
	},
	javaArgumentsTitle: {
		id: 'app.settings.default-instance-options.java-arguments.title',
		defaultMessage: 'Java arguments',
	},
	javaArgumentsPlaceholder: {
		id: 'app.settings.default-instance-options.java-arguments.placeholder',
		defaultMessage: 'Enter Java arguments...',
	},
	javaArgumentsDescription: {
		id: 'app.settings.default-instance-options.java-arguments.description',
		defaultMessage: 'Arguments passed to Java when launching an instance.',
	},
	environmentVariablesTitle: {
		id: 'app.settings.default-instance-options.environment-variables.title',
		defaultMessage: 'Environment variables',
	},
	environmentVariablesPlaceholder: {
		id: 'app.settings.default-instance-options.environment-variables.placeholder',
		defaultMessage: 'Enter environment variables...',
	},
	environmentVariablesDescription: {
		id: 'app.settings.default-instance-options.environment-variables.description',
		defaultMessage: 'Environment variables set when launching an instance.',
	},
	preLaunchHookTitle: {
		id: 'app.settings.default-instance-options.pre-launch-hook.title',
		defaultMessage: 'Pre-launch hook',
	},
	preLaunchHookPlaceholder: {
		id: 'app.settings.default-instance-options.pre-launch-hook.placeholder',
		defaultMessage: 'Enter pre-launch command...',
	},
	preLaunchHookDescription: {
		id: 'app.settings.default-instance-options.pre-launch-hook.description',
		defaultMessage: 'Runs before the instance starts.',
	},
	wrapperHookTitle: {
		id: 'app.settings.default-instance-options.wrapper-hook.title',
		defaultMessage: 'Wrapper hook',
	},
	wrapperHookPlaceholder: {
		id: 'app.settings.default-instance-options.wrapper-hook.placeholder',
		defaultMessage: 'Enter wrapper command...',
	},
	wrapperHookDescription: {
		id: 'app.settings.default-instance-options.wrapper-hook.description',
		defaultMessage: 'Command used to wrap the Minecraft launch process.',
	},
	postExitHookTitle: {
		id: 'app.settings.default-instance-options.post-exit-hook.title',
		defaultMessage: 'Post-exit hook',
	},
	postExitHookPlaceholder: {
		id: 'app.settings.default-instance-options.post-exit-hook.placeholder',
		defaultMessage: 'Enter post-exit command...',
	},
	postExitHookDescription: {
		id: 'app.settings.default-instance-options.post-exit-hook.description',
		defaultMessage: 'Runs after the game closes.',
	},
	hookVariablesDescription: {
		id: 'instance.settings.tabs.hooks.variables.description',
		defaultMessage:
			'Hooks run in the working directory of the instance, with the following variables:',
	},
	instanceNameDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-name.description',
		defaultMessage: '$INST_NAME: The name of the instance',
	},
	instanceIdDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-id.description',
		defaultMessage: "$INST_ID: The name of the instance's folder",
	},
	instanceDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-dir.description',
		defaultMessage: "$INST_DIR: The absolute path to the instance's folder",
	},
	instanceMcDirDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-mc-dir.description',
		defaultMessage: '$INST_MC_DIR: An alias for $INST_DIR',
	},
	instanceJavaDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java.description',
		defaultMessage: '$INST_JAVA: The absolute path to the java binary',
	},
	instanceJavaArgsDescription: {
		id: 'instance.settings.tabs.hooks.variables.inst-java-args.description',
		defaultMessage: '$INST_JAVA_ARGS: The JVM Arguments provided to the game',
	},
})

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
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.fullscreenTitle) }}
					</h3>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.fullscreenDescription) }}
					</p>
				</div>

				<Toggle id="fullscreen" v-model="settings.force_fullscreen" />
			</div>

			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.widthTitle) }}
					</h3>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.widthDescription) }}
					</p>
				</div>

				<StyledInput
					id="width"
					v-model="settings.game_resolution[0]"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.widthPlaceholder)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.heightTitle) }}
					</h3>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.heightDescription) }}
					</p>
				</div>

				<StyledInput
					id="height"
					v-model="settings.game_resolution[1]"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.heightPlaceholder)"
				/>
			</div>
		</div>

		<hr class="my-6 bg-button-border border-none h-[1px]" />

		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.memoryAllocationTitle) }}
				</h2>
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
				<p class="m-0 mt-1 leading-tight">
					{{ formatMessage(messages.memoryAllocationDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.javaArgumentsTitle) }}
				</h2>
				<StyledInput
					id="java-args"
					v-model="settings.launchArgs"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.javaArgumentsPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.javaArgumentsDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.environmentVariablesTitle) }}
				</h2>
				<StyledInput
					id="env-vars"
					v-model="settings.envVars"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.environmentVariablesPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.environmentVariablesDescription) }}
				</p>
			</div>
		</div>

		<hr class="my-6 bg-button-border border-none h-[1px]" />

		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.preLaunchHookTitle) }}
				</h3>
				<StyledInput
					id="pre-launch"
					v-model="settings.hooks.pre_launch"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.preLaunchHookPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.preLaunchHookDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.wrapperHookTitle) }}
				</h3>
				<StyledInput
					id="wrapper"
					v-model="settings.hooks.wrapper"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.wrapperHookPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.wrapperHookDescription) }}
				</p>
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.postExitHookTitle) }}
				</h3>
				<StyledInput
					id="post-exit"
					v-model="settings.hooks.post_exit"
					autocomplete="off"
					type="text"
					:placeholder="formatMessage(messages.postExitHookPlaceholder)"
					wrapper-class="w-full"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.postExitHookDescription) }}
				</p>
			</div>

			<div class="m-0 leading-tight">
				{{ formatMessage(messages.hookVariablesDescription) }}
				<ul>
					<li>{{ formatMessage(messages.instanceNameDescription) }}</li>
					<li>{{ formatMessage(messages.instanceIdDescription) }}</li>
					<li>{{ formatMessage(messages.instanceDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceMcDirDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaDescription) }}</li>
					<li>{{ formatMessage(messages.instanceJavaArgsDescription) }}</li>
				</ul>
			</div>
		</div>
	</div>
</template>
