<template>
	<JavaDetectionModal ref="detectJavaModal" @submit="(val) => emit('update:modelValue', val)" />
	<div :id="props.id" class="toggle-setting" :class="{ compact }">
		<div class="input-with-status">
			<StyledInput
				autocomplete="off"
				:disabled="props.disabled"
				:model-value="props.modelValue ? props.modelValue.path : ''"
				:placeholder="placeholder ?? '/path/to/java'"
				wrapper-class="installation-input"
				@update:model-value="
					(val) => {
						emit('update:modelValue', {
							...props.modelValue,
							path: val,
						})
					}
				"
			/>
			<Button
				:disabled="testingJava || props.disabled"
				@click="runTest(props.modelValue?.path)"
				@mouseenter="!props.disabled && (hoveringTest = true)"
				@mouseleave="hoveringTest = false"
			>
				<SpinnerIcon v-if="testingJava" class="animate-spin h-4 w-4" />
				<CheckCircleIcon
					v-else-if="testingJavaSuccess === true && !hoveringTest"
					class="h-4 w-4 text-brand-green"
				/>
				<XCircleIcon
					v-else-if="testingJavaSuccess !== true && !hoveringTest"
					class="h-4 w-4 text-brand-red"
				/>
				<RefreshCwIcon v-else-if="!props.disabled" class="h-4 w-4" />
			</Button>
		</div>
		<span class="installation-buttons">
			<Button
				v-if="props.version"
				:disabled="props.disabled || installingJava"
				@click="reinstallJava"
			>
				<DownloadIcon />
				{{ installingJava ? 'Installing...' : 'Install recommended' }}
			</Button>
			<Button :disabled="props.disabled" @click="autoDetect">
				<SearchIcon />
				Detect
			</Button>
			<Button :disabled="props.disabled" @click="handleJavaFileInput()">
				<FolderSearchIcon />
				Browse
			</Button>
		</span>
	</div>
</template>

<script setup>
import {
	CheckCircleIcon,
	DownloadIcon,
	FolderSearchIcon,
	RefreshCwIcon,
	SearchIcon,
	SpinnerIcon,
	XCircleIcon,
} from '@modrinth/assets'
import { Button, injectNotificationManager, StyledInput } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { ref, watch } from 'vue'

import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'
import useJavaTest from '@/composables/useJavaTest.js'
import { trackEvent } from '@/helpers/analytics'
import { auto_install_java, find_filtered_jres, get_jre } from '@/helpers/jre.js'

const { handleError } = injectNotificationManager()

const props = defineProps({
	id: {
		type: String,
		required: false,
		default: null,
	},
	version: {
		type: Number,
		required: false,
		default: null,
	},
	modelValue: {
		type: Object,
		default: () => ({
			path: '',
			version: '',
		}),
	},
	disabled: {
		type: Boolean,
		required: false,
		default: false,
	},
	placeholder: {
		type: String,
		required: false,
		default: null,
	},
	compact: {
		type: Boolean,
		default: false,
	},
})

const emit = defineEmits(['update:modelValue'])

const {
	testingJava,
	javaTestResult: testingJavaSuccess,
	testJavaInstallationDebounced,
	testJavaInstallation,
} = useJavaTest()

const installingJava = ref(false)
const hoveringTest = ref(false)
let hasInitialized = false

async function runTest(path) {
	await testJavaInstallation(path, props.version, true)
}

watch(
	() => props.modelValue?.path,
	(newPath) => {
		if (newPath) {
			if (!hasInitialized) {
				testJavaInstallation(newPath, props.version, false)
				hasInitialized = true
			} else {
				testJavaInstallationDebounced(newPath, props.version)
			}
		}
	},
	{ immediate: true },
)

async function handleJavaFileInput() {
	const filePath = await open()

	if (filePath) {
		let result = await get_jre(filePath.path ?? filePath).catch(handleError)
		if (!result) {
			result = {
				path: filePath.path ?? filePath,
				version: props.version.toString(),
				parsed_version: props.version,
				architecture: 'x86',
			}
		}

		trackEvent('JavaManualSelect', {
			version: props.version,
		})

		emit('update:modelValue', result)
	}
}

const detectJavaModal = ref(null)
async function autoDetect() {
	if (!props.compact) {
		detectJavaModal.value.show(props.version, props.modelValue)
	} else {
		const versions = await find_filtered_jres(props.version).catch(handleError)
		if (versions.length > 0) {
			emit('update:modelValue', versions[0])
		}
	}
}

async function reinstallJava() {
	installingJava.value = true
	const path = await auto_install_java(props.version).catch(handleError)
	let result = await get_jre(path)

	if (!result) {
		result = {
			path: path,
			version: props.version.toString(),
			parsed_version: props.version,
			architecture: 'x86',
		}
	}

	trackEvent('JavaReInstall', {
		path: path,
		version: props.version,
	})

	emit('update:modelValue', result)
	installingJava.value = false
	runTest(result.path)
}
</script>

<style lang="scss" scoped>
.input-with-status {
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 0.5rem;
	width: 100%;
	min-width: 0;
}

.installation-input {
	flex: 1 1 0;
	min-width: 0;
}

.toggle-setting {
	display: flex;
	flex-wrap: wrap;
	flex-direction: row;
	justify-content: space-between;
	align-items: center;
	gap: 0.5rem;

	&.compact {
		flex-wrap: wrap;
	}
}

.installation-buttons {
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 0.5rem;
	margin: 0;

	.btn {
		width: max-content;
	}
}
</style>
