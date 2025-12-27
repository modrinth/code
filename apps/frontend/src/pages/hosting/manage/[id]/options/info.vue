<template>
	<div class="h-full w-full gap-2 overflow-y-auto">
		<div class="card">
			<div class="flex flex-col gap-4">
				<div class="flex flex-col justify-between gap-4 sm:flex-row">
					<label class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">SFTP</span>
						<span> SFTP allows you to access your server's files from outside of Modrinth. </span>
					</label>
					<ButtonStyled>
						<button
							v-tooltip="'This button only works with compatible SFTP clients (e.g. WinSCP)'"
							class="!w-full sm:!w-auto"
							@click="openSftp"
						>
							<ExternalIcon class="h-5 w-5" />
							Launch SFTP
						</button>
					</ButtonStyled>
				</div>

				<div
					class="flex w-full flex-row justify-between gap-2 rounded-xl bg-table-alternateRow p-4"
				>
					<div class="flex flex-col gap-2">
						<span class="cursor-pointer font-bold text-contrast">
							{{ server.sftp_host }}
						</span>

						<span class="text-xs text-secondary">Server Address</span>
					</div>

					<ButtonStyled type="transparent">
						<button
							v-tooltip="'Copy SFTP server address'"
							@click="copyToClipboard('Server address', server.sftp_host)"
						>
							<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
						</button>
					</ButtonStyled>
				</div>
				<div class="-mt-2 flex flex-col gap-2 sm:mt-0 sm:flex-row">
					<div
						class="flex w-full flex-col justify-center gap-2 rounded-xl bg-table-alternateRow px-4 py-2"
					>
						<div class="flex h-8 items-center justify-between">
							<span class="font-bold text-contrast">
								{{ server.sftp_username }}
							</span>

							<ButtonStyled type="transparent">
								<button
									v-tooltip="'Copy SFTP username'"
									@click="copyToClipboard('Username', server.sftp_username)"
								>
									<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
								</button>
							</ButtonStyled>
						</div>
						<span class="text-xs text-secondary">Username</span>
					</div>
					<div
						class="flex w-full flex-col justify-center gap-2 rounded-xl bg-table-alternateRow p-4"
					>
						<div class="flex h-8 items-center justify-between">
							<span class="font-bold text-contrast">
								{{
									showPassword ? server.sftp_password : '*'.repeat(server.sftp_password?.length ?? 0)
								}}
							</span>

							<div class="flex flex-row items-center gap-1">
								<ButtonStyled type="transparent">
									<button
										v-tooltip="'Copy SFTP password'"
										@click="copyToClipboard('Password', server.sftp_password)"
									>
										<CopyIcon class="h-5 w-5 hover:cursor-pointer" />
									</button>
								</ButtonStyled>
								<ButtonStyled type="transparent">
									<button
										v-tooltip="showPassword ? 'Hide password' : 'Show password'"
										@click="togglePassword"
									>
										<EyeIcon v-if="showPassword" class="h-5 w-5 hover:cursor-pointer" />
										<EyeOffIcon v-else class="h-5 w-5 hover:cursor-pointer" />
									</button>
								</ButtonStyled>
							</div>
						</div>
						<span class="text-xs text-secondary">Password</span>
					</div>
				</div>
			</div>
		</div>
		<div class="card">
			<h2 class="text-xl font-bold">Info</h2>
			<div class="rounded-xl bg-table-alternateRow p-4">
				<table
					class="min-w-full border-collapse overflow-hidden rounded-lg border-2 border-gray-300"
				>
					<tbody>
						<tr v-for="property in properties" :key="property.name">
							<td v-if="property.value !== 'Unknown'" class="py-3">
								{{ property.name }}
							</td>
							<td v-if="property.value !== 'Unknown'" class="px-4">
								<CopyCode :text="property.value" />
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CopyIcon, ExternalIcon, EyeIcon, EyeOffIcon } from '@modrinth/assets'
import { ButtonStyled, CopyCode, injectModrinthServerContext, injectNotificationManager } from '@modrinth/ui'

const { addNotification } = injectNotificationManager()
const { server, serverId } = injectModrinthServerContext()

const showPassword = ref(false)

function openSftp() {
	const sftpUrl = `sftp://${server.value.sftp_username}@${server.value.sftp_host}`
	window.open(sftpUrl, '_blank')
}

function togglePassword() {
	showPassword.value = !showPassword.value
}

function copyToClipboard(name: string, textToCopy?: string) {
	navigator.clipboard.writeText(textToCopy || '')
	addNotification({
		type: 'success',
		title: `${name} copied to clipboard!`,
	})
}

const properties = computed(() => [
	{ name: 'Server ID', value: serverId ?? 'Unknown' },
	{ name: 'Node', value: server.value.node?.instance ?? 'Unknown' },
	{ name: 'Kind', value: server.value.upstream?.kind ?? server.value.loader ?? 'Unknown' },
	{ name: 'Project ID', value: server.value.upstream?.project_id ?? 'Unknown' },
	{ name: 'Version ID', value: server.value.upstream?.version_id ?? 'Unknown' },
])
</script>
