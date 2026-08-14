<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div class="flex flex-col gap-6">
				<!-- SFTP section -->
				<div class="flex flex-col gap-2">
					<div class="flex flex-col items-center justify-between gap-0.5 sm:flex-row">
						<span class="text-lg font-semibold text-contrast">SFTP</span>
						<ButtonLink
							v-tooltip="sftpActionTooltip"
							class="!w-full sm:!w-auto"
							:class="{ 'opacity-60': !canWriteFiles }"
							:href="canWriteFiles ? sftpUrl : undefined"
							:aria-disabled="!canWriteFiles"
							target="_blank"
							@click="handleSftpLaunchClick"
						>
							<ExternalIcon class="h-5 w-5" />
							Launch SFTP
						</ButtonLink>
					</div>

					<div class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
						<span class="text-lg font-semibold text-contrast">Server Address</span>
						<Button
							v-tooltip="sftpCopyTooltip('Copy SFTP server address')"
							native-type="button"
							size="lg"
							class="w-full !justify-between text-left"
							:disabled="!canWriteFiles"
							@click="copyToClipboard('Server address', server?.sftp_host)"
						>
							<span class="min-w-0 truncate text-base font-semibold text-primary">
								{{ server?.sftp_host }}
							</span>
							<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
						</Button>
						<div class="flex flex-col gap-2 sm:mt-0 sm:flex-row">
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-lg font-semibold text-contrast">Username</span>
								<Button
									v-tooltip="sftpCopyTooltip('Copy SFTP username')"
									native-type="button"
									size="lg"
									class="w-full !justify-between text-left"
									:disabled="!canWriteFiles"
									@click="copyToClipboard('Username', server?.sftp_username)"
								>
									<span class="min-w-0 truncate text-base font-semibold text-primary">
										{{ server?.sftp_username }}
									</span>
									<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
								</Button>
							</div>
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-lg font-semibold text-contrast">Password</span>
								<div class="flex items-center gap-1.5">
									<Button
										v-tooltip="sftpCopyTooltip('Copy SFTP password')"
										native-type="button"
										size="lg"
										class="min-w-0 grow !justify-between text-left"
										:disabled="!canWriteFiles"
										@click="copyToClipboard('Password', server?.sftp_password)"
									>
										<span class="min-w-0 truncate text-base font-semibold text-primary">
											{{
												showPassword
													? server?.sftp_password
													: '*'.repeat(server?.sftp_password?.length ?? 0)
											}}
										</span>
										<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
									</Button>
									<IconButton
										v-tooltip="passwordVisibilityTooltip"
										:label="passwordVisibilityTooltip"
										size="lg"
										:disabled="!canWriteFiles"
										@click="togglePasswordVisibility"
									>
										<EyeIcon v-if="showPassword" />
										<EyeOffIcon v-else />
									</IconButton>
								</div>
							</div>
						</div>
					</div>
				</div>

				<!-- Startup command section -->
				<div class="flex flex-col gap-2.5">
					<div class="flex h-10 flex-col items-end justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="mb-0.5 flex flex-col gap-2">
							<span class="text-lg font-semibold text-contrast">Startup command</span>
						</label>
						<Button
							v-if="startupCommand !== defaultStartupCommand"
							v-tooltip="advancedActionTooltip"
							type="quiet"
							:disabled="
								isStartupLoading ||
								startupCommand === defaultStartupCommand ||
								!canUseAdvancedSettings
							"
							class="relative !w-full sm:!w-auto"
							@click="resetToDefault"
						>
							<UpdatedIcon class="h-5 w-5" />
							Default
						</Button>
					</div>
					<div class="relative">
						<StyledInput
							id="startup-command-field"
							v-model="startupCommand"
							v-tooltip="advancedActionTooltip"
							multiline
							resize="vertical"
							input-class="font-mono field-sizing-content"
							:disabled="isStartupLoading || !canUseAdvancedSettings"
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
				<div class="flex flex-col gap-2.5">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-semibold text-contrast">Java version</span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'java-version-field'"
							v-model="javaVersion"
							v-tooltip="advancedActionTooltip"
							name="java-version"
							:options="displayedJavaVersions"
							:display-value="javaVersionLabel ?? 'Java Version'"
							:disabled="isStartupLoading || !canUseAdvancedSettings"
							trigger-type="base"
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
					<span> The Java version your server runs on. </span>
				</div>

				<!-- Java runtime section -->
				<div class="flex flex-col gap-2.5">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-semibold text-contrast">Java runtime</span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'runtime-field'"
							v-model="jreVendor"
							v-tooltip="advancedActionTooltip"
							name="runtime"
							:options="JRE_VENDORS"
							:display-value="jreVendorLabel ?? 'Runtime'"
							:disabled="isStartupLoading || !canUseAdvancedSettings"
							trigger-type="base"
						/>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
						</div>
					</div>
					<span> The Java runtime your server will use. </span>
				</div>
			</div>
		</div>
		<SaveBanner
			:is-visible="!!hasUnsavedChanges || isPending"
			:server-id="serverId"
			:is-updating="isPending"
			:save="saveStartup"
			:reset="resetStartup"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	ExternalIcon,
	EyeIcon,
	EyeOffIcon,
	SpinnerIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { Combobox, StyledInput } from '#ui/components'
import { Button, ButtonLink, IconButton } from '#ui/components/base/buttons'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { addNotification } = injectNotificationManager()
const { server, serverId, worldId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { canUseAdvancedSettings, canWriteFiles, permissionDeniedMessage } = useServerPermissions()

const showPassword = ref(false)
const sftpUrl = computed(() => `sftp://${server.value?.sftp_username}@${server.value?.sftp_host}`)
const advancedActionTooltip = computed(() =>
	canUseAdvancedSettings.value ? undefined : permissionDeniedMessage.value,
)
const sftpActionTooltip = computed(() =>
	canWriteFiles.value
		? 'This button only works with compatible SFTP clients (e.g. WinSCP)'
		: permissionDeniedMessage.value,
)
const sftpCopyTooltip = (label: string) =>
	canWriteFiles.value ? label : permissionDeniedMessage.value
const passwordVisibilityTooltip = computed(() =>
	canWriteFiles.value
		? showPassword.value
			? 'Hide password'
			: 'Show password'
		: permissionDeniedMessage.value,
)

function handleSftpLaunchClick(event: MouseEvent) {
	if (canWriteFiles.value) return
	event.preventDefault()
}

const copyToClipboard = (name: string, textToCopy?: string) => {
	if (!canWriteFiles.value) return
	navigator.clipboard.writeText(textToCopy || '')
	addNotification({
		type: 'success',
		title: `${name} copied to clipboard!`,
	})
}

function togglePasswordVisibility() {
	if (!canWriteFiles.value) return
	showPassword.value = !showPassword.value
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
	{ value: 25, label: 'Java 25' },
]

const showAllVersions = ref(false)

type MinecraftReleaseVersion = {
	major: number
	minor: number
}

function parseMinecraftReleaseVersion(version: string): MinecraftReleaseVersion | null {
	const [majorPart, minorPart] = version.split('.')

	if (!majorPart || !minorPart) return null

	const major = Number(majorPart)
	const minor = Number(minorPart)

	if (!Number.isInteger(major) || !Number.isInteger(minor)) return null

	return { major, minor }
}

function filterJavaVersions(compatibleVersions: number[]) {
	return JAVA_VERSIONS.filter((version) => compatibleVersions.includes(version.value))
}

const displayedJavaVersions = computed(() => {
	if (showAllVersions.value) return JAVA_VERSIONS

	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return JAVA_VERSIONS

	const releaseVersion = parseMinecraftReleaseVersion(mcVersion)
	if (!releaseVersion) return JAVA_VERSIONS

	if (releaseVersion.major > 1) {
		if (releaseVersion.major >= 26) {
			return filterJavaVersions([25])
		}

		return JAVA_VERSIONS
	}

	if (releaseVersion.minor >= 20) return filterJavaVersions([21])
	if (releaseVersion.minor >= 17) return filterJavaVersions([17, 21])
	if (releaseVersion.minor >= 12) return filterJavaVersions([8, 11, 17, 21])
	if (releaseVersion.minor >= 6) return filterJavaVersions([8, 11])
	return filterJavaVersions([8])
})

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

const { mutate: saveStartupMutation, isPending } = useMutation({
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

function saveStartup() {
	if (!canUseAdvancedSettings.value) return
	saveStartupMutation()
}

function resetStartup() {
	syncFormFromData()
}

function resetToDefault() {
	if (!canUseAdvancedSettings.value) return
	startupCommand.value = defaultStartupCommand.value
}
</script>
