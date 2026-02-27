<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div
				class="rounded-2xl border-[1px] border-solid border-orange bg-bg-orange p-4 text-contrast"
			>
				These settings are for advanced users. Changing them can break your server.
			</div>

			<div class="gap-2">
				<div class="card flex flex-col gap-4">
					<div class="flex flex-col justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Startup command</span>
							<span> The command that runs when your server is started. </span>
						</label>
						<ButtonStyled>
							<button
								:disabled="isStartupLoading || invocation === defaultInvocation"
								class="!w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								Restore default command
							</button>
						</ButtonStyled>
					</div>
					<div class="relative">
						<StyledInput
							id="startup-command-field"
							v-model="invocation"
							multiline
							resize="vertical"
							input-class="min-h-[270px] font-[family-name:var(--mono-font)]"
							:disabled="isStartupLoading"
						/>
						<div
							v-if="isStartupLoading"
							class="absolute inset-0 flex items-center justify-center rounded-xl bg-bg/50"
						>
							<SpinnerIcon class="h-6 w-6 animate-spin text-secondary" />
						</div>
					</div>
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
							<Toggle id="show-all-versions" v-model="showAllVersions" class="flex-none" />
							<label for="show-all-versions" class="text-sm">Show all Java versions</label>
						</div>
						<div class="relative">
							<Combobox
								:id="'java-version-field'"
								v-model="jdkVersion"
								name="java-version"
								:options="displayedJdkVersions"
								:display-value="jdkVersionLabel ?? 'Java Version'"
								:disabled="isStartupLoading"
							/>
							<div
								v-if="isStartupLoading"
								class="absolute inset-0 flex items-center justify-center rounded-xl bg-bg/50"
							>
								<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
							</div>
						</div>
					</div>
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Runtime</span>
							<span> The Java runtime your server will use. </span>
						</div>
						<div class="relative">
							<Combobox
								:id="'runtime-field'"
								v-model="jdkBuild"
								name="runtime"
								:options="JDK_BUILDS"
								:display-value="jdkBuildLabel ?? 'Runtime'"
								:disabled="isStartupLoading"
							/>
							<div
								v-if="isStartupLoading"
								class="absolute inset-0 flex items-center justify-center rounded-xl bg-bg/50"
							>
								<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
		<SaveBanner
			:is-visible="!!hasUnsavedChanges"
			:server-id="serverId"
			:is-updating="isPending"
			:save="() => saveStartup()"
			:reset="resetStartup"
		/>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon, UpdatedIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	StyledInput,
	Toggle,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'

const { addNotification } = injectNotificationManager()
const { server, serverId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const STARTUP_QUERY_KEY = ['servers', 'startup', serverId] as const

const {
	data: startupData,
	isLoading: isStartupLoading,
} = useQuery({
	queryKey: STARTUP_QUERY_KEY,
	queryFn: () => client.archon.servers_v0.getStartupConfig(serverId),
})

const JDK_VERSIONS = [
	{ value: 'lts8', label: 'Java 8' },
	{ value: 'lts11', label: 'Java 11' },
	{ value: 'lts17', label: 'Java 17' },
	{ value: 'lts21', label: 'Java 21' },
]

const JDK_BUILDS = [
	{ value: 'corretto', label: 'Corretto' },
	{ value: 'temurin', label: 'Temurin' },
	{ value: 'graal', label: 'GraalVM' },
]

// Saved state derived directly from query
const savedInvocation = computed(() => startupData.value?.invocation ?? '')
const savedJdkVersion = computed(() => startupData.value?.jdk_version)
const savedJdkBuild = computed(() => startupData.value?.jdk_build)
const defaultInvocation = computed(
	() => startupData.value?.original_invocation ?? savedInvocation.value,
)

// Local form state (stores API keys directly)
const invocation = ref('')
const jdkVersion = ref<string>()
const jdkBuild = ref<string>()

// Display labels for comboboxes
const jdkVersionLabel = computed(() => JDK_VERSIONS.find((v) => v.value === jdkVersion.value)?.label)
const jdkBuildLabel = computed(() => JDK_BUILDS.find((v) => v.value === jdkBuild.value)?.label)

function syncFormFromData() {
	invocation.value = savedInvocation.value
	jdkVersion.value = savedJdkVersion.value
	jdkBuild.value = savedJdkBuild.value
}

watch(startupData, (newData, oldData) => {
	if (newData && !oldData) {
		syncFormFromData()
	}
}, { immediate: true })

const hasUnsavedChanges = computed(
	() =>
		invocation.value !== savedInvocation.value ||
		jdkVersion.value !== savedJdkVersion.value ||
		jdkBuild.value !== savedJdkBuild.value,
)

// Java version filtering
const showAllVersions = ref(false)

const displayedJdkVersions = computed(() => {
	if (showAllVersions.value) return JDK_VERSIONS

	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return JDK_VERSIONS

	const [, minor] = mcVersion.split('.').map(Number)

	if (minor >= 20) return JDK_VERSIONS.filter((v) => v.value === 'lts21')
	if (minor >= 17) return JDK_VERSIONS.filter((v) => ['lts17', 'lts21'].includes(v.value))
	if (minor >= 12) return JDK_VERSIONS
	if (minor >= 6) return JDK_VERSIONS.filter((v) => ['lts8', 'lts11'].includes(v.value))
	return JDK_VERSIONS.filter((v) => v.value === 'lts8')
})

// Save mutation
const { mutate: saveStartup, isPending } = useMutation({
	mutationFn: () =>
		client.archon.servers_v0.updateStartupConfig(serverId, {
			invocation: invocation.value || null,
			jdk_version: jdkVersion.value || null,
			jdk_build: jdkBuild.value || null,
		}),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: STARTUP_QUERY_KEY })
		syncFormFromData()
		addNotification({
			type: 'success',
			title: 'Server settings updated',
			text: 'Your server settings were successfully changed.',
		})
	},
	onError: (error) => {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update server arguments',
			text: 'Please try again later.',
		})
	},
})

function resetStartup() {
	syncFormFromData()
}

function resetToDefault() {
	invocation.value = defaultInvocation.value
}
</script>
