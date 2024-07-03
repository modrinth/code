<template>
  <div class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      title="Are you sure you want to delete this application?"
      description="This will permanently delete this application and revoke all access tokens. (forever!)"
      proceed-label="Delete this application"
      @proceed="removeApp(editingId)"
    />
    <Modal ref="appModal" header="Application information">
      <div class="universal-modal">
        <label for="app-name"><span class="label__title">Name</span> </label>
        <input
          id="app-name"
          v-model="name"
          maxlength="2048"
          type="text"
          autocomplete="off"
          placeholder="Enter the application's name..."
        />
        <label v-if="editingId" for="app-icon"><span class="label__title">Icon</span> </label>
        <div v-if="editingId" class="icon-submission">
          <Avatar size="md" :src="icon" />
          <FileInput
            :max-size="262144"
            class="btn"
            prompt="Upload icon"
            accept="image/png,image/jpeg,image/gif,image/webp"
            @change="onImageSelection"
          >
            <UploadIcon />
          </FileInput>
        </div>
        <label v-if="editingId" for="app-url">
          <span class="label__title">URL</span>
        </label>
        <input
          v-if="editingId"
          id="app-url"
          v-model="url"
          maxlength="255"
          type="url"
          autocomplete="off"
          placeholder="https://example.com"
        />
        <label v-if="editingId" for="app-description">
          <span class="label__title">Description</span>
        </label>
        <textarea
          v-if="editingId"
          id="app-description"
          v-model="description"
          class="description-textarea"
          maxlength="255"
          type="text"
          autocomplete="off"
          placeholder="Enter the application's description..."
        />
        <label for="app-scopes"><span class="label__title">Scopes</span> </label>
        <div id="app-scopes" class="checkboxes">
          <Checkbox
            v-for="scope in scopeList"
            :key="scope"
            :label="scopesToLabels(getScopeValue(scope)).join(', ')"
            :model-value="hasScope(scopesVal, scope)"
            @update:model-value="() => (scopesVal = toggleScope(scopesVal, scope))"
          />
        </div>
        <label for="app-redirect-uris"><span class="label__title">Redirect uris</span> </label>
        <div class="uri-input-list">
          <div v-for="(_, index) in redirectUris" :key="index">
            <div class="input-group url-input-group-fixes">
              <input
                v-model="redirectUris[index]"
                maxlength="2048"
                type="url"
                autocomplete="off"
                placeholder="https://example.com/auth/callback"
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
                <PlusIcon /> Add more
              </Button>
            </div>
          </div>
          <div v-if="redirectUris.length <= 0">
            <Button color="primary" icon-only @click="() => redirectUris.push('')">
              <PlusIcon /> Add a redirect uri
            </Button>
          </div>
        </div>

        <div class="submit-row input-group push-right">
          <button class="iconified-button" @click="$refs.appModal.hide()">
            <XIcon />
            Cancel
          </button>
          <button
            v-if="editingId"
            :disabled="!canSubmit"
            type="button"
            class="iconified-button brand-button"
            @click="editApp"
          >
            <SaveIcon />
            Save changes
          </button>
          <button
            v-else
            :disabled="!canSubmit"
            type="button"
            class="iconified-button brand-button"
            @click="createApp"
          >
            <PlusIcon />
            Create App
          </button>
        </div>
      </div>
    </Modal>

    <div class="header__row">
      <div class="header__title">
        <h2>{{ formatMessage(commonSettingsMessages.applications) }}</h2>
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
        <PlusIcon /> New Application
      </button>
    </div>
    <p>
      Applications can be used to authenticate Modrinth's users with your products. For more
      information, see
      <a class="text-link" href="https://docs.modrinth.com">Modrinth's API documentation</a>.
    </p>
    <div v-for="app in usersApps" :key="app.id" class="universal-card recessed token">
      <div class="token-info">
        <div class="token-icon">
          <Avatar size="sm" :src="app.icon_url" />
          <div>
            <h2 class="token-title">{{ app.name }}</h2>
            <div>Created on {{ new Date(app.created).toLocaleDateString() }}</div>
          </div>
        </div>
        <div>
          <label for="token-information">
            <span class="label__title">About</span>
          </label>
          <div class="token-content">
            <div>
              Client ID
              <CopyCode :text="app.id" />
            </div>
            <div v-if="!!clientCreatedInState(app.id)">
              <div>
                Client Secret <CopyCode :text="clientCreatedInState(app.id)?.client_secret" />
              </div>
              <div class="secret_disclaimer">
                <i> Save your secret now, it will be hidden after you leave this page! </i>
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
          Edit
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
          Delete
        </Button>
      </div>
    </div>
  </div>
</template>
<script setup>
import {
  FileInput,
  UploadIcon,
  PlusIcon,
  Avatar,
  XIcon,
  Button,
  Checkbox,
  TrashIcon,
  EditIcon,
  SaveIcon,
  CopyCode,
  ConfirmModal,
} from 'omorphia'
import Modal from '~/components/ui/Modal.vue'

import {
  scopeList,
  hasScope,
  toggleScope,
  useScopes,
  getScopeValue,
} from '~/composables/auth/scopes.ts'
import { commonSettingsMessages } from '~/utils/common-messages.ts'

const { formatMessage } = useVIntl()

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'Applications - Modrinth',
})

const data = useNuxtApp()
const { scopesToLabels } = useScopes()

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
  }
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
  } catch (err) {
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

    data.$notify({
      group: 'main',
      title: 'Icon updated',
      text: 'Your application icon has been updated.',
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
    data.$notify({
      group: 'main',
      title: 'An error occurred',
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
    data.$notify({
      group: 'main',
      title: 'An error occurred',
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
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
  stopLoading()
}
</script>
<style lang="scss" scoped>
.description-textarea {
  height: 6rem;
  resize: vertical;
}

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
.checkboxes {
  display: grid;
  column-gap: 0.5rem;

  @media screen and (min-width: 432px) {
    grid-template-columns: repeat(2, 1fr);
  }

  @media screen and (min-width: 800px) {
    grid-template-columns: repeat(3, 1fr);
  }
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
