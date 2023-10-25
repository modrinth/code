<template>
  <div class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      title="Are you sure you want to delete this token?"
      description="This will remove this token forever (like really forever)."
      proceed-label="Delete this token"
      @proceed="removePat(deletePatIndex)"
    />
    <Modal
      ref="patModal"
      :header="`${editPatIndex !== null ? 'Edit' : 'Create'} personal access token`"
    >
      <div class="universal-modal">
        <label for="pat-name"><span class="label__title">Name</span> </label>
        <input
          id="pat-name"
          v-model="name"
          maxlength="2048"
          type="email"
          placeholder="Enter the PAT's name..."
        />
        <label for="pat-scopes"><span class="label__title">Scopes</span> </label>
        <div id="pat-scopes" class="checkboxes">
          <Checkbox
            v-for="(scope, index) in scopes"
            :key="scope"
            v-tooltip="
              scope.startsWith('_')
                ? 'This scope is not allowed to be used with personal access tokens.'
                : null
            "
            :disabled="scope.startsWith('_')"
            :label="scope.replace('_', '')"
            :model-value="(scopesVal & (1 << index)) === 1 << index"
            @update:model-value="scopesVal ^= 1 << index"
          />
        </div>
        <label for="pat-name"><span class="label__title">Expires</span> </label>
        <input id="pat-name" v-model="expires" type="date" />
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.patModal.hide()">
            <XIcon />
            Cancel
          </button>
          <button
            v-if="editPatIndex !== null"
            :disabled="loading || !name || !expires"
            type="button"
            class="iconified-button brand-button"
            @click="editPat"
          >
            <SaveIcon />
            Save changes
          </button>
          <button
            v-else
            :disabled="loading || !name || !expires"
            type="button"
            class="iconified-button brand-button"
            @click="createPat"
          >
            <PlusIcon />
            Create PAT
          </button>
        </div>
      </div>
    </Modal>

    <div class="header__row">
      <div class="header__title">
        <h2>Personal Access Tokens</h2>
      </div>
      <button
        class="btn btn-primary"
        @click="
          () => {
            name = null
            scopesVal = 0
            expires = null
            editPatIndex = null
            $refs.patModal.show()
          }
        "
      >
        <PlusIcon /> Create a PAT
      </button>
    </div>
    <p>
      PATs can be used to access Modrinth's API. For more information, see
      <a class="text-link" href="https://docs.modrinth.com">Modrinth's API documentation</a>. They
      can be created and revoked at any time.
    </p>
    <div v-for="(pat, index) in pats" :key="pat.id" class="universal-card recessed token">
      <div>
        <div>
          <strong>{{ pat.name }}</strong>
        </div>
        <div>
          <template v-if="pat.access_token">
            <CopyCode :text="pat.access_token" />
          </template>
          <template v-else>
            <span
              v-tooltip="
                pat.last_used ? $dayjs(pat.last_login).format('MMMM D, YYYY [at] h:mm A') : null
              "
            >
              <template v-if="pat.last_used">Last used {{ fromNow(pat.last_used) }}</template>
              <template v-else>Never used</template>
            </span>
            ⋅
            <span v-tooltip="$dayjs(pat.expires).format('MMMM D, YYYY [at] h:mm A')">
              <template v-if="new Date(pat.expires) > new Date()">
                Expires {{ fromNow(pat.expires) }}
              </template>
              <template v-else> Expired {{ fromNow(pat.expires) }} </template>
            </span>
            ⋅
            <span v-tooltip="$dayjs(pat.created).format('MMMM D, YYYY [at] h:mm A')">
              Created {{ fromNow(pat.created) }}
            </span>
          </template>
        </div>
      </div>
      <div class="input-group">
        <button
          class="iconified-button raised-button"
          @click="
            () => {
              editPatIndex = index
              name = pat.name
              scopesVal = pat.scopes
              expires = $dayjs(pat.expires).format('YYYY-MM-DD')
              $refs.patModal.show()
            }
          "
        >
          <EditIcon /> Edit token
        </button>
        <button
          class="iconified-button raised-button"
          @click="
            () => {
              deletePatIndex = pat.id
              $refs.modal_confirm.show()
            }
          "
        >
          <TrashIcon /> Revoke token
        </button>
      </div>
    </div>
  </div>
</template>
<script setup>
import {
  PlusIcon,
  Modal,
  XIcon,
  Checkbox,
  TrashIcon,
  EditIcon,
  SaveIcon,
  ConfirmModal,
} from 'omorphia'
import CopyCode from '~/components/ui/CopyCode.vue'

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'PATs - Modrinth',
})

const scopes = [
  'Read user email',
  'Read user data',
  'Write user data',
  '_Delete your account',
  '_Write auth data',
  'Read notifications',
  'Write notifications',
  'Read payouts',
  'Write payouts',
  'Read analytics',
  'Create projects',
  'Read projects',
  'Write projects',
  'Delete projects',
  'Create versions',
  'Read versions',
  'Write versions',
  'Delete versions',
  'Create reports',
  'Read reports',
  'Write reports',
  'Delete reports',
  'Read threads',
  'Write threads',
  '_Create PATs',
  '_Read PATs',
  '_Write PATs',
  '_Delete PATs',
  '_Read sessions',
  '_Delete sessions',
]

const data = useNuxtApp()
const patModal = ref()

const editPatIndex = ref(null)

const name = ref(null)
const scopesVal = ref(0)
const expires = ref(null)

const deletePatIndex = ref(null)

const loading = ref(false)

const { data: pats, refresh } = await useAsyncData('pat', () => useBaseFetch('pat'))

async function createPat() {
  startLoading()
  loading.value = true
  try {
    const res = await useBaseFetch('pat', {
      method: 'POST',
      body: {
        name: name.value,
        scopes: scopesVal.value,
        expires: data.$dayjs(expires.value).toISOString(),
      },
    })
    pats.value.push(res)
    patModal.value.hide()
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

async function editPat() {
  startLoading()
  loading.value = true
  try {
    await useBaseFetch(`pat/${pats.value[editPatIndex.value].id}`, {
      method: 'PATCH',
      body: {
        name: name.value,
        scopes: scopesVal.value,
        expires: data.$dayjs(expires.value).toISOString(),
      },
    })
    await refresh()
    patModal.value.hide()
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

async function removePat(id) {
  startLoading()
  try {
    pats.value = pats.value.filter((x) => x.id !== id)
    await useBaseFetch(`pat/${id}`, {
      method: 'DELETE',
    })
    await refresh()
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

.token {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  @media screen and (min-width: 800px) {
    flex-direction: row;
    align-items: center;

    .input-group {
      margin-left: auto;
    }
  }
}
</style>
