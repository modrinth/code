<template>
  <div>
    <section class="card">
      <h2>{{ formatMessage(messages.title) }}</h2>
      <p>
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
            {{ formatMessage(messages.profilePictureReset) }}
          </Button>
        </div>
      </div>
      <label for="username-field">
        <span class="label__title">{{ formatMessage(messages.usernameTitle) }}</span>
        <span class="label__description">
          {{ formatMessage(messages.usernameDescription) }}
        </span>
      </label>
      <input id="username-field" v-model="username" type="text" />
      <label for="bio-field">
        <span class="label__title">{{ formatMessage(messages.bioTitle) }}</span>
        <span class="label__description">
          {{ formatMessage(messages.bioDescription) }}
        </span>
      </label>
      <textarea id="bio-field" v-model="bio" type="text" />
      <div v-if="hasUnsavedChanges" class="input-group">
        <Button color="primary" :action="() => saveChanges()">
          <SaveIcon /> {{ formatMessage(commonMessages.saveChangesButton) }}
        </Button>
        <Button :action="() => cancel()">
          <XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
        </Button>
      </div>
      <div v-else class="input-group">
        <Button disabled color="primary" :action="() => saveChanges()">
          <SaveIcon />
          {{
            saved
              ? formatMessage(commonMessages.changesSavedLabel)
              : formatMessage(commonMessages.saveChangesButton)
          }}
        </Button>
        <Button :link="`/user/${auth.user.username}`">
          <UserIcon /> {{ formatMessage(commonMessages.visitYourProfile) }}
        </Button>
      </div>
    </section>
  </div>
</template>

<script setup>
import {
  Button,
  UserIcon,
  SaveIcon,
  Avatar,
  FileInput,
  UploadIcon,
  UndoIcon,
  XIcon,
} from 'omorphia'
import { commonMessages } from '~/utils/common-messages.ts'

useHead({
  title: 'Profile settings - Modrinth',
})

definePageMeta({
  middleware: 'auth',
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
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
  profilePictureReset: {
    id: 'settings.profile.profile-picture.reset',
    defaultMessage: 'Reset',
  },
  usernameTitle: {
    id: 'settings.profile.username.title',
    defaultMessage: 'Username',
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

const username = ref(auth.value.user.username)
const bio = ref(auth.value.user.bio)
const avatarUrl = ref(auth.value.user.avatar_url)
const icon = shallowRef(null)
const previewImage = shallowRef(null)
const saved = ref(false)

const hasUnsavedChanges = computed(
  () =>
    username.value !== auth.value.user.username ||
    bio.value !== auth.value.user.bio ||
    previewImage.value
)

function showPreviewImage(files) {
  const reader = new FileReader()
  icon.value = files[0]
  reader.readAsDataURL(icon.value)
  reader.onload = (event) => {
    previewImage.value = event.target.result
  }
}

function cancel() {
  icon.value = null
  previewImage.value = null
  username.value = auth.value.user.username
  bio.value = auth.value.user.bio
}

async function saveChanges() {
  startLoading()
  try {
    if (icon.value) {
      await useBaseFetch(
        `user/${auth.value.user.id}/icon?ext=${
          icon.value.type.split('/')[icon.value.type.split('/').length - 1]
        }`,
        {
          method: 'PATCH',
          body: icon.value,
        }
      )
      icon.value = null
      previewImage.value = null
    }

    const body = {}

    if (auth.value.user.username !== username.value) {
      body.username = username.value
    }

    if (auth.value.user.bio !== bio.value) {
      body.bio = bio.value
    }

    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: 'PATCH',
      body,
    })
    await useAuth(auth.value.token)
    avatarUrl.value = auth.value.user.avatar_url
    saved.value = true
  } catch (err) {
    addNotification({
      group: 'main',
      title: 'An error occurred',
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
  stopLoading()
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
