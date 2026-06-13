<template>
	<div class="contents">
		<ConfirmModal
			ref="removePasskeyModal"
			:title="formatMessage(messages.removePasskeyConfirmTitle)"
			:description="
				formatMessage(messages.removePasskeyConfirmDescription, { name: passkeyToRemove?.name })
			"
			:proceed-label="formatMessage(commonMessages.removeButton)"
			@proceed="removePasskey()"
		/>

		<NewModal
			ref="managePasskeyModal"
			width="600px"
			:header="formatMessage(messages.managePasskeyTitle)"
		>
			<div class="flex flex-col gap-4">
				<div v-if="passkeysLoading" class="flex items-center gap-2 text-secondary">
					<SpinnerIcon class="animate-spin" />
					{{ formatMessage(messages.managePasskeyModalLoading) }}
				</div>
				<template v-else>
					<p v-if="passkeys.length === 0" class="m-0 text-secondary">
						{{ formatMessage(messages.managePasskeyModalNoPasskeys) }}
					</p>
					<div
						v-for="passkey in passkeys"
						:key="passkey.id"
						class="universal-card recessed passkey"
					>
						<div>
							<div>
								<strong>{{ passkey.name }}</strong>
							</div>
							<div>
								<span v-tooltip="formatDateTime(passkey.created_at)">
									{{
										formatMessage(messages.managePasskeyModalAdded, {
											ago: formatRelativeTime(passkey.created_at),
										})
									}}
								</span>
								⋅
								<span v-if="passkey.last_used" v-tooltip="formatDateTime(passkey.last_used)">
									{{
										formatMessage(messages.managePasskeyModalLastUsed, {
											ago: formatRelativeTime(passkey.last_used),
										})
									}}
								</span>
								<span v-else>
									{{ formatMessage(messages.managePasskeyModalNeverUsed) }}
								</span>
							</div>
						</div>
						<div class="passkey-actions flex gap-2">
							<ButtonStyled>
								<button
									@click="
										() => {
											passkeyToRename = { ...passkey }
											$refs.renamePasskeyModal.show()
										}
									"
								>
									<EditIcon /> {{ formatMessage(commonMessages.renameButton) }}
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<button
									@click="
										() => {
											passkeyToRemove = passkey
											$refs.removePasskeyModal.show()
										}
									"
								>
									<TrashIcon /> {{ formatMessage(commonMessages.removeButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</template>
				<div class="input-group self-end">
					<ButtonStyled color="brand">
						<button @click="registerPasskey()">
							<PlusIcon />
							{{ formatMessage(messages.managePasskeyAddPasskey) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="$refs.managePasskeyModal.hide()">
							<XIcon />
							{{ formatMessage(commonMessages.closeButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

		<NewModal
			ref="addPasskeyModal"
			width="500px"
			:header="formatMessage(messages.managePasskeyAddPasskey)"
		>
			<div class="flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<label for="passkey-name">
						<div class="text-md mb-2 font-semibold text-contrast">
							{{ formatMessage(messages.passkeyNameLabel) }}
						</div>
						<div class="label__description mb-2">
							{{ formatMessage(messages.passkeyNameDescription) }}
						</div>
					</label>
					<StyledInput
						id="passkey-name"
						v-model="pendingPasskey.name"
						:maxlength="255"
						type="text"
						:placeholder="formatMessage(messages.passkeyNamePlaceholder)"
						@keyup.enter="pendingPasskey.name && finishRegisterPasskey()"
					/>
				</div>
				<div class="input-group push-right">
					<ButtonStyled>
						<button @click="$refs.addPasskeyModal.hide()">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="!pendingPasskey.name" @click="finishRegisterPasskey()">
							<PlusIcon />
							{{ formatMessage(messages.managePasskeyAddPasskey) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

		<NewModal
			ref="renamePasskeyModal"
			width="500px"
			:header="formatMessage(messages.renamePasskeyModalHeader)"
		>
			<div class="flex flex-col gap-6">
				<div class="flex flex-col gap-2.5">
					<label for="passkey-rename">
						<div class="text-md mb-2 font-semibold text-contrast">
							{{ formatMessage(messages.passkeyNameLabel) }}
						</div>
						<div class="label__description mb-2">
							{{ formatMessage(messages.passkeyNameDescription) }}
						</div>
					</label>
					<StyledInput
						id="passkey-rename"
						v-model="passkeyToRename.name"
						:maxlength="255"
						type="text"
						:placeholder="formatMessage(messages.passkeyNamePlaceholder)"
						@keyup.enter="passkeyToRename.name && renamePasskey()"
					/>
				</div>
				<div class="input-group push-right">
					<ButtonStyled>
						<button @click="$refs.renamePasskeyModal.hide()">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="!passkeyToRename.name" @click="renamePasskey()">
							<SaveIcon />
							{{ formatMessage(commonMessages.saveButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

		<div class="adjacent-input mb-0">
			<label for="manage-passkeys">
				<span class="label__title">{{ formatMessage(messages.managePasskeyTitle) }}</span>
				<span class="label__description">{{
					formatMessage(messages.managePasskeyDescription)
				}}</span>
			</label>
			<div>
				<ButtonStyled>
					<button id="manage-passkeys" @click="showPasskeyModal">
						<UserKeyIcon /> {{ formatMessage(messages.managePasskeyTitle) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	EditIcon,
	PlusIcon,
	SaveIcon,
	SpinnerIcon,
	TrashIcon,
	UserKeyIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	StyledInput,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'

import { createPasskeyCredential } from '~/helpers/passkey.ts'

const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

function notifyError(err: unknown) {
	addNotification({
		title: formatMessage(commonMessages.errorNotificationTitle),
		text: err instanceof Error ? err.message : String(err),
		type: 'error',
	})
}

const messages = defineMessages({
	managePasskeyTitle: {
		id: 'settings.account.security.passkey.title',
		defaultMessage: 'Manage passkeys',
	},
	managePasskeyDescription: {
		id: 'settings.account.security.passkey.description',
		defaultMessage: 'Manage your registered passkeys, or add a new one.',
	},
	managePasskeyAddPasskey: {
		id: 'settings.account.security.passkey.add',
		defaultMessage: 'Add passkey',
	},
	managePasskeyModalLoading: {
		id: 'settings.account.security.passkey.modal.loading',
		defaultMessage: 'Loading passkeys…',
	},
	managePasskeyModalNoPasskeys: {
		id: 'settings.account.security.passkey.modal.no-passkeys',
		defaultMessage: 'You have no passkeys registered yet.',
	},
	managePasskeyModalAdded: {
		id: 'settings.account.security.passkey.modal.added',
		defaultMessage: 'Added {ago}',
	},
	managePasskeyModalLastUsed: {
		id: 'settings.account.security.passkey.modal.last-used',
		defaultMessage: 'Last used {ago}',
	},
	managePasskeyModalNeverUsed: {
		id: 'settings.account.security.passkey.modal.never-used',
		defaultMessage: 'Never used',
	},
	passkeyNameLabel: {
		id: 'settings.account.security.passkey.add-modal.name.label',
		defaultMessage: 'Name',
	},
	passkeyNameDescription: {
		id: 'settings.account.security.passkey.add-modal.name.description',
		defaultMessage:
			'Make sure to pick something memorable, so you can identify this passkey later.',
	},
	passkeyNamePlaceholder: {
		id: 'settings.account.security.passkey.add-modal.name.placeholder',
		defaultMessage: 'My passkey',
	},
	renamePasskeyModalHeader: {
		id: 'settings.account.security.passkey.rename-modal.header',
		defaultMessage: 'Rename passkey',
	},
	removePasskeyConfirmTitle: {
		id: 'settings.account.security.passkey.remove.title',
		defaultMessage: 'Are you sure you want to remove this passkey?',
	},
	removePasskeyConfirmDescription: {
		id: 'settings.account.security.passkey.remove.description',
		defaultMessage:
			'This will permanently remove the passkey "{name}". You will no longer be able to sign in with it.',
	},
})

const managePasskeyModal = ref()
const renamePasskeyModal = ref()
const addPasskeyModal = ref()

const pendingPasskey = ref<Labrinth.Auth.v2.PasskeyRegisterFinishRequest | null>(null)
const passkeyToRemove = ref<Labrinth.Auth.v2.Passkey | null>(null)
const passkeyToRename = ref<Labrinth.Auth.v2.Passkey | null>(null)

const passkeys = ref<Labrinth.Auth.v2.Passkey[]>([])
const passkeysLoading = ref(false)

async function fetchPasskeys() {
	passkeysLoading.value = true
	try {
		passkeys.value = await labrinth.auth_v2.listPasskeys()
	} catch (err) {
		notifyError(err)
	}
	passkeysLoading.value = false
}

async function showPasskeyModal() {
	managePasskeyModal.value.show()
	await fetchPasskeys()
}

async function registerPasskey() {
	startLoading()
	try {
		const res = await labrinth.auth_v2.registerPasskeyStart()

		const credential = await createPasskeyCredential(res.options.publicKey)
		pendingPasskey.value = {
			flow: res.flow,
			credential: credential,
			name: '',
		}

		addPasskeyModal.value.show()
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}

async function finishRegisterPasskey() {
	if (!pendingPasskey.value) return

	startLoading()
	try {
		const passkey = await labrinth.auth_v2.registerPasskeyFinish(pendingPasskey.value)
		passkeys.value.unshift(passkey)
		pendingPasskey.value = null
		addPasskeyModal.value.hide()
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}

async function renamePasskey() {
	if (!passkeyToRename.value) return
	const { id, name } = passkeyToRename.value

	startLoading()
	try {
		await labrinth.auth_v2.renamePasskey(id, { name })
		const existing = passkeys.value.find((passkey) => passkey.id === id)
		if (existing) {
			existing.name = name
		}
		passkeyToRename.value = null
		renamePasskeyModal.value.hide()
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}

async function removePasskey() {
	if (!passkeyToRemove.value) return
	const { id } = passkeyToRemove.value

	startLoading()
	try {
		await labrinth.auth_v2.deletePasskey(id)
		passkeys.value = passkeys.value.filter((passkey) => passkey.id !== id)
		passkeyToRemove.value = null
	} catch (err) {
		notifyError(err)
	}
	stopLoading()
}
</script>

<style lang="scss" scoped>
.passkey {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;

	@media screen and (min-width: 650px) {
		flex-direction: row;
		align-items: center;
	}
}

.passkey-actions {
	flex-direction: row;

	@media screen and (min-width: 650px) {
		flex-direction: column;
		margin-left: auto;
	}
}
</style>
