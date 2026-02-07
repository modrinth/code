<template>
	<div>
		<section class="card">
			<h2 class="text-2xl">{{ formatMessage(messages.title) }}</h2>
			<p class="mb-4">
				<IntlFormatted :message-id="messages.description">
					<template #docs-link="{ children }">
						<a href="https://docs.modrinth.com/" target="_blank" class="text-link">
							<component :is="() => children" />
						</a>
					</template>
				</IntlFormatted>
			</p>
			<label>
				<span class="label__title">{{ formatMessage(messages.profilePicture) }}</span>
			</label>
			<div class="avatar-changer">
				<Avatar
					:src="previewImage ? previewImage : avatarUrl"
					size="md"
					circle
					:alt="auth.user.username"
				/>
				<div class="input-stack">
					<FileInput
						:max-size="262144"
						:show-icon="true"
						class="btn"
						:prompt="formatMessage(commonMessages.uploadImageButton)"
						accept="image/png,image/jpeg,image/gif,image/webp"
						@change="showPreviewImage"
					>
						<UploadIcon />
					</FileInput>
					<Button v-if="avatarUrl !== null" :action="removePreviewImage">
						<TrashIcon />
						{{ formatMessage(commonMessages.removeImageButton) }}
					</Button>
					<Button
						v-if="previewImage"
						:action="
							() => {
								icon = null
								previewImage = null
							}
						"
					>
						<UndoIcon />
						{{ formatMessage(commonMessages.resetButton) }}
					</Button>
				</div>
			</div>
			<label for="username-field">
				<span class="label__title">{{ formatMessage(commonMessages.usernameLabel) }}</span>
				<span class="label__description">
					{{ formatMessage(messages.usernameDescription) }}
				</span>
			</label>
			<input id="username-field" v-model="current.username" type="text" />
			<label for="bio-field">
				<span class="label__title">{{ formatMessage(messages.bioTitle) }}</span>
				<span class="label__description">
					{{ formatMessage(messages.bioDescription) }}
				</span>
			</label>
			<textarea id="bio-field" v-model="current.bio" type="text" />
			<div class="input-group">
				<Button :link="`/user/${auth.user.username}`">
					<UserIcon /> {{ formatMessage(commonMessages.visitYourProfile) }}
				</Button>
			</div>
		</section>
		<UnsavedChangesPopup
			:original="originalState"
			:modified="modifiedState"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup>
import { TrashIcon, UndoIcon, UploadIcon, UserIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	FileInput,
	injectNotificationManager,
	IntlFormatted,
	UnsavedChangesPopup,
	useSavable,
	useVIntl,
} from '@modrinth/ui'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

definePageMeta({
	middleware: 'auth',
})

const messages = defineMessages({
	headTitle: {
		id: 'settings.profile.head-title',
		defaultMessage: 'Profile settings',
	},
	title: {
		id: 'settings.profile.profile-info',
		defaultMessage: 'Profile information',
	},
	description: {
		id: 'settings.profile.description',
		defaultMessage:
			'Your profile information is publicly viewable on Modrinth and through the <docs-link>Modrinth API</docs-link>.',
	},
	profilePicture: {
		id: 'settings.profile.profile-picture.title',
		defaultMessage: 'Profile picture',
	},
	usernameDescription: {
		id: 'settings.profile.username.description',
		defaultMessage: 'A unique case-insensitive name to identify your profile.',
	},
	bioTitle: {
		id: 'settings.profile.bio.title',
		defaultMessage: 'Bio',
	},
	bioDescription: {
		id: 'settings.profile.bio.description',
		defaultMessage: 'A short description to tell everyone a little bit about you.',
	},
})

const auth = await useAuth()

// Avatar state (separate from useSavable)
const avatarUrl = ref(auth.value.user.avatar_url)
const icon = shallowRef(null)
const previewImage = shallowRef(null)
const pendingAvatarDeletion = ref(false)
const saving = ref(false)

const {
	saved,
	current,
	reset: resetFields,
} = useSavable(
	() => ({
		username: auth.value.user.username,
		bio: auth.value.user.bio ?? '',
	}),
	async () => {}, // Save is handled manually due to complex icon logic
)

// Combined state for UnsavedChangesPopup
const originalState = computed(() => ({
	...saved.value,
	avatarChanged: false,
}))

const modifiedState = computed(() => ({
	...current.value,
	avatarChanged: !!(previewImage.value || pendingAvatarDeletion.value),
}))

const reset = () => {
	resetFields()
	icon.value = null
	previewImage.value = null
	pendingAvatarDeletion.value = false
}

function showPreviewImage(files) {
	const reader = new FileReader()
	icon.value = files[0]
	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target.result
	}
}

function removePreviewImage() {
	pendingAvatarDeletion.value = true
	previewImage.value = 'https://cdn.modrinth.com/placeholder.png'
}

async function save() {
	saving.value = true
	try {
		if (pendingAvatarDeletion.value) {
			await useBaseFetch(`user/${auth.value.user.id}/icon`, {
				method: 'DELETE',
			})
			pendingAvatarDeletion.value = false
			previewImage.value = null
		}

		if (icon.value) {
			await useBaseFetch(
				`user/${auth.value.user.id}/icon?ext=${
					icon.value.type.split('/')[icon.value.type.split('/').length - 1]
				}`,
				{
					method: 'PATCH',
					body: icon.value,
				},
			)
			icon.value = null
			previewImage.value = null
		}

		const body = {}

		if (auth.value.user.username !== current.value.username) {
			body.username = current.value.username
		}

		if (auth.value.user.bio !== current.value.bio) {
			body.bio = current.value.bio
		}

		await useBaseFetch(`user/${auth.value.user.id}`, {
			method: 'PATCH',
			body,
		})
		await useAuth(auth.value.token)
		avatarUrl.value = auth.value.user.avatar_url
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err
				? err.data
					? err.data.description
						? err.data.description
						: err.data
					: err
				: 'aaaaahhh',
			type: 'error',
		})
	}
	saving.value = false
}
</script>
<style lang="scss" scoped>
.avatar-changer {
	display: flex;
	gap: var(--gap-lg);
	margin-top: var(--gap-md);
}

textarea {
	height: 6rem;
	width: 40rem;
	margin-bottom: var(--gap-lg);
}
</style>
