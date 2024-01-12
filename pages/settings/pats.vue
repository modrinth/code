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
            v-for="scope in scopeList"
            :key="scope"
            :label="scopesToLabels(getScopeValue(scope)).join(', ')"
            :model-value="hasScope(scopesVal, scope)"
            @update:model-value="scopesVal = toggleScope(scopesVal, scope)"
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
import { PlusIcon, XIcon, Checkbox, TrashIcon, EditIcon, SaveIcon, ConfirmModal } from 'omorphia'

import {
  hasScope,
  scopeList,
  toggleScope,
  useScopes,
  getScopeValue,
} from '~/composables/auth/scopes.ts'

import CopyCode from '~/components/ui/CopyCode.vue'
import Modal from '~/components/ui/Modal.vue'

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'PATs - Modrinth',
})

const data = useNuxtApp()
const { scopesToLabels } = useScopes()
const patModal = ref()

const editPatIndex = ref(null)

const name = ref(null)
const scopesVal = ref(BigInt(0))
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
        scopes: Number(scopesVal.value),
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
        scopes: Number(scopesVal.value),
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
