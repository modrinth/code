<template>
	<EmptyState
		v-if="!auth.user.value"
		type="empty"
		class="[&>div:last-child]:!mt-6"
		:heading="formatMessage(messages.signInRequiredTitle)"
		:description="formatMessage(messages.signInRequiredDescription)"
	>
		<template #illustration>
			<div class="relative mb-4 h-[200px]">
				<img :src="ThinkingRinthbot" alt="" class="h-full w-auto object-contain" />
				<div
					class="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-gradient-to-t from-surface-2 to-transparent"
				/>
			</div>
		</template>
		<template #actions>
			<Button type="colored" color="brand" size="xl" native-type="button" @click="requestSignIn">
				<LogInIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.signInButton) }}
			</Button>
		</template>
	</EmptyState>

	<div v-else class="flex flex-col gap-4">
		<p class="m-0 text-secondary" :class="{ 'order-last': disclaimerPosition === 'bottom' }">
			<IntlFormatted :message-id="messages.description">
				<template #profile-link="{ children }">
					<RouterLink v-slot="{ href, navigate }" :to="profilePath" custom>
						<a :href="href" class="text-link" @click="handleProfileLinkClick($event, navigate)">
							<component :is="() => children" />
						</a>
					</RouterLink>
				</template>
				<template #docs-link="{ children }">
					<a href="https://docs.modrinth.com/" target="_blank" class="text-link">
						<component :is="() => children" />
					</a>
				</template>
			</IntlFormatted>
		</p>

		<hr
			v-if="disclaimerPosition === 'top'"
			class="m-0 h-px w-full border-none bg-divider"
			aria-hidden="true"
		/>

		<section class="flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.profilePicture) }}
				</h2>
				<div class="flex items-center gap-4">
					<Avatar :src="displayedAvatarUrl" size="md" circle :alt="auth.user.value.username" />
					<div class="flex flex-col gap-2">
						<FileButton
							:disabled="!globals?.max_user_icon_size || avatarSaving"
							:max-size="globals?.max_user_icon_size"
							:prompt="formatMessage(commonMessages.uploadImageButton)"
							accept="image/png,image/jpeg,image/gif,image/webp"
							size="md"
							@change="uploadAvatar"
						>
							<UploadIcon aria-hidden="true" />
						</FileButton>
						<Button
							v-if="displayedAvatarUrl"
							native-type="button"
							size="md"
							:disabled="avatarSaving"
							@click="removeAvatar"
						>
							<TrashIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.removeImageButton) }}
						</Button>
					</div>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(commonMessages.usernameLabel) }}
				</h2>
				<div v-if="changeUsername" class="flex flex-wrap gap-2">
					<Button native-type="button" size="md" @click="changeUsername()">
						<UserPenIcon aria-hidden="true" />
						{{ formatMessage(messages.changeUsername) }}
					</Button>
					<Button
						native-type="button"
						size="md"
						:disabled="refreshing || refreshCooldown || saving || hasUnsavedProfileFields"
						@click="refreshProfile"
					>
						<RefreshCwIcon aria-hidden="true" />
						{{ formatMessage(messages.refreshProfile) }}
					</Button>
				</div>
				<template v-else>
					<div class="flex items-center gap-2">
						<Input
							id="username-field"
							ref="usernameInput"
							v-model="profile.username"
							class="w-full max-w-md"
							:error="!usernameValid"
							:readonly="usernameLocked"
							@focus="onUsernameFocus"
						/>
						<span
							v-if="profile.username.length >= 30"
							class="shrink-0 text-secondary"
							:class="{ 'text-red': profile.username.length > USERNAME_MAX_LENGTH }"
						>
							{{ profile.username.length }}/{{ USERNAME_MAX_LENGTH }}
						</span>
					</div>
					<p class="m-0 text-secondary">
						{{ formatMessage(messages.usernameDescription) }}
					</p>
				</template>
			</div>

			<div class="flex flex-col gap-2.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.bioTitle) }}
				</h2>
				<Textarea id="bio-field" v-model="profile.bio" :error="!bioValid" />
				<div class="text-secondary" :class="{ 'text-red': !bioValid }">
					{{ profile.bio.length }}/{{ BIO_MAX_LENGTH }}
				</div>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.bioDescription) }}
				</p>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import {
	LogInIcon,
	RefreshCwIcon,
	ThinkingRinthbot,
	TrashIcon,
	UploadIcon,
	UserPenIcon,
} from '@modrinth/assets'
import { useDebounceFn } from '@vueuse/core'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'

import Avatar from '#ui/components/base/Avatar.vue'
import { Button, FileButton } from '#ui/components/base/buttons'
import EmptyState from '#ui/components/base/EmptyState.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import Textarea from '#ui/components/base/inputs/Textarea.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { defineMessages, useGlobals, useVIntl } from '#ui/composables'
import { type AuthUser, injectAuth, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils'

type ProfileFields = {
	username: string
	bio: string
}

const USERNAME_MAX_LENGTH = 39
const BIO_MAX_LENGTH = 160

const { data: globals } = useGlobals()

const props = withDefaults(
	defineProps<{
		patchUser: (userId: string, patch: Partial<ProfileFields>) => Promise<void>
		changeAvatar: (userId: string, file: Blob, extension: string) => Promise<void>
		deleteAvatar: (userId: string) => Promise<void>
		getAuthenticatedUser: () => Promise<AuthUser>
		disclaimerPosition?: 'top' | 'bottom'
		changeUsername?: () => void
		prepareUsernameChange?: () => Promise<boolean>
	}>(),
	{
		disclaimerPosition: 'top',
	},
)
const emit = defineEmits<{
	profileLinkClick: [event: MouseEvent]
}>()

const auth = injectAuth()
const notificationManager = injectNotificationManager()
const { formatMessage } = useVIntl()

const activeUserId = ref<string | null>(null)
const savedProfile = ref<ProfileFields>({ username: '', bio: '' })
const profile = ref<ProfileFields>({ username: '', bio: '' })
const avatarUrl = ref<string | null>(null)
const previewImageUrl = ref<string | null>(null)
const avatarSaving = ref(false)
const usernameInput = ref<{ focus: () => void; blur: () => void } | null>(null)
const usernameLocked = ref(props.prepareUsernameChange != null)
let usernamePrompted = false
const activeSaves = ref(0)
const refreshing = ref(false)
const refreshCooldown = ref(false)
let refreshCooldownTimer: ReturnType<typeof setTimeout> | undefined
let profileSaveQueue = Promise.resolve()
let profileFieldsSavePending = false

const saving = computed(() => activeSaves.value > 0)
const displayedAvatarUrl = computed(() => previewImageUrl.value ?? avatarUrl.value)
const profilePath = computed(
	() => `/user/${encodeURIComponent(auth.user.value?.username ?? profile.value.username)}`,
)
const usernameValid = computed(
	() => profile.value.username.length > 0 && profile.value.username.length <= USERNAME_MAX_LENGTH,
)
const bioValid = computed(() => profile.value.bio.length <= BIO_MAX_LENGTH)
const hasUnsavedProfileFields = computed(
	() =>
		(!props.changeUsername && profile.value.username !== savedProfile.value.username) ||
		profile.value.bio !== savedProfile.value.bio,
)

const debouncedSaveProfileFields = useDebounceFn(flushProfileFieldsSave, 500)

watch(
	() => auth.user.value,
	(user) => {
		if (!user || user.id !== activeUserId.value) {
			syncFromUser(user)
		}
	},
	{ immediate: true },
)

watch(
	() => [profile.value.username, profile.value.bio],
	() => {
		if (!hasUnsavedProfileFields.value) return
		profileFieldsSavePending = true
		void debouncedSaveProfileFields()
	},
)

function syncFromUser(user: AuthUser | null): void {
	revokePreviewImage()
	activeUserId.value = user?.id ?? null
	savedProfile.value = {
		username: user?.username ?? '',
		bio: user?.bio ?? '',
	}
	profile.value = { ...savedProfile.value }
	avatarUrl.value = user?.avatar_url ?? null
}

function revokePreviewImage(): void {
	if (previewImageUrl.value) {
		URL.revokeObjectURL(previewImageUrl.value)
		previewImageUrl.value = null
	}
}

function queueProfileSave(task: () => Promise<void>): Promise<void> {
	activeSaves.value++
	const run = profileSaveQueue.then(task).finally(() => {
		activeSaves.value--
	})
	profileSaveQueue = run.catch(() => undefined)
	return run
}

function notifySaveError(error: unknown): void {
	if (error instanceof Error && error.name === 'ReauthCancelled') {
		return
	}
	notificationManager.addNotification({
		type: 'error',
		title: formatMessage(messages.saveError),
		text: error instanceof Error ? error.message : formatMessage(messages.saveErrorDescription),
	})
}

async function refreshAuthUser(): Promise<AuthUser> {
	const refreshedUser = await props.getAuthenticatedUser()
	auth.user.value = refreshedUser
	return refreshedUser
}

function flushProfileFieldsSave(): Promise<void> {
	if (!profileFieldsSavePending) return Promise.resolve()
	profileFieldsSavePending = false
	return queueProfileSave(saveProfileFields)
}

async function saveProfileFields(): Promise<void> {
	const user = auth.user.value
	if (!user || user.id !== activeUserId.value) return

	const patch: Partial<ProfileFields> = {}
	if (
		!props.changeUsername &&
		usernameValid.value &&
		profile.value.username !== savedProfile.value.username
	) {
		patch.username = profile.value.username
	}
	if (bioValid.value && profile.value.bio !== savedProfile.value.bio) {
		patch.bio = profile.value.bio
	}
	if (Object.keys(patch).length === 0) return

	try {
		await props.patchUser(user.id, patch)
		savedProfile.value = { ...savedProfile.value, ...patch }
		await refreshAuthUser()
	} catch (error) {
		notifySaveError(error)
	}
}

function uploadAvatar(files: File[]): void {
	const file = files[0]
	const user = auth.user.value
	if (!file || !user) return

	revokePreviewImage()
	const preview = URL.createObjectURL(file)
	previewImageUrl.value = preview
	avatarSaving.value = true
	void queueProfileSave(async () => {
		try {
			const extension = file.type.split('/').at(-1)
			if (!extension) throw new Error('The selected image does not have a valid file type.')
			await props.changeAvatar(user.id, file, extension)
			avatarUrl.value = (await refreshAuthUser()).avatar_url ?? null
		} catch (error) {
			notifySaveError(error)
		} finally {
			if (previewImageUrl.value === preview) revokePreviewImage()
			avatarSaving.value = false
		}
	})
}

function removeAvatar(): void {
	const user = auth.user.value
	if (!user) return

	const previousAvatarUrl = avatarUrl.value
	revokePreviewImage()
	avatarUrl.value = null
	avatarSaving.value = true
	void queueProfileSave(async () => {
		try {
			await props.deleteAvatar(user.id)
			avatarUrl.value = (await refreshAuthUser()).avatar_url ?? null
		} catch (error) {
			avatarUrl.value = previousAvatarUrl
			notifySaveError(error)
		} finally {
			avatarSaving.value = false
		}
	})
}

function startRefreshCooldown(): void {
	refreshCooldown.value = true
	clearTimeout(refreshCooldownTimer)
	refreshCooldownTimer = setTimeout(() => {
		refreshCooldown.value = false
	}, 30_000)
}

async function refreshProfile(): Promise<void> {
	if (refreshing.value || refreshCooldown.value || saving.value || hasUnsavedProfileFields.value) {
		return
	}
	refreshing.value = true
	try {
		const user = await props.getAuthenticatedUser()
		auth.user.value = user
		syncFromUser(user)
		startRefreshCooldown()
	} catch (error) {
		if (error instanceof Error && error.name === 'ReauthCancelled') {
			return
		}
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.saveError),
			text: formatMessage(messages.saveErrorDescription),
		})
	} finally {
		refreshing.value = false
	}
}

async function onUsernameFocus(): Promise<void> {
	if (!props.prepareUsernameChange || !usernameLocked.value || usernamePrompted) return
	usernamePrompted = true
	try {
		const allowed = await props.prepareUsernameChange()
		if (!allowed) {
			usernameInput.value?.blur()
			return
		}
		usernameLocked.value = false
		await nextTick()
		usernameInput.value?.focus()
	} finally {
		usernamePrompted = false
	}
}

async function requestSignIn(): Promise<void> {
	await auth.requestSignIn('')
}

function handleProfileLinkClick(
	event: MouseEvent,
	navigate: (event?: MouseEvent) => unknown,
): void {
	emit('profileLinkClick', event)
	if (!event.defaultPrevented) {
		navigate(event)
	}
}

onBeforeUnmount(() => {
	void flushProfileFieldsSave()
	revokePreviewImage()
	clearTimeout(refreshCooldownTimer)
})

const messages = defineMessages({
	description: {
		id: 'settings.profile.public-information.description',
		defaultMessage:
			'Your profile information is publicly <profile-link>viewable on Modrinth</profile-link> and through the <docs-link>Modrinth API</docs-link>.',
	},
	profilePicture: {
		id: 'settings.profile.profile-picture.title',
		defaultMessage: 'Profile picture',
	},
	usernameDescription: {
		id: 'settings.profile.username.description',
		defaultMessage: 'A unique case-insensitive name to identify your profile.',
	},
	changeUsername: {
		id: 'settings.profile.username.change',
		defaultMessage: 'Change username',
	},
	refreshProfile: {
		id: 'settings.profile.refresh',
		defaultMessage: 'Refresh profile',
	},
	bioTitle: {
		id: 'settings.profile.bio.title',
		defaultMessage: 'Bio',
	},
	bioDescription: {
		id: 'settings.profile.bio.description',
		defaultMessage: 'A short description to tell everyone a little bit about you.',
	},
	signInRequiredTitle: {
		id: 'settings.profile.sign-in-required.title',
		defaultMessage: 'Modrinth account required',
	},
	signInRequiredDescription: {
		id: 'settings.profile.sign-in-required.description',
		defaultMessage: 'Sign in with a Modrinth account to customize your public profile.',
	},
	saveError: {
		id: 'settings.profile.save-error',
		defaultMessage: 'Failed to update profile',
	},
	saveErrorDescription: {
		id: 'settings.profile.save-error-description',
		defaultMessage: 'An error occurred while updating your profile. Please try again.',
	},
})
</script>
