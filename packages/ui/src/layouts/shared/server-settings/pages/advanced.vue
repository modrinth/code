<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div class="flex flex-col gap-6">
				<!-- SFTP section -->
				<div class="flex flex-col gap-2">
					<div class="flex flex-col items-center justify-between gap-0.5 sm:flex-row">
						<span class="text-lg font-semibold text-contrast">{{
							formatMessage(messages.sftpSectionTitle)
						}}</span>
						<ButtonStyled>
							<a
								v-tooltip="formatMessage(messages.sftpLaunchTooltip)"
								class="!w-full sm:!w-auto"
								:href="sftpUrl"
								target="_blank"
							>
								<ExternalIcon class="h-5 w-5" />
								{{ formatMessage(messages.launchSftpButton) }}
							</a>
						</ButtonStyled>
					</div>

					<div class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
						<span class="text-lg font-semibold text-contrast">{{
							formatMessage(messages.serverAddressLabel)
						}}</span>
						<div
							v-tooltip="formatMessage(messages.copySftpAddressTooltip)"
							class="copy-field hover:bg-button-bg-hover"
							@click="
								copyToClipboard(formatMessage(messages.serverAddressLabel), server?.sftp_host)
							"
						>
							<span class="cursor-pointer font-semibold text-primary">
								{{ server?.sftp_host }}
							</span>
							<div class="grid h-10 w-10 place-content-center">
								<CopyIcon class="h-5 w-5" />
							</div>
						</div>
						<div class="flex flex-col gap-2 sm:mt-0 sm:flex-row">
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-lg font-semibold text-contrast">{{
									formatMessage(commonMessages.usernameLabel)
								}}</span>
								<div
									v-tooltip="formatMessage(messages.copySftpUsernameTooltip)"
									class="copy-field hover:bg-button-bg-hover"
									@click="
										copyToClipboard(
											formatMessage(commonMessages.usernameLabel),
											server?.sftp_username,
										)
									"
								>
									<div class="truncate font-semibold">
										{{ server?.sftp_username }}
									</div>
									<div class="grid h-10 w-9 place-content-center">
										<CopyIcon class="h-5 w-5" />
									</div>
								</div>
							</div>
							<div class="flex w-full flex-col justify-center gap-2">
								<span class="text-lg font-semibold text-contrast">{{
									formatMessage(commonMessages.passwordLabel)
								}}</span>
								<div
									class="copy-field-has-button [&:hover:not(:has(button:hover))]:bg-button-bg-hover"
									@click="
										copyToClipboard(
											formatMessage(commonMessages.passwordLabel),
											server?.sftp_password,
										)
									"
								>
									<div class="flex items-center gap-1.5 h-full w-full">
										<div
											v-tooltip="formatMessage(messages.copySftpPasswordTooltip)"
											class="h-full flex justify-between grow items-center"
										>
											<div class="truncate font-semibold">
												{{
													showPassword
														? server?.sftp_password
														: '*'.repeat(server?.sftp_password?.length ?? 0)
												}}
											</div>
											<CopyIcon class="h-5 w-5" />
										</div>

										<ButtonStyled type="transparent" circular>
											<button
												v-tooltip="
													showPassword
														? formatMessage(messages.hidePasswordTooltip)
														: formatMessage(messages.showPasswordTooltip)
												"
												class="hover:bg-button-bg-hover grid h-10 w-10 place-content-center rounded-lg"
												@click.stop="showPassword = !showPassword"
											>
												<!-- look into doing stop propagation here -->
												<EyeIcon v-if="showPassword" class="h-5 w-5" />
												<EyeOffIcon v-else class="h-5 w-5" />
											</button>
										</ButtonStyled>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>

				<!-- Startup command section -->
				<div class="flex flex-col gap-2.5">
					<div class="flex h-10 flex-col items-end justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="mb-0.5 flex flex-col gap-2">
							<span class="text-lg font-semibold text-contrast">{{
								formatMessage(messages.startupCommandLabel)
							}}</span>
						</label>
						<ButtonStyled v-if="startupCommand !== defaultStartupCommand" type="transparent">
							<button
								:disabled="isStartupLoading || startupCommand === defaultStartupCommand"
								class="relative !w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								{{ formatMessage(messages.defaultStartupButton) }}
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
					<span>{{ formatMessage(messages.startupCommandDescription) }}</span>
				</div>

				<!-- Java version section -->
				<div class="flex flex-col gap-2.5">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-semibold text-contrast">{{
							formatMessage(messages.javaVersionLabel)
						}}</span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'java-version-field'"
							v-model="javaVersion"
							name="java-version"
							:options="displayedJavaVersions"
							:display-value="
								javaVersionLabel ?? formatMessage(messages.javaVersionComboboxFallback)
							"
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
									{{
										showAllVersions
											? formatMessage(messages.hideExtraJavaVersions)
											: formatMessage(messages.showAllJavaVersions)
									}}
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
					<span>{{ formatMessage(messages.javaVersionDescription) }}</span>
				</div>

				<!-- Java runtime section -->
				<div class="flex flex-col gap-2.5">
					<div class="flex flex-col gap-2">
						<span class="text-lg font-semibold text-contrast">{{
							formatMessage(messages.javaRuntimeLabel)
						}}</span>
					</div>
					<div class="relative max-w-xs">
						<Combobox
							:id="'runtime-field'"
							v-model="jreVendor"
							name="runtime"
							:options="JRE_VENDOR_OPTIONS"
							:display-value="jreVendorLabel ?? formatMessage(messages.javaRuntimeComboboxFallback)"
							:disabled="isStartupLoading"
						/>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
						</div>
					</div>
					<span>{{ formatMessage(messages.javaRuntimeDescription) }}</span>
				</div>
			</div>
		</div>
		<SaveBanner
			:is-visible="!!hasUnsavedChanges || isPending"
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
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { ButtonStyled, Combobox, StyledInput } from '#ui/components'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	sftpSectionTitle: {
		id: 'server.settings.advanced.sftp.title',
		defaultMessage: 'SFTP',
	},
	sftpLaunchTooltip: {
		id: 'server.settings.advanced.sftp.launch-tooltip',
		defaultMessage: 'This button only works with compatible SFTP clients (e.g. WinSCP)',
	},
	launchSftpButton: {
		id: 'server.settings.advanced.sftp.launch',
		defaultMessage: 'Launch SFTP',
	},
	serverAddressLabel: {
		id: 'server.settings.advanced.sftp.server-address',
		defaultMessage: 'Server Address',
	},
	copySftpAddressTooltip: {
		id: 'server.settings.advanced.sftp.copy-address-tooltip',
		defaultMessage: 'Copy SFTP server address',
	},
	copySftpUsernameTooltip: {
		id: 'server.settings.advanced.sftp.copy-username-tooltip',
		defaultMessage: 'Copy SFTP username',
	},
	copySftpPasswordTooltip: {
		id: 'server.settings.advanced.sftp.copy-password-tooltip',
		defaultMessage: 'Copy SFTP password',
	},
	showPasswordTooltip: {
		id: 'server.settings.advanced.sftp.show-password-tooltip',
		defaultMessage: 'Show password',
	},
	hidePasswordTooltip: {
		id: 'server.settings.advanced.sftp.hide-password-tooltip',
		defaultMessage: 'Hide password',
	},
	startupCommandLabel: {
		id: 'server.settings.advanced.startup-command.title',
		defaultMessage: 'Startup command',
	},
	defaultStartupButton: {
		id: 'server.settings.advanced.startup-command.default',
		defaultMessage: 'Default',
	},
	startupCommandDescription: {
		id: 'server.settings.advanced.startup-command.description',
		defaultMessage: 'The command that runs when your server is started.',
	},
	javaVersionLabel: {
		id: 'server.settings.advanced.java-version.title',
		defaultMessage: 'Java version',
	},
	javaVersionComboboxFallback: {
		id: 'server.settings.advanced.java-version.fallback',
		defaultMessage: 'Java version',
	},
	javaVersionDescription: {
		id: 'server.settings.advanced.java-version.description',
		defaultMessage: 'The Java version your server runs on.',
	},
	showAllJavaVersions: {
		id: 'server.settings.advanced.java-version.show-all',
		defaultMessage: 'Show all versions',
	},
	hideExtraJavaVersions: {
		id: 'server.settings.advanced.java-version.hide-extra',
		defaultMessage: 'Hide extra versions',
	},
	javaRuntimeLabel: {
		id: 'server.settings.advanced.java-runtime.title',
		defaultMessage: 'Java runtime',
	},
	javaRuntimeComboboxFallback: {
		id: 'server.settings.advanced.java-runtime.fallback',
		defaultMessage: 'Runtime',
	},
	javaRuntimeDescription: {
		id: 'server.settings.advanced.java-runtime.description',
		defaultMessage: 'The Java runtime your server will use.',
	},
	clipboardCopiedTitle: {
		id: 'server.settings.advanced.clipboard.copied.title',
		defaultMessage: '{label} copied to clipboard!',
	},
	startupUpdateFailedTitle: {
		id: 'server.settings.advanced.error.startup.title',
		defaultMessage: 'Failed to update server arguments',
	},
	startupUpdateFailedText: {
		id: 'server.settings.advanced.error.startup.text',
		defaultMessage: 'Please try again later.',
	},
})
const { server, serverId, worldId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()

// SFTP state
const showPassword = ref(false)
const sftpUrl = computed(() => `sftp://${server.value?.sftp_username}@${server.value?.sftp_host}`)

const copyToClipboard = (label: string, textToCopy?: string) => {
	navigator.clipboard.writeText(textToCopy || '')
	addNotification({
		type: 'success',
		title: formatMessage(messages.clipboardCopiedTitle, { label }),
	})
}

// Startup state
const startupQueryKey = computed(() => ['servers', 'startup', 'v1', serverId, worldId.value])

const { data: startupData, isLoading: isStartupLoading } = useQuery({
	queryKey: startupQueryKey,
	queryFn: () => client.archon.options_v1.getStartup(serverId, worldId.value!),
	enabled: computed(() => worldId.value !== null),
})

const JAVA_VERSION_OPTIONS: { value: number; label: string }[] = [
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
	return JAVA_VERSION_OPTIONS.filter((version) => compatibleVersions.includes(version.value))
}

const displayedJavaVersions = computed(() => {
	if (showAllVersions.value) return JAVA_VERSION_OPTIONS

	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return JAVA_VERSION_OPTIONS

	const releaseVersion = parseMinecraftReleaseVersion(mcVersion)
	if (!releaseVersion) return JAVA_VERSION_OPTIONS

	if (releaseVersion.major > 1) {
		if (releaseVersion.major >= 26) {
			return filterJavaVersions([25])
		}

		return JAVA_VERSION_OPTIONS
	}

	if (releaseVersion.minor >= 20) return filterJavaVersions([21])
	if (releaseVersion.minor >= 17) return filterJavaVersions([17, 21])
	if (releaseVersion.minor >= 12) return filterJavaVersions([8, 11, 17, 21])
	if (releaseVersion.minor >= 6) return filterJavaVersions([8, 11])
	return filterJavaVersions([8])
})

const JRE_VENDOR_OPTIONS: { value: Archon.Content.v1.JreVendor; label: string }[] = [
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
	() => JAVA_VERSION_OPTIONS.find((v) => v.value === javaVersion.value)?.label,
)
const jreVendorLabel = computed(
	() => JRE_VENDOR_OPTIONS.find((v) => v.value === jreVendor.value)?.label,
)

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
			title: formatMessage(commonMessages.serverSettingsUpdatedTitle),
			text: formatMessage(commonMessages.serverSettingsUpdatedText),
		})
	},
	onError: (error) => {
		console.error(error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.startupUpdateFailedTitle),
			text: formatMessage(messages.startupUpdateFailedText),
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

<style scoped>
.copy-field {
	@apply flex h-10 cursor-pointer items-center justify-between gap-2 rounded-lg bg-button-bg px-3 pr-1.5 transition-all;
	@apply hover:brightness-125 active:scale-95;
}

.copy-field-has-button {
	@apply flex h-10 cursor-pointer items-center justify-between gap-2 rounded-lg bg-button-bg px-3 pr-1.5 transition-all;
	@apply [&:hover:not(:has(button:hover))]:brightness-125 [&:active:not(:has(button:active))]:scale-95;
}
</style>
