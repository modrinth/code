<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	Input,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { ref, watch } from 'vue'

import useMemorySlider from '@/composables/useMemorySlider'
import {
	type AppSettings,
	appSettingsKeys,
	appSettingsQueryOptions,
	get,
	parseEnvVars,
	serializeEnvVars,
	set,
} from '@/helpers/settings'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
type LaunchSettings = Pick<
	AppSettings,
	'force_fullscreen' | 'game_resolution' | 'memory' | 'hooks'
> & {
	launchArgs: string
	envVars: string
}
type LaunchSettingsUpdate = Pick<
	AppSettings,
	| 'force_fullscreen'
	| 'game_resolution'
	| 'memory'
	| 'hooks'
	| 'extra_launch_args'
	| 'custom_env_vars'
>

const settingsQuery = useQuery(appSettingsQueryOptions())
const settings = ref<LaunchSettings | null>(null)
const mutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (changes: LaunchSettingsUpdate) => {
		await set({ ...(await get()), ...changes })
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onError: handleError,
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: appSettingsKeys.update }) === 1) {
			await queryClient.invalidateQueries({ queryKey: appSettingsKeys.all })
		}
	},
})
watch(
	settingsQuery.data,
	(value) => {
		if (
			!value ||
			(settings.value && queryClient.isMutating({ mutationKey: appSettingsKeys.update }))
		) {
			return
		}
		settings.value = {
			force_fullscreen: value.force_fullscreen,
			game_resolution: [...value.game_resolution],
			memory: { ...value.memory },
			hooks: { ...value.hooks },
			launchArgs: value.extra_launch_args.join(' '),
			envVars: serializeEnvVars(value.custom_env_vars),
		}
	},
	{ immediate: true, flush: 'sync' },
)
watch(settingsQuery.error, (error) => {
	if (error) handleError(error)
})
watch(
	settings,
	(value, previous) => {
		if (!value || value !== previous) return
		if (!value.game_resolution.every((dimension) => Number.isInteger(dimension) && dimension > 0)) {
			return
		}
		mutation.mutate({
			force_fullscreen: value.force_fullscreen,
			game_resolution: [...value.game_resolution],
			memory: { ...value.memory },
			hooks: { ...value.hooks },
			extra_launch_args: value.launchArgs.trim().split(/\s+/).filter(Boolean),
			custom_env_vars: parseEnvVars(value.envVars),
		})
	},
	{ deep: true },
)

const { maxMemory, snapPoints } = await useMemorySlider()

const messages = defineMessages({
	windowSectionTitle: {
		id: 'app.settings.default-instance-options.window.title',
		defaultMessage: 'Window',
	},
	javaAndMemorySectionTitle: {
		id: 'app.settings.default-instance-options.java-and-memory.title',
		defaultMessage: 'Java and memory',
	},
	launchHooksSectionTitle: {
		id: 'app.settings.default-instance-options.launch-hooks.title',
		defaultMessage: 'Launch hooks',
	},
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
</script>

<template>
	<div v-if="settings">
		<section class="mt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.windowSectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<h3 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.fullscreenTitle) }}
						</h3>
						<p class="m-0 leading-tight">
							{{ formatMessage(messages.fullscreenDescription) }}
						</p>
					</div>

					<Toggle
						id="fullscreen"
						v-model="settings.force_fullscreen"
						:aria-label="formatMessage(messages.fullscreenTitle)"
					/>
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

					<Input
						id="width"
						v-model="settings.game_resolution[0]"
						:aria-label="formatMessage(messages.widthTitle)"
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

					<Input
						id="height"
						v-model="settings.game_resolution[1]"
						:aria-label="formatMessage(messages.heightTitle)"
						:disabled="settings.force_fullscreen"
						autocomplete="off"
						type="number"
						:placeholder="formatMessage(messages.heightPlaceholder)"
					/>
				</div>
			</div>
		</section>

		<section class="mt-8 border-0 border-t border-solid border-surface-4 pt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.javaAndMemorySectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.memoryAllocationTitle) }}
					</h3>
					<Slider
						id="max-memory"
						v-model="settings.memory.maximum"
						:aria-label="formatMessage(messages.memoryAllocationTitle)"
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
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaArgumentsTitle) }}
					</h3>
					<Input
						id="java-args"
						v-model="settings.launchArgs"
						:aria-label="formatMessage(messages.javaArgumentsTitle)"
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
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.environmentVariablesTitle) }}
					</h3>
					<Input
						id="env-vars"
						v-model="settings.envVars"
						:aria-label="formatMessage(messages.environmentVariablesTitle)"
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
		</section>

		<section class="mt-8 border-0 border-t border-solid border-surface-4 pt-6">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.launchHooksSectionTitle) }}
			</h2>
			<div class="mt-4 flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.preLaunchHookTitle) }}
					</h3>
					<Input
						id="pre-launch"
						v-model="settings.hooks.pre_launch"
						:aria-label="formatMessage(messages.preLaunchHookTitle)"
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
					<Input
						id="wrapper"
						v-model="settings.hooks.wrapper"
						:aria-label="formatMessage(messages.wrapperHookTitle)"
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
					<Input
						id="post-exit"
						v-model="settings.hooks.post_exit"
						:aria-label="formatMessage(messages.postExitHookTitle)"
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
		</section>
	</div>
	<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
		{{ formatMessage(commonMessages.refreshButton) }}
	</Button>
	<div v-else class="flex items-center gap-2 py-6 text-secondary">
		<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
		{{ formatMessage(commonMessages.loadingLabel) }}
	</div>
</template>
