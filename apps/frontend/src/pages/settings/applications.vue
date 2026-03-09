<template>
	<div class="universal-card">
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(messages.deleteConfirmTitle)"
			:description="formatMessage(messages.deleteConfirmDescription)"
			:proceed-label="formatMessage(messages.deleteConfirmButton)"
			@proceed="removeApp(editingId)"
		/>
		<Modal ref="appModal" :header="formatMessage(messages.modalHeader)">
			<div class="universal-modal">
				<label for="app-name"
					><span class="label__title">{{ formatMessage(messages.nameLabel) }}</span>
				</label>
				<StyledInput
					id="app-name"
					v-model="name"
					:maxlength="2048"
					autocomplete="off"
					:placeholder="formatMessage(messages.namePlaceholder)"
				/>
				<label v-if="editingId" for="app-icon"
					><span class="label__title">{{ formatMessage(messages.iconLabel) }}</span>
				</label>
				<div v-if="editingId" class="icon-submission">
					<Avatar size="md" :src="icon" />
					<FileInput
						:max-size="262144"
						class="btn"
						:prompt="formatMessage(messages.uploadIcon)"
						accept="image/png,image/jpeg,image/gif,image/webp"
						@change="onImageSelection"
					>
						<UploadIcon />
					</FileInput>
				</div>
				<label v-if="editingId" for="app-url">
					<span class="label__title">{{ formatMessage(messages.urlLabel) }}</span>
				</label>
				<StyledInput
					v-if="editingId"
					id="app-url"
					v-model="url"
					:maxlength="255"
					type="url"
					autocomplete="off"
					:placeholder="formatMessage(messages.urlPlaceholder)"
				/>
				<label v-if="editingId" for="app-description">
					<span class="label__title">{{ formatMessage(messages.descriptionLabel) }}</span>
				</label>
				<StyledInput
					v-if="editingId"
					id="app-description"
					v-model="description"
					multiline
					:maxlength="255"
					autocomplete="off"
					:placeholder="formatMessage(messages.descriptionPlaceholder)"
					input-class="h-24 resize-y"
				/>
				<label for="app-scopes"
					><span class="label__title">{{ formatMessage(messages.scopesLabel) }}</span>
				</label>
				<div
					id="app-scopes"
					class="scope-items mt-2 grid grid-cols-1 gap-x-6 gap-y-4 min-[600px]:grid-cols-2"
				>
					<div v-for="category in scopeCategories" :key="category.name" class="flex flex-col gap-2">
						<h4 class="m-0 border-b border-divider pb-1 text-base font-bold text-contrast">
							{{ category.name }}
						</h4>
						<div class="flex flex-col gap-2">
							<Checkbox
								v-for="scope in category.scopes"
								:key="scope"
								:label="scopesToLabels(getScopeValue(scope)).join(', ')"
								:model-value="hasScope(scopesVal, scope)"
								@update:model-value="() => (scopesVal = toggleScope(scopesVal, scope))"
							/>
						</div>
					</div>
				</div>
				<label for="app-redirect-uris" class="mt-4"
					><span class="label__title">{{ formatMessage(messages.redirectUrisLabel) }}</span>
				</label>
				<div class="uri-input-list">
					<div v-for="(_, index) in redirectUris" :key="index">
						<div class="input-group url-input-group-fixes">
							<StyledInput
								v-model="redirectUris[index]"
								:maxlength="2048"
								type="url"
								autocomplete="off"
								:placeholder="formatMessage(messages.redirectUriPlaceholder)"
							/>
							<Button v-if="index !== 0" icon-only @click="() => redirectUris.splice(index, 1)">
								<TrashIcon />
							</Button>
							<Button
								v-if="index === 0"
								color="primary"
								icon-only
								@click="() => redirectUris.push('')"
							>
								<PlusIcon /> {{ formatMessage(messages.addMore) }}
							</Button>
						</div>
					</div>
					<div v-if="redirectUris.length <= 0">
						<Button color="primary" icon-only @click="() => redirectUris.push('')">
							<PlusIcon /> {{ formatMessage(messages.addRedirectUri) }}
						</Button>
					</div>
				</div>

				<div class="submit-row input-group push-right">
					<button class="iconified-button" @click="$refs.appModal.hide()">
						<XIcon />
						{{ formatMessage(messages.cancel) }}
					</button>
					<button
						v-if="editingId"
						:disabled="!canSubmit"
						type="button"
						class="iconified-button brand-button"
						@click="editApp"
					>
						<SaveIcon />
						{{ formatMessage(messages.saveChanges) }}
					</button>
					<button
						v-else
						:disabled="!canSubmit"
						type="button"
						class="iconified-button brand-button"
						@click="createApp"
					>
						<PlusIcon />
						{{ formatMessage(messages.createApp) }}
					</button>
				</div>
			</div>
		</Modal>

		<div class="header__row">
			<div class="header__title">
				<h2 class="text-2xl">{{ formatMessage(commonSettingsMessages.applications) }}</h2>
			</div>
			<button
				class="btn btn-primary"
				@click="
					() => {
						name = null
						icon = null
						scopesVal = 0
						redirectUris = ['']
						editingId = null
						expires = null
						$refs.appModal.show()
					}
				"
			>
				<PlusIcon /> {{ formatMessage(messages.newApplication) }}
			</button>
		</div>
		<p>
			<IntlFormatted :message-id="messages.descriptionIntro">
				<template #docs-link="{ children }">
					<a class="text-link" href="https://docs.modrinth.com">
						<component :is="() => normalizeChildren(children)" />
					</a>
				</template>
			</IntlFormatted>
		</p>
		<div v-for="app in usersApps" :key="app.id" class="universal-card recessed token mt-4">
			<div class="token-info">
				<div class="token-icon">
					<Avatar size="sm" :src="app.icon_url" />
					<div>
						<h2 class="token-title">{{ app.name }}</h2>
						<div>
							{{
								formatMessage(messages.createdOn, {
									date: new Date(app.created).toLocaleDateString(),
								})
							}}
						</div>
					</div>
				</div>
				<div>
					<label for="token-information">
						<span class="label__title">{{ formatMessage(messages.aboutLabel) }}</span>
					</label>
					<div class="token-content">
						<div>
							{{ formatMessage(messages.clientId) }}
							<CopyCode :text="app.id" />
						</div>
						<div v-if="!!clientCreatedInState(app.id)">
							<div>
								{{ formatMessage(messages.clientSecret) }}
								<CopyCode :text="clientCreatedInState(app.id)?.client_secret" />
							</div>
							<div class="secret_disclaimer">
								<i>{{ formatMessage(messages.secretDisclaimer) }}</i>
							</div>
						</div>
					</div>
				</div>
			</div>
			<div class="input-group">
				<Button
					icon-only
					@click="
						() => {
							setForm({
								...app,
								redirect_uris: app.redirect_uris.map((u) => u.uri) || [],
							})
							$refs.appModal.show()
						}
					"
				>
					<EditIcon />
					{{ formatMessage(messages.edit) }}
				</Button>
				<Button
					color="danger"
					icon-only
					@click="
						() => {
							editingId = app.id
							$refs.modal_confirm.show()
						}
					"
				>
					<TrashIcon />
					{{ formatMessage(messages.delete) }}
				</Button>
			</div>
		</div>
	</div>
</template>
<script setup>
import { EditIcon, PlusIcon, SaveIcon, TrashIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	Checkbox,
	commonSettingsMessages,
	ConfirmModal,
	CopyCode,
	defineMessages,
	FileInput,
	injectNotificationManager,
	IntlFormatted,
	normalizeChildren,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

import Modal from '~/components/ui/Modal.vue'
import {
	getScopeValue,
	hasScope,
	scopeCategoryMessages,
	scopeList,
	toggleScope,
	useScopes,
} from '~/composables/auth/scopes.ts'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Applications - Modrinth',
})

const messages = defineMessages({
	modalHeader: {
		id: 'settings.applications.modal.header',
		defaultMessage: 'Application information',
	},
	deleteConfirmTitle: {
		id: 'settings.applications.delete.confirm.title',
		defaultMessage: 'Are you sure you want to delete this application?',
	},
	deleteConfirmDescription: {
		id: 'settings.applications.delete.confirm.description',
		defaultMessage:
			'This will permanently delete this application and revoke all access tokens. (forever!)',
	},
	deleteConfirmButton: {
		id: 'settings.applications.delete.confirm.button',
		defaultMessage: 'Delete this application',
	},
	nameLabel: {
		id: 'settings.applications.field.name',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'settings.applications.field.name.placeholder',
		defaultMessage: "Enter the application's name...",
	},
	iconLabel: {
		id: 'settings.applications.field.icon',
		defaultMessage: 'Icon',
	},
	uploadIcon: {
		id: 'settings.applications.button.upload-icon',
		defaultMessage: 'Upload icon',
	},
	urlLabel: {
		id: 'settings.applications.field.url',
		defaultMessage: 'URL',
	},
	urlPlaceholder: {
		id: 'settings.applications.field.url.placeholder',
		defaultMessage: 'https://example.com',
	},
	descriptionLabel: {
		id: 'settings.applications.field.description',
		defaultMessage: 'Description',
	},
	descriptionPlaceholder: {
		id: 'settings.applications.field.description.placeholder',
		defaultMessage: "Enter the application's description...",
	},
	scopesLabel: {
		id: 'settings.applications.field.scopes',
		defaultMessage: 'Scopes',
	},
	redirectUrisLabel: {
		id: 'settings.applications.field.redirect-uris',
		defaultMessage: 'Redirect URIs',
	},
	redirectUriPlaceholder: {
		id: 'settings.applications.field.redirect-uri.placeholder',
		defaultMessage: 'https://example.com/auth/callback',
	},
	addMore: {
		id: 'settings.applications.button.add-more',
		defaultMessage: 'Add more',
	},
	addRedirectUri: {
		id: 'settings.applications.button.add-redirect-uri',
		defaultMessage: 'Add a redirect URI',
	},
	cancel: {
		id: 'settings.applications.button.cancel',
		defaultMessage: 'Cancel',
	},
	saveChanges: {
		id: 'settings.applications.button.save-changes',
		defaultMessage: 'Save changes',
	},
	createApp: {
		id: 'settings.applications.button.create',
		defaultMessage: 'Create app',
	},
	newApplication: {
		id: 'settings.applications.button.new',
		defaultMessage: 'New application',
	},
	descriptionIntro: {
		id: 'settings.applications.description.intro',
		defaultMessage:
			"Applications can be used to authenticate Modrinth's users with your products. For more information, see <docs-link>Modrinth's API documentation</docs-link>.",
	},
	aboutLabel: {
		id: 'settings.applications.about',
		defaultMessage: 'About',
	},
	clientId: {
		id: 'settings.applications.client-id',
		defaultMessage: 'Client ID',
	},
	clientSecret: {
		id: 'settings.applications.client-secret',
		defaultMessage: 'Client secret',
	},
	secretDisclaimer: {
		id: 'settings.applications.secret.disclaimer',
		defaultMessage: 'Save your secret now, it will be hidden after you leave this page!',
	},
	createdOn: {
		id: 'settings.applications.created-on',
		defaultMessage: 'Created on {date}',
	},
	edit: {
		id: 'settings.applications.button.edit',
		defaultMessage: 'Edit',
	},
	delete: {
		id: 'settings.applications.button.delete',
		defaultMessage: 'Delete',
	},
	iconUpdatedTitle: {
		id: 'settings.applications.notification.icon-updated.title',
		defaultMessage: 'Icon updated',
	},
	iconUpdatedDescription: {
		id: 'settings.applications.notification.icon-updated.description',
		defaultMessage: 'Your application icon has been updated.',
	},
	errorTitle: {
		id: 'settings.applications.notification.error.title',
		defaultMessage: 'An error occurred',
	},
})

const { scopesToLabels } = useScopes()

const scopeCategories = computed(() => {
	return [
		{
			name: formatMessage(scopeCategoryMessages.categoryUserAccount),
			scopes: scopeList.filter((s) => s.startsWith('USER_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryProjects),
			scopes: scopeList.filter((s) => s.startsWith('PROJECT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryVersions),
			scopes: scopeList.filter((s) => s.startsWith('VERSION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryCollections),
			scopes: scopeList.filter((s) => s.startsWith('COLLECTION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryOrganizations),
			scopes: scopeList.filter((s) => s.startsWith('ORGANIZATION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryReports),
			scopes: scopeList.filter((s) => s.startsWith('REPORT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryThreads),
			scopes: scopeList.filter((s) => s.startsWith('THREAD_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryPats),
			scopes: scopeList.filter((s) => s.startsWith('PAT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categorySessions),
			scopes: scopeList.filter((s) => s.startsWith('SESSION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryNotifications),
			scopes: scopeList.filter((s) => s.startsWith('NOTIFICATION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryPayouts),
			scopes: scopeList.filter((s) => s.startsWith('PAYOUTS_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryAnalytics),
			scopes: scopeList.filter(
				(s) => s.startsWith('ANALYTICS') || s.startsWith('PERFORM_ANALYTICS'),
			),
		},
	].filter((c) => c.scopes.length > 0)
})

const appModal = ref()

// Any apps created in the current state will be stored here
// Users can copy Client Secrets and such before the page reloads
const createdApps = ref([])

const editingId = ref(null)
const name = ref(null)
const icon = ref(null)
const scopesVal = ref(BigInt(0))
const redirectUris = ref([''])
const url = ref(null)
const description = ref(null)

const loading = ref(false)

const auth = await useAuth()

const { data: usersApps, refresh } = await useAsyncData(
	'usersApps',
	() =>
		useBaseFetch(`user/${auth.value.user.id}/oauth_apps`, {
			apiVersion: 3,
		}),
	{
		watch: [auth],
	},
)

const setForm = (app) => {
	if (app?.id) {
		editingId.value = app.id
	} else {
		editingId.value = null
	}
	name.value = app?.name || ''
	icon.value = app?.icon_url || ''
	scopesVal.value = app?.max_scopes || BigInt(0)
	url.value = app?.url || ''
	description.value = app?.description || ''

	if (app?.redirect_uris) {
		redirectUris.value = app.redirect_uris.map((uri) => uri?.uri || uri)
	} else {
		redirectUris.value = ['']
	}
}

const canSubmit = computed(() => {
	// Make sure name, scopes, and return uri are at least filled in
	const filledIn =
		name.value && name.value !== '' && name.value?.length > 2 && redirectUris.value.length > 0
	// Make sure the redirect uris are either one empty string or all filled in with valid urls
	const oneValid = redirectUris.value.length === 1 && redirectUris.value[0] === ''
	let allValid
	try {
		allValid = redirectUris.value.every((uri) => {
			const url = new URL(uri)
			return !!url
		})
	} catch {
		allValid = false
	}
	return filledIn && (oneValid || allValid)
})

const clientCreatedInState = (id) => {
	return createdApps.value.find((app) => app.id === id)
}

async function onImageSelection(files) {
	if (!editingId.value) {
		throw new Error('No editing id')
	}

	if (files.length > 0) {
		const file = files[0]
		const extFromType = file.type.split('/')[1]

		await useBaseFetch('oauth/app/' + editingId.value + '/icon', {
			method: 'PATCH',
			internal: true,
			body: file,
			query: {
				ext: extFromType,
			},
		})

		await refresh()

		const app = usersApps.value.find((app) => app.id === editingId.value)
		if (app) {
			setForm(app)
		}

		addNotification({
			title: formatMessage(messages.iconUpdatedTitle),
			text: formatMessage(messages.iconUpdatedDescription),
			type: 'success',
		})
	}
}

async function createApp() {
	startLoading()
	loading.value = true
	try {
		const createdAppInfo = await useBaseFetch('oauth/app', {
			method: 'POST',
			internal: true,
			body: {
				name: name.value,
				icon_url: icon.value,
				max_scopes: Number(scopesVal.value), // JS is 52 bit for ints so we're good for now
				redirect_uris: redirectUris.value,
			},
		})

		createdApps.value.push(createdAppInfo)

		setForm(null)
		appModal.value.hide()

		await refresh()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	loading.value = false
	stopLoading()
}

async function editApp() {
	startLoading()
	loading.value = true
	try {
		if (!editingId.value) {
			throw new Error('No editing id')
		}

		// check if there's any difference between the current app and the one in the state
		const app = usersApps.value.find((app) => app.id === editingId.value)
		if (!app) {
			throw new Error('No app found')
		}

		if (
			app.name === name.value &&
			app.icon_url === icon.value &&
			app.max_scopes === scopesVal.value &&
			app.redirect_uris === redirectUris.value &&
			app.url === url.value &&
			app.description === description.value
		) {
			setForm(null)
			editingId.value = null
			appModal.value.hide()
			throw new Error('No changes detected')
		}

		const body = {
			name: name.value,
			max_scopes: Number(scopesVal.value), // JS is 52 bit for ints so we're good for now
			redirect_uris: redirectUris.value,
		}

		if (url.value && url.value?.length > 0) {
			body.url = url.value
		}

		if (description.value && description.value?.length > 0) {
			body.description = description.value
		}

		if (icon.value && icon.value?.length > 0) {
			body.icon_url = icon.value
		}

		await useBaseFetch('oauth/app/' + editingId.value, {
			method: 'PATCH',
			internal: true,
			body,
		})

		await refresh()
		setForm(null)
		editingId.value = null

		appModal.value.hide()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	loading.value = false
	stopLoading()
}

async function removeApp() {
	startLoading()
	try {
		if (!editingId.value) {
			throw new Error('No editing id')
		}
		await useBaseFetch(`oauth/app/${editingId.value}`, {
			internal: true,
			method: 'DELETE',
		})
		await refresh()
		editingId.value = null
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
<style lang="scss" scoped>
.secret_disclaimer {
	font-size: var(--font-size-sm);
}
.submit-row {
	padding-top: var(--gap-lg);
}
.uri-input-list {
	display: grid;
	row-gap: 0.5rem;
}
.url-input-group-fixes {
	width: 100%;

	input {
		width: 100% !important;
		flex-basis: 24rem !important;
	}
}

.scope-items :deep(.checkbox-outer) {
	white-space: nowrap !important;
	justify-content: flex-start !important;
}

.icon-submission {
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: var(--gap-md);
}

.token {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: var(--gap-sm);

	.token-info {
		display: flex;
		flex-direction: column;
		gap: var(--gap-sm);
	}

	.token-content {
		display: grid;
		gap: var(--gap-xs);
	}

	.token-icon {
		display: flex;
		align-items: flex-start;
		gap: var(--gap-lg);
		padding-bottom: var(--gap-sm);
	}

	.token-heading {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--color-gray-700);

		margin-top: var(--spacing-card-md);
		margin-bottom: var(--spacing-card-sm);
	}

	.token-title {
		margin-bottom: var(--spacing-card-xs);
	}

	.input-group {
		margin-left: auto;

		// For the children override the padding so that y padding is --gap-sm and x padding is --gap-lg
		// Knossos global styling breaks everything
		> * {
			padding: var(--gap-sm) var(--gap-lg);
		}
	}

	@media screen and (min-width: 800px) {
		flex-direction: row;
	}
}
</style>
