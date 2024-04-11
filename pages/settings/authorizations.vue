<template>
  <div class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      title="Are you sure you want to revoke this application?"
      description="This will revoke the application's access to your account. You can always re-authorize it later."
      proceed-label="Revoke"
      @proceed="revokeApp(revokingId)"
    />
    <h2>{{ formatMessage(commonSettingsMessages.authorizedApps) }}</h2>
    <p>
      When you authorize an application with your Modrinth account, you grant it access to your
      account. You can manage and review access to your account here at any time.
    </p>
    <div v-if="appInfoLookup.length === 0" class="universal-card recessed">
      You have not authorized any applications.
    </div>
    <div
      v-for="authorization in appInfoLookup"
      :key="authorization.id"
      class="universal-card recessed token"
    >
      <div class="token-content">
        <div>
          <div class="icon-name">
            <Avatar :src="authorization.app.icon_url" />
            <div>
              <h2 class="token-title">
                {{ authorization.app.name }}
              </h2>
              <div>
                by
                <nuxt-link class="text-link" :to="'/user/' + authorization.owner.id">{{
                  authorization.owner.username
                }}</nuxt-link>
                <template v-if="authorization.app.url">
                  <span> ⋅ </span>
                  <nuxt-link class="text-link" :to="authorization.app.url">
                    {{ authorization.app.url }}
                  </nuxt-link>
                </template>
              </div>
            </div>
          </div>
        </div>
        <div>
          <template v-if="authorization.app.description">
            <label for="app-description">
              <span class="label__title"> About this app </span>
            </label>
            <div id="app-description">{{ authorization.app.description }}</div>
          </template>

          <label for="app-scope-list">
            <span class="label__title">Scopes</span>
          </label>
          <div class="scope-list">
            <div
              v-for="scope in scopesToDefinitions(authorization.scopes)"
              :key="scope"
              class="scope-list-item"
            >
              <div class="scope-list-item-icon">
                <CheckIcon />
              </div>
              {{ scope }}
            </div>
          </div>
        </div>
      </div>

      <div class="input-group">
        <Button
          color="danger"
          icon-only
          @click="
            () => {
              revokingId = authorization.app_id
              $refs.modal_confirm.show()
            }
          "
        >
          <TrashIcon />
          Revoke
        </Button>
      </div>
    </div>
  </div>
</template>
<script setup>
import { Button, TrashIcon, CheckIcon, ConfirmModal, Avatar } from 'omorphia'
import { commonSettingsMessages } from '~/utils/common-messages.ts'
import { useScopes } from '~/composables/auth/scopes.ts'

const { formatMessage } = useVIntl()

const { scopesToDefinitions } = useScopes()

const revokingId = ref(null)

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'Authorizations - Modrinth',
})

const { data: usersApps, refresh } = await useAsyncData('userAuthorizations', () =>
  useBaseFetch(`oauth/authorizations`, {
    internal: true,
  })
)

const { data: appInformation } = await useAsyncData(
  'appInfo',
  () =>
    useBaseFetch('oauth/apps', {
      internal: true,
      query: {
        ids: usersApps.value.map((c) => c.app_id).join(','),
      },
    }),
  {
    watch: usersApps,
  }
)

const { data: appCreatorsInformation } = await useAsyncData(
  'appCreatorsInfo',
  () =>
    useBaseFetch('users', {
      query: {
        ids: JSON.stringify(appInformation.value.map((c) => c.created_by)),
      },
    }),
  {
    watch: appInformation,
  }
)

const appInfoLookup = computed(() => {
  return usersApps.value.map((app) => {
    const info = appInformation.value.find((c) => c.id === app.app_id)
    const owner = appCreatorsInformation.value.find((c) => c.id === info.created_by)
    return {
      ...app,
      app: info || null,
      owner: owner || null,
    }
  })
})

async function revokeApp(id) {
  try {
    await useBaseFetch(`oauth/authorizations`, {
      internal: true,
      method: 'DELETE',
      query: {
        client_id: id,
      },
    })
    revokingId.value = null
    await refresh()
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
}
</script>

<style lang="scss" scoped>
.input-group {
  // Overrides for omorphia compat
  > * {
    padding: var(--gap-sm) var(--gap-lg) !important;
  }
}

.scope-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  gap: var(--gap-sm);

  .scope-list-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: 0.25rem;
    background-color: var(--color-gray-200);
    color: var(--color-gray-700);
    font-size: 0.875rem;
    font-weight: 500;
    line-height: 1.25rem;

    // avoid breaking text or overflowing
    white-space: nowrap;
    overflow: hidden;
  }

  .scope-list-item-icon {
    width: 1.25rem;
    height: 1.25rem;
    flex: 0 0 auto;

    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-green);
    color: var(--color-raised-bg);
  }
}

.icon-name {
  display: flex;
  align-items: flex-start;
  gap: var(--gap-lg);
  padding-bottom: var(--gap-sm);
}

.token-content {
  width: 100%;

  .token-title {
    margin-bottom: var(--spacing-card-xs);
  }
}

.token {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  @media screen and (min-width: 800px) {
    flex-direction: row;
    align-items: flex-start;
  }
}
</style>
