<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div class="card flex flex-col gap-8">
				<!-- SFTP section -->
				<div class="flex flex-col gap-4">
					<div class="flex flex-col justify-between gap-4 sm:flex-row">
						<label class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">SFTP</span>
							<span> SFTP allows you to access your server's files from outside of Modrinth. </span>
						</label>
						<ButtonStyled>
							<a
								v-tooltip="'This button only works with compatible SFTP clients (e.g. WinSCP)'"
								class="!w-full sm:!w-auto"
								:href="sftpUrl"
								target="_blank"
							>
								<ExternalIcon class="h-5 w-5" />
								Launch SFTP
							</a>
						</ButtonStyled>
					</div>

					<div class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
						<span class="text-md font-semibold text-contrast">Server Address</span>
						<div class="flex h-10 items-center justify-between rounded-xl bg-surface-3 px-3 py-2">
							<div class="flex flex-col gap-2">
								<span class="cursor-pointer font-semibold text-primary">
									{{ server?.sftp_host }}
								</span>
							</div>
							<ButtonStyled type="transparent" circular>
								<button
									v-tooltip="'Copy SFTP server address'"
									@click="copyToClipboard('Server address', server?.sftp_host)"
								>
									<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
								</button>
							</ButtonStyled>
						</div>
						<div class="flex flex-col gap-2 sm:mt-0 sm:flex-row">
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-md font-semibold text-contrast">Username</span>
								<div
									class="flex h-10 items-center justify-between rounded-xl bg-surface-3 px-3 py-2"
								>
									<span class="font-semibold text-primary">
										{{ server?.sftp_username }}
									</span>
									<ButtonStyled type="transparent" circular>
										<button
											v-tooltip="'Copy SFTP username'"
											@click="copyToClipboard('Username', server?.sftp_username)"
										>
											<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
										</button>
									</ButtonStyled>
								</div>
							</div>
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-md font-semibold text-contrast">Password</span>
								<div
									class="flex h-10 items-center justify-between rounded-xl bg-surface-3 px-3 py-2"
								>
									<span class="font-semibold text-primary">
										{{
											showPassword
												? server?.sftp_password
												: '*'.repeat(server?.sftp_password?.length ?? 0)
										}}
									</span>
									<div class="flex flex-row items-center gap-1">
										<ButtonStyled type="transparent" circular>
											<button
												v-tooltip="'Copy SFTP password'"
												@click="copyToClipboard('Password', server?.sftp_password)"
											>
												<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
											</button>
										</ButtonStyled>
										<ButtonStyled type="transparent" circular>
											<button
												v-tooltip="showPassword ? 'Hide password' : 'Show password'"
												@click="showPassword = !showPassword"
											>
												<EyeIcon v-if="showPassword" class="h-5 w-5 hover:cursor-pointer" />
												<EyeOffIcon v-else class="h-5 w-5 hover:cursor-pointer" />
											</button>
										</ButtonStyled>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>

				<!-- Startup command section -->
				<div class="flex flex-col gap-2">
					<div class="flex h-9 flex-col items-center justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Startup command</span>
						</label>
						<ButtonStyled v-if="startupCommand !== defaultStartupCommand" type="transparent">
							<button
								:disabled="isStartupLoading || startupCommand === defaultStartupCommand"
								class="!w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								Default
							</button>
						</ButtonStyled>
					</div>
					<div class="relative">
						<StyledInput
							id="startup-command-field"
							v-model="startupCommand"
							multiline
							resize="vertical"
							input-class="font-mono field-sizing-content"
							:disabled="isStartupLoading"
						/>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-6 w-6 animate-spin text-secondary" />
						</div>
					</div>
					<span> The command that runs when your server is started. </span>
				</div>

				<!-- Java version section -->
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">Java version</span>
						<span>
							The version of Java that your server will run on. By default, only the Java versions
							compatible with this version of Minecraft are shown. Some mods may require a different
							Java version to work properly.
						</span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'java-version-field'"
							v-model="javaVersion"
							name="java-version"
							:options="displayedJavaVersions"
							:display-value="javaVersionLabel ?? 'Java Version'"
							:disabled="isStartupLoading"
						>
							<template #dropdown-footer>
								<button
									class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
									@mousedown.prevent
									@click="showAllVersions = !showAllVersions"
								>
									<EyeOffIcon v-if="showAllVersions" class="size-4" />
									<EyeIcon v-else class="size-4" />
									{{ showAllVersions ? 'Hide extra versions' : 'Show all versions' }}
								</button>
							</template>
						</Combobox>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
						</div>
					</div>
				</div>

				<!-- Java runtime section -->
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">Runtime</span>
						<span> The Java runtime your server will use. </span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'runtime-field'"
							v-model="jreVendor"
							name="runtime"
							:options="JRE_VENDORS"
							:display-value="jreVendorLabel ?? 'Runtime'"
							:disabled="isStartupLoading"
						/>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
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
import type { Archon } from '@modrinth/api-client'
import {
	CopyIcon,
	ExternalIcon,
	EyeIcon,
	EyeOffIcon,
	SpinnerIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	StyledInput,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'

const { addNotification } = injectNotificationManager()
const { server, serverId, worldId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()

// SFTP state
const showPassword = ref(false)
const sftpUrl = computed(() => `sftp://${server.value?.sftp_username}@${server.value?.sftp_host}`)

const copyToClipboard = (name: string, textToCopy?: string) => {
	navigator.clipboard.writeText(textToCopy || '')
	addNotification({
		type: 'success',
		title: `${name} copied to clipboard!`,
	})
}

// Startup state
const startupQueryKey = computed(() => ['servers', 'startup', 'v1', serverId, worldId.value])

const { data: startupData, isLoading: isStartupLoading } = useQuery({
	queryKey: startupQueryKey,
	queryFn: () => client.archon.options_v1.getStartup(serverId, worldId.value!),
	enabled: computed(() => worldId.value !== null),
})

const JAVA_VERSIONS = [
	{ value: 8, label: 'Java 8' },
	{ value: 11, label: 'Java 11' },
	{ value: 17, label: 'Java 17' },
	{ value: 21, label: 'Java 21' },
]

const JRE_VENDORS: { value: Archon.Content.v1.JreVendor; label: string }[] = [
	{ value: 'corretto', label: 'Corretto' },
	{ value: 'temurin', label: 'Temurin' },
	{ value: 'graal', label: 'GraalVM' },
]

const savedStartupCommand = computed(() => startupData.value?.startup_command ?? '')
const savedJavaVersion = computed(() => startupData.value?.java_version ?? undefined)
const savedJreVendor = computed(() => startupData.value?.jre_vendor ?? undefined)
const defaultStartupCommand = computed(
	() => startupData.value?.original_invocation ?? savedStartupCommand.value,
)

const startupCommand = ref('')
const javaVersion = ref<number>()
const jreVendor = ref<Archon.Content.v1.JreVendor>()

const javaVersionLabel = computed(
	() => JAVA_VERSIONS.find((v) => v.value === javaVersion.value)?.label,
)
const jreVendorLabel = computed(() => JRE_VENDORS.find((v) => v.value === jreVendor.value)?.label)

function syncFormFromData() {
	startupCommand.value = savedStartupCommand.value
	javaVersion.value = savedJavaVersion.value
	jreVendor.value = savedJreVendor.value
}

watch(
	startupData,
	(newData, oldData) => {
		if (newData && !oldData) {
			syncFormFromData()
		}
	},
	{ immediate: true },
)

const hasUnsavedChanges = computed(
	() =>
		startupCommand.value !== savedStartupCommand.value ||
		javaVersion.value !== savedJavaVersion.value ||
		jreVendor.value !== savedJreVendor.value,
)

const showAllVersions = ref(false)

const displayedJavaVersions = computed(() => {
	if (showAllVersions.value) return JAVA_VERSIONS

	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return JAVA_VERSIONS

	const [, minor] = mcVersion.split('.').map(Number)

	if (minor >= 20) return JAVA_VERSIONS.filter((v) => v.value === 21)
	if (minor >= 17) return JAVA_VERSIONS.filter((v) => [17, 21].includes(v.value))
	if (minor >= 12) return JAVA_VERSIONS
	if (minor >= 6) return JAVA_VERSIONS.filter((v) => [8, 11].includes(v.value))
	return JAVA_VERSIONS.filter((v) => v.value === 8)
})

const { mutate: saveStartup, isPending } = useMutation({
	mutationFn: () =>
		client.archon.options_v1.patchStartup(serverId, worldId.value!, {
			startup_command: startupCommand.value || null,
			java_version: javaVersion.value ?? null,
			jre_vendor: jreVendor.value ?? null,
		}),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: startupQueryKey.value })
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
	startupCommand.value = defaultStartupCommand.value
}
</script>
