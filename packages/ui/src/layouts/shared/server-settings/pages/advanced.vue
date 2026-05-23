<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div class="flex flex-col gap-6">
				<div class="flex flex-col gap-2">
					<div class="flex flex-col items-center justify-between gap-0.5 sm:flex-row">
						<span class="text-lg font-semibold text-contrast">SFTP</span>
						<ButtonStyled>
							<a
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
							</a>
						</ButtonStyled>
					</div>

					<div class="flex flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
						<span class="text-lg font-semibold text-contrast">Server Address</span>
						<div
							v-tooltip="sftpCopyTooltip('Copy SFTP server address')"
							class="copy-field hover:bg-button-bg-hover"
							:class="{ 'opacity-60': !canWriteFiles }"
							@click="copyToClipboard('Server address', server?.sftp_host)"
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
								<span class="text-lg font-semibold text-contrast">Username</span>
								<div
									v-tooltip="sftpCopyTooltip('Copy SFTP username')"
									class="copy-field hover:bg-button-bg-hover"
									:class="{ 'opacity-60': !canWriteFiles }"
									@click="copyToClipboard('Username', server?.sftp_username)"
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
								<span class="text-lg font-semibold text-contrast">Password</span>
								<div
									class="copy-field-has-button [&:hover:not(:has(button:hover))]:bg-button-bg-hover"
									:class="{ 'opacity-60': !canWriteFiles }"
									@click="copyToClipboard('Password', server?.sftp_password)"
								>
									<div class="flex items-center gap-1.5 h-full w-full">
										<div
											v-tooltip="sftpCopyTooltip('Copy SFTP Password')"
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
													canWriteFiles
														? showPassword
															? 'Hide password'
															: 'Show password'
														: permissionDeniedMessage
												"
												class="hover:bg-button-bg-hover grid h-10 w-10 place-content-center rounded-lg"
												:disabled="!canWriteFiles"
												@click.stop="togglePasswordVisibility"
											>
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
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CopyIcon, ExternalIcon, EyeIcon, EyeOffIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { ButtonStyled } from '#ui/components'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { addNotification } = injectNotificationManager()
const { server } = injectModrinthServerContext()
const { canWriteFiles, permissionDeniedMessage } = useServerPermissions()

const showPassword = ref(false)
const sftpUrl = computed(() => `sftp://${server.value?.sftp_username}@${server.value?.sftp_host}`)
const sftpActionTooltip = computed(() =>
	canWriteFiles.value
		? 'This button only works with compatible SFTP clients (e.g. WinSCP)'
		: permissionDeniedMessage.value,
)
const sftpCopyTooltip = (label: string) =>
	canWriteFiles.value ? label : permissionDeniedMessage.value

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
