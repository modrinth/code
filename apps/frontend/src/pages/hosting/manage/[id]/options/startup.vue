<template>
	<div class="relative h-full w-full">
		<div v-if="isError" class="flex w-full flex-col items-center justify-center gap-4 p-4">
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-orange p-4">
							<IssuesIcon class="size-12 text-orange" />
						</div>
						<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load startup settings</h1>
					</div>
					<p class="text-lg text-secondary">
						We couldn't load your server's startup settings. Here's what we know:
					</p>
					<p>
						<span class="break-all font-mono">{{ error?.message ?? 'Unknown error' }}</span>
					</p>
					<ButtonStyled size="large" color="brand" @click="refetch">
						<button class="mt-6 !w-full">Retry</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div v-else-if="isLoading" class="flex h-full w-full items-center justify-center">
			<div class="text-secondary">Loading startup settings...</div>
		</div>
		<div v-else-if="startupData" class="flex h-full w-full flex-col gap-4">
			<Admonition type="warning">
				These settings are for advanced users. Changing them can break your server.
			</Admonition>

			<div class="gap-2">
				<div class="card flex flex-col gap-4">
					<div class="flex flex-col justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Startup command</span>
							<span> The command that runs when your server is started. </span>
						</label>
						<ButtonStyled>
							<button
								:disabled="invocation === startupData.original_invocation"
								class="!w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								Restore default command
							</button>
						</ButtonStyled>
					</div>
					<textarea
						id="startup-command-field"
						v-model="invocation"
						class="min-h-[270px] w-full resize-y font-mono"
					/>
				</div>

				<div class="card flex flex-col gap-8">
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Java version</span>
							<span>
								The version of Java that your server will run on. By default, only the Java versions
								compatible with this version of Minecraft are shown. Some mods may require a
								different Java version to work properly.
							</span>
						</div>
						<div class="flex items-center gap-2">
							<input
								id="show-all-versions"
								v-model="showAllVersions"
								class="switch stylized-toggle flex-none"
								type="checkbox"
							/>
							<label for="show-all-versions" class="text-sm">Show all Java versions</label>
						</div>
						<Combobox
							:id="'java-version-field'"
							v-model="jdkVersion"
							name="java-version"
							:options="displayedJavaVersions.map((v) => ({ value: v, label: v }))"
							:display-value="jdkVersion ?? 'Java Version'"
						/>
					</div>
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Runtime</span>
							<span> The Java runtime your server will use. </span>
						</div>
						<Combobox
							:id="'runtime-field'"
							v-model="jdkBuild"
							name="runtime"
							:options="['Corretto', 'Temurin', 'GraalVM'].map((v) => ({ value: v, label: v }))"
							:display-value="jdkBuild ?? 'Runtime'"
						/>
					</div>
				</div>
			</div>
		</div>
		<UnsavedChangesPopup
			:original="originalValues"
			:modified="modifiedValues"
			:saving="isSaving"
			@save="saveStartup"
			@reset="resetStartup"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { IssuesIcon, UpdatedIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server, serverId } = injectModrinthServerContext()

// Fetch startup settings using tanstack-query
const {
	data: startupData,
	isLoading,
	isError,
	error,
	refetch,
} = useQuery({
	queryKey: ['server-startup', serverId],
	queryFn: () => client.archon.settings_v0.getStartupSettings(serverId),
})

const showAllVersions = ref(false)

const jdkVersionMap = [
	{ value: 'lts8', label: 'Java 8' },
	{ value: 'lts11', label: 'Java 11' },
	{ value: 'lts17', label: 'Java 17' },
	{ value: 'lts21', label: 'Java 21' },
]

const jdkBuildMap: { value: Archon.Settings.v0.JreVendor; label: string }[] = [
	{ value: 'corretto', label: 'Corretto' },
	{ value: 'temurin', label: 'Temurin' },
	{ value: 'graal', label: 'GraalVM' },
]

// Form state
const invocation = ref('')
const jdkVersion = ref('')
const jdkBuild = ref('')

// Original values for comparison
const originalInvocation = ref('')
const originalJdkVersion = ref('')
const originalJdkBuild = ref('')

// Initialize form when data loads
watch(
	startupData,
	(data) => {
		if (data) {
			invocation.value = data.invocation ?? ''
			originalInvocation.value = data.invocation ?? ''

			const version = jdkVersionMap.find((v) => v.value === data.jdk_version)?.label ?? 'Java 17'
			jdkVersion.value = version
			originalJdkVersion.value = version

			const build = jdkBuildMap.find((v) => v.value === data.jdk_build)?.label ?? 'Corretto'
			jdkBuild.value = build
			originalJdkBuild.value = build
		}
	},
	{ immediate: true },
)

// For UnsavedChangesPopup comparison
const originalValues = computed(() => ({
	invocation: originalInvocation.value,
	jdkVersion: originalJdkVersion.value,
	jdkBuild: originalJdkBuild.value,
}))

const modifiedValues = computed(() => ({
	invocation: invocation.value,
	jdkVersion: jdkVersion.value,
	jdkBuild: jdkBuild.value,
}))

const isSaving = ref(false)

const compatibleJavaVersions = computed(() => {
	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return jdkVersionMap.map((v) => v.label)

	const [major, minor] = mcVersion.split('.').map(Number)

	if (major >= 1) {
		if (minor >= 20) return ['Java 21']
		if (minor >= 18) return ['Java 17', 'Java 21']
		if (minor >= 17) return ['Java 16', 'Java 17', 'Java 21']
		if (minor >= 12) return ['Java 8', 'Java 11', 'Java 17', 'Java 21']
		if (minor >= 6) return ['Java 8', 'Java 11']
	}

	return ['Java 8']
})

const displayedJavaVersions = computed(() => {
	return showAllVersions.value ? jdkVersionMap.map((v) => v.label) : compatibleJavaVersions.value
})

// Update mutation using the new api-client
const updateStartupMutation = useMutation({
	mutationFn: (options: Archon.Settings.v0.PostStartupRequest) =>
		client.archon.settings_v0.updateStartupSettings(serverId, options),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['server-startup', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to update startup settings',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

async function saveStartup() {
	try {
		isSaving.value = true
		const jdkVersionKey = jdkVersionMap.find((v) => v.label === jdkVersion.value)?.value
		const jdkBuildKey = jdkBuildMap.find((v) => v.label === jdkBuild.value)?.value

		await updateStartupMutation.mutateAsync({
			invocation: invocation.value,
			jdk_version: jdkVersionKey,
			jdk_build: jdkBuildKey,
		})

		// Update originals after successful save
		originalInvocation.value = invocation.value
		originalJdkVersion.value = jdkVersion.value
		originalJdkBuild.value = jdkBuild.value

		addNotification({
			type: 'success',
			title: 'Server settings updated',
			text: 'Your server settings were successfully changed.',
		})
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update server arguments',
			text: 'Please try again later.',
		})
	} finally {
		isSaving.value = false
	}
}

function resetStartup() {
	invocation.value = originalInvocation.value
	jdkVersion.value = originalJdkVersion.value
	jdkBuild.value = originalJdkBuild.value
}

function resetToDefault() {
	invocation.value = startupData.value?.original_invocation ?? ''
}
</script>

<style scoped>
.stylized-toggle:checked::after {
	background: var(--color-accent-contrast) !important;
}
</style>
