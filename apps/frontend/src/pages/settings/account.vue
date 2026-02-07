<template>
	<div>
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(messages.deleteAccountConfirmTitle)"
			:description="formatMessage(messages.deleteAccountConfirmDescription)"
			:proceed-label="formatMessage(messages.deleteAccountConfirmProceed)"
			:confirmation-text="auth.user.username"
			:has-to-type="true"
			@proceed="deleteAccount"
		/>
		<Modal
			ref="changeEmailModal"
			:header="`${auth.user.email ? formatMessage(messages.changeEmailHeaderChange) : formatMessage(messages.changeEmailHeaderAdd)}`"
		>
			<div class="universal-modal">
				<p>{{ formatMessage(messages.emailNotPublicNotice) }}</p>
				<label for="email-input">
					<span class="label__title">{{ formatMessage(messages.emailAddressLabel) }}</span>
				</label>
				<input
					id="email-input"
					v-model="email"
					maxlength="2048"
					type="email"
					:placeholder="formatMessage(messages.emailAddressPlaceholder)"
					@keyup.enter="saveEmail()"
				/>
				<div class="input-group push-right">
					<button class="iconified-button" @click="$refs.changeEmailModal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
					<button
						type="button"
						class="iconified-button brand-button"
						:disabled="!email"
						@click="saveEmail()"
					>
						<SaveIcon />
						{{ formatMessage(messages.saveEmailButton) }}
					</button>
				</div>
			</div>
		</Modal>
		<Modal
			ref="managePasswordModal"
			:header="`${
				removePasswordMode
					? formatMessage(messages.passwordHeaderRemove)
					: auth.user.has_password
						? formatMessage(messages.passwordHeaderChange)
						: formatMessage(messages.passwordHeaderAdd)
			}`"
		>
			<div class="universal-modal">
				<ul
					v-if="newPassword !== confirmNewPassword && confirmNewPassword.length > 0"
					class="known-errors"
				>
					<li>{{ formatMessage(messages.passwordsDoNotMatchError) }}</li>
				</ul>
				<label v-if="removePasswordMode" for="old-password">
					<span class="label__title">{{ formatMessage(messages.confirmPasswordLabel) }}</span>
					<span class="label__description">{{
						formatMessage(messages.confirmPasswordDescription)
					}}</span>
				</label>
				<label v-else-if="auth.user.has_password" for="old-password">
					<span class="label__title">{{ formatMessage(messages.oldPasswordLabel) }}</span>
				</label>
				<input
					v-if="auth.user.has_password"
					id="old-password"
					v-model="oldPassword"
					maxlength="2048"
					type="password"
					autocomplete="current-password"
					:placeholder="
						removePasswordMode
							? formatMessage(messages.confirmPasswordPlaceholder)
							: formatMessage(messages.oldPasswordPlaceholder)
					"
				/>
				<template v-if="!removePasswordMode">
					<label for="new-password"
						><span class="label__title">{{ formatMessage(messages.newPasswordLabel) }}</span></label
					>
					<input
						id="new-password"
						v-model="newPassword"
						maxlength="2048"
						type="password"
						autocomplete="new-password"
						:placeholder="formatMessage(messages.newPasswordPlaceholder)"
					/>
					<label for="confirm-new-password">
						<span class="label__title">{{ formatMessage(messages.confirmNewPasswordLabel) }}</span>
					</label>
					<input
						id="confirm-new-password"
						v-model="confirmNewPassword"
						maxlength="2048"
						type="password"
						autocomplete="new-password"
						:placeholder="formatMessage(messages.confirmNewPasswordPlaceholder)"
					/>
				</template>
				<p></p>
				<div class="input-group push-right">
					<button class="iconified-button" @click="$refs.managePasswordModal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
					<template v-if="removePasswordMode">
						<button
							type="button"
							class="iconified-button danger-button"
							:disabled="!oldPassword"
							@click="savePassword"
						>
							<TrashIcon />
							{{ formatMessage(messages.removePasswordButton) }}
						</button>
					</template>
					<template v-else>
						<button
							v-if="auth.user.has_password && auth.user.auth_providers.length > 0"
							type="button"
							class="iconified-button danger-button"
							@click="removePasswordMode = true"
						>
							<TrashIcon />
							{{ formatMessage(messages.removePasswordButton) }}
						</button>
						<button
							type="button"
							class="iconified-button brand-button"
							:disabled="
								newPassword.length == 0 ||
								(auth.user.has_password && oldPassword.length == 0) ||
								newPassword !== confirmNewPassword
							"
							@click="savePassword"
						>
							<SaveIcon />
							{{ formatMessage(messages.savePasswordButton) }}
						</button>
					</template>
				</div>
			</div>
		</Modal>
		<Modal
			ref="manageTwoFactorModal"
			:header="`${auth.user.has_totp && twoFactorStep === 0 ? formatMessage(messages.twoFactorRemoveButton) : formatMessage(messages.twoFactorSetupButton)}`"
		>
			<div class="universal-modal">
				<template v-if="auth.user.has_totp && twoFactorStep === 0">
					<label for="two-factor-code">
						<span class="label__title">{{ formatMessage(messages.twoFactorEnterCodeLabel) }}</span>
						<span class="label__description">{{
							formatMessage(messages.twoFactorEnterCodeDescription)
						}}</span>
					</label>
					<input
						id="two-factor-code"
						v-model="twoFactorCode"
						maxlength="11"
						type="text"
						:placeholder="formatMessage(messages.twoFactorCodePlaceholder)"
						@keyup.enter="removeTwoFactor()"
					/>
					<p v-if="twoFactorIncorrect" class="known-errors">
						{{ formatMessage(messages.twoFactorIncorrectError) }}
					</p>
					<div class="input-group push-right">
						<button class="iconified-button" @click="$refs.manageTwoFactorModal.hide()">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
						<button class="iconified-button danger-button" @click="removeTwoFactor">
							<TrashIcon />
							{{ formatMessage(messages.twoFactorRemoveButton) }}
						</button>
					</div>
				</template>
				<template v-else>
					<template v-if="twoFactorStep === 0">
						<p>{{ formatMessage(messages.twoFactorSetupIntro) }}</p>
						<p>
							<IntlFormatted :message-id="messages.twoFactorSetupScan">
								<template #authy-link="{ children }">
									<a href="https://authy.com/" target="_blank" rel="noreferrer">
										<component :is="() => children" />
									</a>
								</template>
								<template #microsoft-authenticator-link="{ children }">
									<a
										href="https://www.microsoft.com/en-us/security/mobile-authenticator-app"
										target="_blank"
										rel="noreferrer"
									>
										<component :is="() => children" />
									</a>
								</template>
							</IntlFormatted>
						</p>
						<qrcode-vue
							v-if="twoFactorSecret"
							:value="`otpauth://totp/${encodeURIComponent(
								auth.user.email,
							)}?secret=${twoFactorSecret}&issuer=Modrinth`"
							:size="250"
							:margin="2"
							level="H"
						/>
						<p>
							{{ formatMessage(messages.twoFactorManualSecretPrefix) }}
							<strong>{{ twoFactorSecret }}</strong>
						</p>
					</template>
					<template v-if="twoFactorStep === 1">
						<label for="verify-code">
							<span class="label__title">{{
								formatMessage(messages.twoFactorVerifyCodeLabel)
							}}</span>
							<span class="label__description">{{
								formatMessage(messages.twoFactorVerifyCodeDescription)
							}}</span>
						</label>
						<input
							id="verify-code"
							v-model="twoFactorCode"
							maxlength="6"
							type="text"
							autocomplete="one-time-code"
							:placeholder="formatMessage(messages.twoFactorCodePlaceholder)"
							@keyup.enter="verifyTwoFactorCode()"
						/>
						<p v-if="twoFactorIncorrect" class="known-errors">
							{{ formatMessage(messages.twoFactorIncorrectError) }}
						</p>
					</template>
					<template v-if="twoFactorStep === 2">
						<p>{{ formatMessage(messages.twoFactorBackupCodesIntro) }}</p>
						<p>{{ formatMessage(messages.twoFactorBackupCodesSingleUse) }}</p>
						<ul>
							<li v-for="code in backupCodes" :key="code">{{ code }}</li>
						</ul>
					</template>
					<div class="input-group push-right">
						<button v-if="twoFactorStep === 1" class="iconified-button" @click="twoFactorStep = 0">
							<LeftArrowIcon />
							{{ formatMessage(commonMessages.backButton) }}
						</button>
						<button
							v-if="twoFactorStep !== 2"
							class="iconified-button"
							@click="$refs.manageTwoFactorModal.hide()"
						>
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
						<button
							v-if="twoFactorStep <= 1"
							class="iconified-button brand-button"
							@click="twoFactorStep === 1 ? verifyTwoFactorCode() : (twoFactorStep = 1)"
						>
							<RightArrowIcon />
							{{ formatMessage(commonMessages.continueButton) }}
						</button>
						<button
							v-if="twoFactorStep === 2"
							class="iconified-button brand-button"
							@click="$refs.manageTwoFactorModal.hide()"
						>
							<CheckIcon />
							{{ formatMessage(messages.completeSetupButton) }}
						</button>
					</div>
				</template>
			</div>
		</Modal>
		<Modal ref="manageProvidersModal" :header="formatMessage(messages.manageProvidersModalHeader)">
			<div class="universal-modal">
				<div class="table">
					<div class="table-head table-row">
						<div class="table-text table-cell">
							{{ formatMessage(messages.providersTableProvider) }}
						</div>
						<div class="table-text table-cell">
							{{ formatMessage(messages.providersTableActions) }}
						</div>
					</div>
					<div v-for="provider in authProviders" :key="provider.id" class="table-row">
						<div class="table-text table-cell">
							<span><component :is="provider.icon" /> {{ provider.display }}</span>
						</div>
						<div class="table-text manage table-cell">
							<button
								v-if="auth.user.auth_providers.includes(provider.id)"
								class="btn"
								@click="handleRemoveAuthProvider(provider.id)"
							>
								<TrashIcon /> {{ formatMessage(commonMessages.removeButton) }}
							</button>
							<a
								v-else
								class="btn"
								:href="`${getAuthUrl(provider.id, '/settings/account')}&token=${auth.token}`"
							>
								<ExternalIcon /> {{ formatMessage(messages.providerAddButton) }}
							</a>
						</div>
					</div>
				</div>
				<p></p>
				<div class="input-group push-right">
					<button class="iconified-button" @click="$refs.manageProvidersModal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.closeButton) }}
					</button>
				</div>
			</div>
		</Modal>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(messages.accountSecurityTitle) }}</h2>

			<div class="adjacent-input">
				<label for="theme-selector">
					<span class="label__title">{{ formatMessage(messages.emailFieldTitle) }}</span>
					<span class="label__description">{{
						formatMessage(messages.emailFieldDescription)
					}}</span>
				</label>
				<div>
					<button class="iconified-button" @click="$refs.changeEmailModal.show()">
						<template v-if="auth.user.email">
							<EditIcon />
							{{ formatMessage(messages.changeEmailButton) }}
						</template>
						<template v-else>
							<PlusIcon />
							{{ formatMessage(messages.addEmailButton) }}
						</template>
					</button>
				</div>
			</div>
			<div class="adjacent-input">
				<label for="theme-selector">
					<span class="label__title">{{ formatMessage(messages.passwordFieldTitle) }}</span>
					<span v-if="auth.user.has_password" class="label__description">
						{{
							auth.user.auth_providers.length > 0
								? formatMessage(messages.passwordDescriptionChangeOrRemove)
								: formatMessage(messages.passwordDescriptionChange)
						}}
					</span>
					<span v-else class="label__description">
						{{ formatMessage(messages.passwordDescriptionSet) }}
					</span>
				</label>
				<div>
					<button
						class="iconified-button"
						@click="
							() => {
								oldPassword = ''
								newPassword = ''
								confirmNewPassword = ''
								removePasswordMode = false
								$refs.managePasswordModal.show()
							}
						"
					>
						<KeyIcon />
						<template v-if="auth.user.has_password">{{
							formatMessage(messages.changePasswordButton)
						}}</template>
						<template v-else> {{ formatMessage(messages.addPasswordButton) }} </template>
					</button>
				</div>
			</div>
			<div class="adjacent-input">
				<label for="theme-selector">
					<span class="label__title">{{ formatMessage(messages.twoFactorFieldTitle) }}</span>
					<span class="label__description">{{
						formatMessage(messages.twoFactorFieldDescription)
					}}</span>
				</label>
				<div>
					<button class="iconified-button" @click="showTwoFactorModal">
						<template v-if="auth.user.has_totp">
							<TrashIcon /> {{ formatMessage(messages.twoFactorRemoveButton) }}
						</template>
						<template v-else>
							<PlusIcon /> {{ formatMessage(messages.twoFactorSetupButton) }}
						</template>
					</button>
				</div>
			</div>
			<div class="adjacent-input">
				<label for="theme-selector">
					<span class="label__title">{{ formatMessage(messages.manageProvidersFieldTitle) }}</span>
					<span class="label__description">{{
						formatMessage(messages.manageProvidersFieldDescription)
					}}</span>
				</label>
				<div>
					<button class="iconified-button" @click="$refs.manageProvidersModal.show()">
						<SettingsIcon /> {{ formatMessage(messages.manageProvidersButton) }}
					</button>
				</div>
			</div>
		</section>

		<section id="data-export" class="universal-card">
			<h2>{{ formatMessage(messages.dataExportTitle) }}</h2>
			<p>{{ formatMessage(messages.dataExportDescription) }}</p>
			<a v-if="generated" class="iconified-button" :href="generated" download="export.json">
				<DownloadIcon />
				{{ formatMessage(messages.downloadExportButton) }}
			</a>
			<button v-else class="iconified-button" :disabled="generatingExport" @click="exportData">
				<template v-if="generatingExport">
					<UpdatedIcon /> {{ formatMessage(messages.generatingExportButton) }}
				</template>
				<template v-else>
					<UpdatedIcon /> {{ formatMessage(messages.generateExportButton) }}
				</template>
			</button>
		</section>

		<section id="delete-account" class="universal-card">
			<h2>{{ formatMessage(messages.deleteAccountSectionTitle) }}</h2>
			<p>{{ formatMessage(messages.deleteAccountSectionDescription) }}</p>
			<button
				type="button"
				class="iconified-button danger-button"
				@click="$refs.modal_confirm.show()"
			>
				<TrashIcon />
				{{ formatMessage(messages.deleteAccountButton) }}
			</button>
		</section>
	</div>
</template>

<script setup>
import {
	CheckIcon,
	DownloadIcon,
	EditIcon,
	ExternalIcon,
	LeftArrowIcon,
	PlusIcon,
	RightArrowIcon,
	SaveIcon,
	SettingsIcon,
	TrashIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	ConfirmModal,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	useVIntl,
} from '@modrinth/ui'
import KeyIcon from 'assets/icons/auth/key.svg'
import DiscordIcon from 'assets/icons/auth/sso-discord.svg'
import GithubIcon from 'assets/icons/auth/sso-github.svg'
import GitLabIcon from 'assets/icons/auth/sso-gitlab.svg'
import GoogleIcon from 'assets/icons/auth/sso-google.svg'
import MicrosoftIcon from 'assets/icons/auth/sso-microsoft.svg'
import SteamIcon from 'assets/icons/auth/sso-steam.svg'
import QrcodeVue from 'qrcode.vue'

import Modal from '~/components/ui/Modal.vue'
import { getAuthUrl, removeAuthProvider } from '~/composables/auth.js'

definePageMeta({
	middleware: 'auth',
})

const { addNotification } = injectNotificationManager()
const auth = await useAuth()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	deleteAccountConfirmTitle: {
		id: 'settings.account.delete.confirm.title',
		defaultMessage: 'Are you sure you want to delete your account?',
	},
	deleteAccountConfirmDescription: {
		id: 'settings.account.delete.confirm.description',
		defaultMessage:
			'This will **immediately delete all of your user data and follows**. This will not delete your projects. Deleting your account cannot be reversed.<br><br>If you need help with your account, get support on the [Modrinth Discord](https://discord.modrinth.com).',
	},
	deleteAccountConfirmProceed: {
		id: 'settings.account.delete.confirm.proceed',
		defaultMessage: 'Delete this account',
	},
	changeEmailHeaderChange: {
		id: 'settings.account.email.modal.header.change',
		defaultMessage: 'Change email',
	},
	changeEmailHeaderAdd: {
		id: 'settings.account.email.modal.header.add',
		defaultMessage: 'Add email',
	},
	emailNotPublicNotice: {
		id: 'settings.account.email.modal.notice',
		defaultMessage: 'Your account information is not displayed publicly.',
	},
	emailAddressLabel: {
		id: 'settings.account.email.field.label',
		defaultMessage: 'Email address',
	},
	emailAddressPlaceholder: {
		id: 'settings.account.email.field.placeholder',
		defaultMessage: 'Enter your email address...',
	},
	saveEmailButton: {
		id: 'settings.account.email.action.save',
		defaultMessage: 'Save email',
	},
	passwordHeaderRemove: {
		id: 'settings.account.password.modal.header.remove',
		defaultMessage: 'Remove password',
	},
	passwordHeaderChange: {
		id: 'settings.account.password.modal.header.change',
		defaultMessage: 'Change password',
	},
	passwordHeaderAdd: {
		id: 'settings.account.password.modal.header.add',
		defaultMessage: 'Add password',
	},
	passwordsDoNotMatchError: {
		id: 'settings.account.password.error.mismatch',
		defaultMessage: 'Input passwords do not match!',
	},
	confirmPasswordLabel: {
		id: 'settings.account.password.field.confirm-current.label',
		defaultMessage: 'Confirm password',
	},
	confirmPasswordDescription: {
		id: 'settings.account.password.field.confirm-current.description',
		defaultMessage: 'Please enter your password to proceed.',
	},
	oldPasswordLabel: {
		id: 'settings.account.password.field.old.label',
		defaultMessage: 'Old password',
	},
	confirmPasswordPlaceholder: {
		id: 'settings.account.password.field.confirm-current.placeholder',
		defaultMessage: 'Confirm password',
	},
	oldPasswordPlaceholder: {
		id: 'settings.account.password.field.old.placeholder',
		defaultMessage: 'Old password',
	},
	newPasswordLabel: {
		id: 'settings.account.password.field.new.label',
		defaultMessage: 'New password',
	},
	newPasswordPlaceholder: {
		id: 'settings.account.password.field.new.placeholder',
		defaultMessage: 'New password',
	},
	confirmNewPasswordLabel: {
		id: 'settings.account.password.field.confirm-new.label',
		defaultMessage: 'Confirm new password',
	},
	confirmNewPasswordPlaceholder: {
		id: 'settings.account.password.field.confirm-new.placeholder',
		defaultMessage: 'Confirm new password',
	},
	removePasswordButton: {
		id: 'settings.account.password.action.remove',
		defaultMessage: 'Remove password',
	},
	savePasswordButton: {
		id: 'settings.account.password.action.save',
		defaultMessage: 'Save password',
	},
	accountSecurityTitle: {
		id: 'settings.account.security.title',
		defaultMessage: 'Account security',
	},
	emailFieldTitle: {
		id: 'settings.account.security.email.title',
		defaultMessage: 'Email',
	},
	emailFieldDescription: {
		id: 'settings.account.security.email.description',
		defaultMessage: 'Changes the email associated with your account.',
	},
	changeEmailButton: {
		id: 'settings.account.security.email.action.change',
		defaultMessage: 'Change email',
	},
	addEmailButton: {
		id: 'settings.account.security.email.action.add',
		defaultMessage: 'Add email',
	},
	passwordFieldTitle: {
		id: 'settings.account.security.password.title',
		defaultMessage: 'Password',
	},
	passwordDescriptionChange: {
		id: 'settings.account.security.password.description.change',
		defaultMessage: 'Change the password used to login to your account.',
	},
	passwordDescriptionChangeOrRemove: {
		id: 'settings.account.security.password.description.change-or-remove',
		defaultMessage: 'Change or remove the password used to login to your account.',
	},
	passwordDescriptionSet: {
		id: 'settings.account.security.password.description.set',
		defaultMessage: 'Set a permanent password to login to your account.',
	},
	changePasswordButton: {
		id: 'settings.account.security.password.action.change',
		defaultMessage: 'Change password',
	},
	addPasswordButton: {
		id: 'settings.account.security.password.action.add',
		defaultMessage: 'Add password',
	},
	twoFactorFieldTitle: {
		id: 'settings.account.security.two-factor.title',
		defaultMessage: 'Two-factor authentication',
	},
	twoFactorFieldDescription: {
		id: 'settings.account.security.two-factor.description',
		defaultMessage: 'Add an additional layer of security to your account during login.',
	},
	twoFactorSetupButton: {
		id: 'settings.account.security.two-factor.action.setup',
		defaultMessage: 'Setup 2FA',
	},
	twoFactorEnterCodeLabel: {
		id: 'settings.account.two-factor.field.code.label',
		defaultMessage: 'Enter two-factor code',
	},
	twoFactorEnterCodeDescription: {
		id: 'settings.account.two-factor.field.code.description',
		defaultMessage: 'Please enter a two-factor code to proceed.',
	},
	twoFactorCodePlaceholder: {
		id: 'settings.account.two-factor.field.code.placeholder',
		defaultMessage: 'Enter code...',
	},
	twoFactorIncorrectError: {
		id: 'settings.account.two-factor.error.incorrect-code',
		defaultMessage: 'The code entered is incorrect!',
	},
	twoFactorRemoveButton: {
		id: 'settings.account.security.two-factor.action.remove',
		defaultMessage: 'Remove 2FA',
	},
	twoFactorSetupIntro: {
		id: 'settings.account.two-factor.setup.intro',
		defaultMessage:
			'Two-factor authentication keeps your account secure by requiring access to a second device in order to sign in.',
	},
	twoFactorSetupScan: {
		id: 'settings.account.two-factor.setup.scan',
		defaultMessage:
			'Scan the QR code with <authy-link>Authy</authy-link>, <microsoft-authenticator-link>Microsoft Authenticator</microsoft-authenticator-link>, or any other 2FA app to begin.',
	},
	twoFactorManualSecretPrefix: {
		id: 'settings.account.two-factor.setup.manual-secret',
		defaultMessage: 'If the QR code does not scan, you can manually enter the secret:',
	},
	twoFactorVerifyCodeLabel: {
		id: 'settings.account.two-factor.verify.label',
		defaultMessage: 'Verify code',
	},
	twoFactorVerifyCodeDescription: {
		id: 'settings.account.two-factor.verify.description',
		defaultMessage: 'Enter the one-time code from authenticator to verify access.',
	},
	twoFactorBackupCodesIntro: {
		id: 'settings.account.two-factor.backup.intro',
		defaultMessage:
			'Download and save these back-up codes in a safe place. You can use these in-place of a 2FA code if you ever lose access to your device! You should protect these codes like your password.',
	},
	twoFactorBackupCodesSingleUse: {
		id: 'settings.account.two-factor.backup.single-use',
		defaultMessage: 'Backup codes can only be used once.',
	},
	completeSetupButton: {
		id: 'settings.account.button.complete-setup',
		defaultMessage: 'Complete setup',
	},
	manageProvidersModalHeader: {
		id: 'settings.account.providers.modal.header',
		defaultMessage: 'Authentication providers',
	},
	providersTableProvider: {
		id: 'settings.account.providers.table.provider',
		defaultMessage: 'Provider',
	},
	providersTableActions: {
		id: 'settings.account.providers.table.actions',
		defaultMessage: 'Actions',
	},
	providerAddButton: {
		id: 'settings.account.providers.action.add',
		defaultMessage: 'Add',
	},
	manageProvidersFieldTitle: {
		id: 'settings.account.security.providers.title',
		defaultMessage: 'Manage authentication providers',
	},
	manageProvidersFieldDescription: {
		id: 'settings.account.security.providers.description',
		defaultMessage:
			'Add or remove sign-on methods from your account, including GitHub, GitLab, Microsoft, Discord, Steam, and Google.',
	},
	manageProvidersButton: {
		id: 'settings.account.security.providers.action.manage',
		defaultMessage: 'Manage providers',
	},
	dataExportTitle: {
		id: 'settings.account.data-export.title',
		defaultMessage: 'Data export',
	},
	dataExportDescription: {
		id: 'settings.account.data-export.description',
		defaultMessage:
			'Request a copy of all your personal data you have uploaded to Modrinth. This may take several minutes to complete.',
	},
	downloadExportButton: {
		id: 'settings.account.data-export.action.download',
		defaultMessage: 'Download export',
	},
	generatingExportButton: {
		id: 'settings.account.data-export.action.generating',
		defaultMessage: 'Generating export...',
	},
	generateExportButton: {
		id: 'settings.account.data-export.action.generate',
		defaultMessage: 'Generate export',
	},
	deleteAccountSectionTitle: {
		id: 'settings.account.delete.section.title',
		defaultMessage: 'Delete account',
	},
	deleteAccountSectionDescription: {
		id: 'settings.account.delete.section.description',
		defaultMessage:
			'Once you delete your account, there is no going back. Deleting your account will remove all attached data, excluding projects, from our servers.',
	},
	deleteAccountButton: {
		id: 'settings.account.delete.section.action',
		defaultMessage: 'Delete account',
	},
})

const changeEmailModal = ref()
const email = ref(auth.value.user.email)
async function saveEmail() {
	if (!email.value) {
		return
	}

	startLoading()
	try {
		await useBaseFetch(`auth/email`, {
			method: 'PATCH',
			body: {
				email: email.value,
			},
		})
		changeEmailModal.value.hide()
		await useAuth(auth.value.token)
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

async function handleRemoveAuthProvider(provider) {
	try {
		await removeAuthProvider(provider)
	} catch (error) {
		handleError(error)
	}
}

const managePasswordModal = ref()
const removePasswordMode = ref(false)
const oldPassword = ref('')
const newPassword = ref('')
const confirmNewPassword = ref('')
async function savePassword() {
	if (newPassword.value !== confirmNewPassword.value) {
		return
	}

	startLoading()
	try {
		await useBaseFetch(`auth/password`, {
			method: 'PATCH',
			body: {
				old_password: auth.value.user.has_password ? oldPassword.value : null,
				new_password: removePasswordMode.value ? null : newPassword.value,
			},
		})
		managePasswordModal.value.hide()
		await useAuth(auth.value.token)
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

const manageTwoFactorModal = ref()
const twoFactorSecret = ref(null)
const twoFactorFlow = ref(null)
const twoFactorStep = ref(0)
async function showTwoFactorModal() {
	twoFactorStep.value = 0
	twoFactorCode.value = null
	twoFactorIncorrect.value = false
	if (auth.value.user.has_totp) {
		manageTwoFactorModal.value.show()
		return
	}

	twoFactorSecret.value = null
	twoFactorFlow.value = null
	backupCodes.value = []
	manageTwoFactorModal.value.show()

	startLoading()
	try {
		const res = await useBaseFetch('auth/2fa/get_secret', {
			method: 'POST',
		})

		twoFactorSecret.value = res.secret
		twoFactorFlow.value = res.flow
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

const twoFactorIncorrect = ref(false)
const twoFactorCode = ref(null)
const backupCodes = ref([])
async function verifyTwoFactorCode() {
	startLoading()
	try {
		const res = await useBaseFetch('auth/2fa', {
			method: 'POST',
			body: {
				code: twoFactorCode.value ? twoFactorCode.value : '',
				flow: twoFactorFlow.value,
			},
		})

		backupCodes.value = res.backup_codes
		twoFactorStep.value = 2
		await useAuth(auth.value.token)
	} catch {
		twoFactorIncorrect.value = true
	}
	stopLoading()
}

async function removeTwoFactor() {
	startLoading()
	try {
		await useBaseFetch('auth/2fa', {
			method: 'DELETE',
			body: {
				code: twoFactorCode.value ? twoFactorCode.value.toString() : '',
			},
		})
		manageTwoFactorModal.value.hide()
		await useAuth(auth.value.token)
	} catch {
		twoFactorIncorrect.value = true
	}
	stopLoading()
}

const authProviders = [
	{
		id: 'github',
		display: 'GitHub',
		icon: GithubIcon,
	},
	{
		id: 'gitlab',
		display: 'GitLab',
		icon: GitLabIcon,
	},
	{
		id: 'steam',
		display: 'Steam',
		icon: SteamIcon,
	},
	{
		id: 'discord',
		display: 'Discord',
		icon: DiscordIcon,
	},
	{
		id: 'microsoft',
		display: 'Microsoft',
		icon: MicrosoftIcon,
	},
	{
		id: 'google',
		display: 'Google',
		icon: GoogleIcon,
	},
]

async function deleteAccount() {
	startLoading()
	try {
		await useBaseFetch(`user/${auth.value.user.id}`, {
			method: 'DELETE',
		})
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	useCookie('auth-token').value = null
	window.location.href = '/'

	stopLoading()
}

const generatingExport = ref(false)
const generated = ref()
async function exportData() {
	startLoading()
	generatingExport.value = true
	try {
		const res = await useBaseFetch('gdpr/export', {
			method: 'POST',
			internal: true,
		})

		const jsonString = JSON.stringify(res, null, 2)

		const blob = new Blob([jsonString], { type: 'application/json' })
		generated.value = URL.createObjectURL(blob)
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	generatingExport.value = false
	stopLoading()
}
</script>
<style lang="scss" scoped>
canvas {
	margin: 0 auto;
	border-radius: var(--size-rounded-card);
}

.table-row {
	grid-template-columns: 1fr 10rem;

	span {
		display: flex;
		align-items: center;
		margin: auto 0;

		svg {
			width: 1.25rem;
			height: 1.25rem;
			margin-right: 0.35rem;
		}
	}
}
</style>
