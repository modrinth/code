<script setup lang="ts">
import {
	CheckCircleIcon,
	CoffeeIcon,
	FolderSearchIcon,
	RefreshCwIcon,
	SearchIcon,
	SpinnerIcon,
	XCircleIcon,
} from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, readonly, ref, watch } from 'vue'

import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'
import useJavaTest from '@/composables/useJavaTest'
import useMemorySlider from '@/composables/useMemorySlider'
import { edit, get_optimal_jre_key } from '@/helpers/instance'
import { get, parseEnvVars, serializeEnvVars } from '@/helpers/settings.ts'

import type { AppSettings } from '../../../../helpers/types'
import { injectInstanceSettings } from './instance-settings-context'
import SettingsOptionsTransition from './settings-options-transition.vue'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const globalSettings = (await get().catch(handleError)) as unknown as AppSettings

const optimalJava = readonly(await get_optimal_jre_key(instance.value.id).catch(handleError))

const overrideJavaInstall = ref(!!instance.value.java_path)
const javaPath = ref(instance.value.java_path ?? optimalJava?.path ?? '')

const activePath = computed(() => (overrideJavaInstall.value ? javaPath.value : ''))

watch(overrideJavaInstall, (enabled) => {
	if (enabled && !javaPath.value) {
		javaPath.value = optimalJava?.path ?? ''
	}
})

const { testingJava, javaTestResult, testJavaInstallationDebounced, testJavaInstallation } =
	useJavaTest()

const hoveringTest = ref(false)
let hasInitialized = false

watch(
	activePath,
	(newPath) => {
		if (newPath && optimalJava?.parsed_version) {
			if (!hasInitialized) {
				testJavaInstallation(newPath, optimalJava?.parsed_version, false)
				hasInitialized = true
			} else {
				testJavaInstallationDebounced(newPath, optimalJava?.parsed_version)
			}
		}
	},
	{ immediate: true },
)

const javaDetectionModal = ref<{ show: (version: number, current: object) => void } | null>(null)

async function handleBrowseJava() {
	const result = await open({ multiple: false })
	if (result) {
		javaPath.value = result
	}
}

function handleDetectJava() {
	javaDetectionModal.value?.show(optimalJava?.parsed_version, { path: javaPath.value })
}

const overrideJavaArgs = ref((instance.value.extra_launch_args?.length ?? 0) > 0)
const javaArgs = ref(
	(instance.value.extra_launch_args ?? globalSettings.extra_launch_args).join(' '),
)

const overrideEnvVars = ref((instance.value.custom_env_vars?.length ?? 0) > 0)
const envVars = ref(
	serializeEnvVars(instance.value.custom_env_vars ?? globalSettings.custom_env_vars),
)

const overrideMemorySettings = ref(!!instance.value.memory)
const memory = ref(instance.value.memory ?? globalSettings.memory)
const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

const editInstanceObject = computed(() => {
	return {
		java_path:
			overrideJavaInstall.value && javaPath.value
				? javaPath.value.replace('java.exe', 'javaw.exe')
				: null,
		extra_launch_args: overrideJavaArgs.value
			? javaArgs.value.trim().split(/\s+/).filter(Boolean)
			: null,
		custom_env_vars: overrideEnvVars.value ? parseEnvVars(envVars.value) : null,
		memory: overrideMemorySettings.value ? memory.value : null,
	}
})

watch(
	[
		overrideJavaInstall,
		javaPath,
		overrideJavaArgs,
		javaArgs,
		overrideEnvVars,
		envVars,
		overrideMemorySettings,
		memory,
	],
	async () => {
		await edit(instance.value.id, editInstanceObject.value).catch(handleError)
	},
	{ deep: true },
)

const messages = defineMessages({
	javaInstallation: {
		id: 'instance.settings.tabs.java.java-installation',
		defaultMessage: 'Java installation',
	},
	customJavaInstallation: {
		id: 'instance.settings.tabs.java.custom-java-installation',
		defaultMessage: 'Use a custom Java installation for this instance.',
	},
	javaPathPlaceholder: {
		id: 'instance.settings.tabs.java.java-path-placeholder',
		defaultMessage: '/path/to/java',
	},
	javaMemory: {
		id: 'instance.settings.tabs.java.java-memory',
		defaultMessage: 'Memory allocated',
	},
	customMemoryAllocation: {
		id: 'instance.settings.tabs.java.custom-memory-allocation',
		defaultMessage: 'Use a custom memory allocation for this instance.',
	},
	javaArguments: {
		id: 'instance.settings.tabs.java.java-arguments',
		defaultMessage: 'Java arguments',
	},
	customJavaArguments: {
		id: 'instance.settings.tabs.java.custom-java-arguments',
		defaultMessage: 'Use custom Java arguments for this instance.',
	},
	enterJavaArguments: {
		id: 'instance.settings.tabs.java.enter-java-arguments',
		defaultMessage: 'Enter Java arguments...',
	},
	javaEnvironmentVariables: {
		id: 'instance.settings.tabs.java.environment-variables',
		defaultMessage: 'Environment variables',
	},
	customEnvironmentVariables: {
		id: 'instance.settings.tabs.java.custom-environment-variables',
		defaultMessage: 'Use custom environment variables for this instance.',
	},
	enterEnvironmentVariables: {
		id: 'instance.settings.tabs.java.enter-environment-variables',
		defaultMessage: 'Enter environmental variables...',
	},
	hooks: {
		id: 'instance.settings.tabs.java.hooks',
		defaultMessage: 'Hooks',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<JavaDetectionModal ref="javaDetectionModal" @submit="(val) => (javaPath = val.path)" />

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaInstallation) }}
					</h2>
					<p class="m-0">{{ formatMessage(messages.customJavaInstallation) }}</p>
				</div>
				<Toggle id="override-java-installation" v-model="overrideJavaInstall" />
			</div>
			<SettingsOptionsTransition :show="overrideJavaInstall">
				<div class="pt-3">
					<div class="flex gap-4 rounded-2xl bg-bg p-4">
						<div class="flex gap-3 items-start flex-1 min-w-0">
							<div
								class="w-10 h-10 flex items-center justify-center rounded-full bg-button-bg border-solid border-[1px] border-button-border p-2 mt-1 shrink-0 [&_svg]:h-full [&_svg]:w-full"
							>
								<CoffeeIcon />
							</div>
							<div class="flex flex-col gap-2 flex-1 min-w-0">
								<span class="font-semibold leading-none mt-2"
									>Java {{ optimalJava?.parsed_version }}</span
								>
								<div class="flex gap-2 items-center">
									<Input
										:model-value="activePath"
										autocomplete="off"
										:placeholder="formatMessage(messages.javaPathPlaceholder)"
										wrapper-class="flex-1 min-w-0"
										@update:model-value="(val) => (javaPath = String(val))"
									/>
									<Button
										type="quiet"
										:color="
											!hoveringTest && !testingJava
												? javaTestResult === true
													? 'green'
													: 'red'
												: undefined
										"
										:disabled="testingJava"
										:style="{
											'--legacy-button-color':
												(!hoveringTest && !testingJava
													? javaTestResult === true
														? 'green'
														: 'red'
													: 'standard') &&
												(!hoveringTest && !testingJava
													? javaTestResult === true
														? 'green'
														: 'red'
													: 'standard') !== 'standard'
													? `var(--color-${
															!hoveringTest && !testingJava
																? javaTestResult === true
																	? 'green'
																	: 'red'
																: 'standard'
														})`
													: undefined,
										}"
										class="!text-[var(--legacy-button-color,var(--color-base))] [&>svg]:!text-[var(--legacy-button-color,var(--color-primary))]"
										@click="testJavaInstallation(activePath, optimalJava?.parsed_version, true)"
										@mouseenter="hoveringTest = true"
										@mouseleave="hoveringTest = false"
									>
										<SpinnerIcon v-if="testingJava" class="animate-spin h-4 w-4" />
										<CheckCircleIcon
											v-else-if="javaTestResult === true && !hoveringTest"
											class="h-4 w-4"
										/>
										<XCircleIcon
											v-else-if="javaTestResult !== true && !hoveringTest"
											class="h-4 w-4"
										/>
										<RefreshCwIcon v-else class="h-4 w-4" />
									</Button>
								</div>
								<div class="flex gap-2">
									<Button @click="handleDetectJava">
										<SearchIcon />
										Detect
									</Button>
									<Button @click="handleBrowseJava">
										<FolderSearchIcon />
										Browse
									</Button>
								</div>
							</div>
						</div>
					</div>
				</div>
			</SettingsOptionsTransition>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaMemory) }}
					</h2>
					<p class="m-0">{{ formatMessage(messages.customMemoryAllocation) }}</p>
				</div>
				<Toggle id="override-memory-allocation" v-model="overrideMemorySettings" />
			</div>
			<SettingsOptionsTransition :show="overrideMemorySettings">
				<div class="pt-3">
					<Slider
						id="max-memory"
						v-model="memory.maximum"
						:min="512"
						:max="maxMemory"
						:step="64"
						:snap-points="snapPoints"
						:snap-range="512"
						unit="MB"
					/>
				</div>
			</SettingsOptionsTransition>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaArguments) }}
					</h2>
					<p class="m-0">{{ formatMessage(messages.customJavaArguments) }}</p>
				</div>
				<Toggle id="override-java-arguments" v-model="overrideJavaArgs" />
			</div>
			<SettingsOptionsTransition :show="overrideJavaArgs">
				<div class="pt-3">
					<Input
						id="java-args"
						v-model="javaArgs"
						autocomplete="off"
						:placeholder="formatMessage(messages.enterJavaArguments)"
						wrapper-class="w-full"
					/>
				</div>
			</SettingsOptionsTransition>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaEnvironmentVariables) }}
					</h2>
					<p class="m-0">{{ formatMessage(messages.customEnvironmentVariables) }}</p>
				</div>
				<Toggle id="override-environment-variables" v-model="overrideEnvVars" />
			</div>
			<SettingsOptionsTransition :show="overrideEnvVars">
				<div class="pt-3">
					<Input
						id="env-vars"
						v-model="envVars"
						autocomplete="off"
						:placeholder="formatMessage(messages.enterEnvironmentVariables)"
						wrapper-class="w-full"
					/>
				</div>
			</SettingsOptionsTransition>
		</section>
	</div>
</template>
