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
	Checkbox,
	defineMessages,
	injectNotificationManager,
	Slider,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, readonly, ref, watch } from 'vue'

import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'
import useMemorySlider from '@/composables/useMemorySlider'
import { test_jre } from '@/helpers/jre.js'
import { edit, get_optimal_jre_key } from '@/helpers/profile'
import { get } from '@/helpers/settings.ts'

import type { AppSettings, InstanceSettingsTabProps } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const globalSettings = (await get().catch(handleError)) as unknown as AppSettings

const optimalJava = readonly(await get_optimal_jre_key(props.instance.path).catch(handleError))

const overrideJavaInstall = ref(!!props.instance.java_path)
const javaPath = ref(props.instance.java_path ?? optimalJava?.path ?? '')

const activePath = computed(() =>
	overrideJavaInstall.value ? javaPath.value : (optimalJava?.path ?? ''),
)

watch(overrideJavaInstall, (enabled) => {
	if (enabled && !javaPath.value) {
		javaPath.value = optimalJava?.path ?? ''
	}
})

// Auto-test state
const testingJava = ref(false)
const javaTestResult = ref<boolean | null>(null)
const hoveringTest = ref(false)
let testDebounceTimer: ReturnType<typeof setTimeout> | null = null

async function runJavaTest(path: string) {
	if (testDebounceTimer) {
		clearTimeout(testDebounceTimer)
		testDebounceTimer = null
	}
	if (!path || !optimalJava?.parsed_version) {
		javaTestResult.value = null
		return
	}
	testingJava.value = true
	try {
		javaTestResult.value = await test_jre(path, optimalJava?.parsed_version ?? null)
	} catch {
		javaTestResult.value = false
	}
	testingJava.value = false
}

function scheduleJavaTest(path: string) {
	if (testDebounceTimer) clearTimeout(testDebounceTimer)
	javaTestResult.value = null
	testDebounceTimer = setTimeout(() => runJavaTest(path), 600)
}

onMounted(() => {
	if (activePath.value) runJavaTest(activePath.value)
})

watch(activePath, (newPath) => scheduleJavaTest(newPath))

const javaDetectionModal = ref<{ show: (version: number, current: object) => void } | null>(null)

async function handleBrowseJava() {
	const result = await open({ multiple: false })
	if (result) {
		javaPath.value = result
	}
}

function handleDetectJava() {
	javaDetectionModal.value.show(optimalJava?.parsed_version, { path: javaPath.value })
}

const overrideJavaArgs = ref((props.instance.extra_launch_args?.length ?? 0) > 0)
const javaArgs = ref(
	(props.instance.extra_launch_args ?? globalSettings.extra_launch_args).join(' '),
)

const overrideEnvVars = ref((props.instance.custom_env_vars?.length ?? 0) > 0)
const envVars = ref(
	(props.instance.custom_env_vars ?? globalSettings.custom_env_vars)
		.map((x) => x.join('='))
		.join(' '),
)

const overrideMemorySettings = ref(!!props.instance.memory)
const memory = ref(props.instance.memory ?? globalSettings.memory)
const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

const editProfileObject = computed(() => {
	return {
		java_path:
			overrideJavaInstall.value && javaPath.value
				? javaPath.value.replace('java.exe', 'javaw.exe')
				: null,
		extra_launch_args: overrideJavaArgs.value
			? javaArgs.value.trim().split(/\s+/).filter(Boolean)
			: null,
		custom_env_vars: overrideEnvVars.value
			? envVars.value
					.trim()
					.split(/\s+/)
					.filter(Boolean)
					.map((x) => x.split('=').filter(Boolean))
			: null,
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
		await edit(props.instance.path, editProfileObject.value)
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
		defaultMessage: 'Custom Java installation',
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
		defaultMessage: 'Custom memory allocation',
	},
	javaArguments: {
		id: 'instance.settings.tabs.java.java-arguments',
		defaultMessage: 'Java arguments',
	},
	customJavaArguments: {
		id: 'instance.settings.tabs.java.custom-java-arguments',
		defaultMessage: 'Custom Java arguments',
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
		defaultMessage: 'Custom environment variables',
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
	<div>
		<JavaDetectionModal ref="javaDetectionModal" @submit="(val) => (javaPath = val.path)" />
		<h2 class="m-0 mb-2 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaInstallation) }}
		</h2>
		<Checkbox
			v-model="overrideJavaInstall"
			:label="formatMessage(messages.customJavaInstallation)"
			class="mb-2"
		/>
		<div class="flex gap-4 p-4 bg-bg rounded-2xl">
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
						<StyledInput
							:model-value="activePath"
							:disabled="!overrideJavaInstall"
							autocomplete="off"
							:placeholder="formatMessage(messages.javaPathPlaceholder)"
							wrapper-class="flex-1 min-w-0"
							@update:model-value="(val) => (javaPath = String(val))"
						/>
						<Button
							:disabled="!overrideJavaInstall || testingJava"
							@click="runJavaTest(activePath)"
							@mouseenter="hoveringTest = true"
							@mouseleave="hoveringTest = false"
						>
							<SpinnerIcon v-if="testingJava" class="animate-spin h-4 w-4" />
							<RefreshCwIcon v-else-if="hoveringTest && overrideJavaInstall" class="h-4 w-4" />
							<CheckCircleIcon
								v-else-if="javaTestResult === true"
								class="h-4 w-4 text-brand-green"
							/>
							<XCircleIcon v-else-if="javaTestResult === false" class="h-4 w-4 text-brand-red" />
							<RefreshCwIcon v-else class="h-4 w-4" />
						</Button>
					</div>
					<div v-if="overrideJavaInstall" class="flex gap-2">
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
		<h2 class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaMemory) }}
		</h2>
		<Checkbox
			v-model="overrideMemorySettings"
			:label="formatMessage(messages.customMemoryAllocation)"
			class="mb-2"
		/>
		<Slider
			id="max-memory"
			v-model="memory.maximum"
			:disabled="!overrideMemorySettings"
			:min="512"
			:max="maxMemory"
			:step="64"
			:snap-points="snapPoints"
			:snap-range="512"
			unit="MB"
		/>
		<h2 class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaArguments) }}
		</h2>
		<Checkbox
			v-model="overrideJavaArgs"
			:label="formatMessage(messages.customJavaArguments)"
			class="my-2"
		/>
		<StyledInput
			id="java-args"
			v-model="javaArgs"
			autocomplete="off"
			:disabled="!overrideJavaArgs"
			:placeholder="formatMessage(messages.enterJavaArguments)"
			wrapper-class="w-full"
		/>
		<h2 class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaEnvironmentVariables) }}
		</h2>
		<Checkbox
			v-model="overrideEnvVars"
			:label="formatMessage(messages.customEnvironmentVariables)"
			class="mb-2"
		/>
		<StyledInput
			id="env-vars"
			v-model="envVars"
			autocomplete="off"
			:disabled="!overrideEnvVars"
			:placeholder="formatMessage(messages.enterEnvironmentVariables)"
			wrapper-class="w-full"
		/>
	</div>
</template>
