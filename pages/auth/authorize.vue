<template>
  <div>
    <div v-if="error" class="oauth-items">
      <div>
        <h1>{{ formatMessage(commonMessages.errorLabel) }}</h1>
      </div>
      <p>
        <span>{{ error.data.error }}: </span>
        {{ error.data.description }}
      </p>
    </div>
    <div v-else class="oauth-items">
      <div class="connected-items">
        <div class="profile-pics">
          <Avatar size="md" :src="app.icon_url" />
          <!-- <img class="profile-pic" :src="app.icon_url" alt="User profile picture" /> -->
          <div class="connection-indicator">→</div>
          <Avatar size="md" circle :src="auth.user.avatar_url" />
          <!-- <img class="profile-pic" :src="auth.user.avatar_url" alt="User profile picture" /> -->
        </div>
      </div>
      <div class="title">
        <h1>{{ formatMessage(messages.title, { appName: app.name }) }}</h1>
      </div>
      <div class="auth-info">
        <div class="scope-heading">
          <IntlFormatted
            :message-id="messages.appInfo"
            :values="{
              appName: app.name,
              creator: createdBy.username,
            }"
          >
            <template #strong="{ children }">
              <strong>
                <component :is="() => normalizeChildren(children)" />
              </strong>
            </template>
            <template #creator-link="{ children }">
              <nuxt-link class="text-link" :to="'/user/' + createdBy.id">
                <component :is="() => normalizeChildren(children)" />
              </nuxt-link>
            </template>
          </IntlFormatted>
        </div>
        <div class="scope-items">
          <div v-for="scopeItem in scopeDefinitions" :key="scopeItem">
            <div class="scope-item">
              <div class="scope-icon">
                <CheckIcon />
              </div>
              {{ scopeItem }}
            </div>
          </div>
        </div>
      </div>
      <div class="button-row">
        <Button class="wide-button" large :action="onReject" :disabled="pending">
          <XIcon />
          {{ formatMessage(messages.decline) }}
        </Button>
        <Button class="wide-button" color="primary" large :action="onAuthorize" :disabled="pending">
          <CheckIcon />
          {{ formatMessage(messages.authorize) }}
        </Button>
      </div>
      <div class="redirection-notice">
        <p class="redirect-instructions">
          <IntlFormatted :message-id="messages.redirectUrl" :values="{ url: redirectUri }">
            <template #redirect-url="{ children }">
              <span class="redirect-url">
                <component :is="() => normalizeChildren(children)" />
              </span>
            </template>
          </IntlFormatted>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Button, XIcon, CheckIcon, Avatar } from 'omorphia'
import { useBaseFetch } from '@/composables/fetch.js'
import { useAuth } from '@/composables/auth.js'

import { useScopes } from '@/composables/auth/scopes.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  appInfo: {
    id: 'auth.authorize.app-info',
    defaultMessage:
      '<strong>{appName}</strong> by <creator-link>{creator}</creator-link> will be able to:',
  },
  authorize: {
    id: 'auth.authorize.action.authorize',
    defaultMessage: 'Authorize',
  },
  decline: {
    id: 'auth.authorize.action.decline',
    defaultMessage: 'Decline',
  },
  noRedirectUrlError: {
    id: 'auth.authorize.error.no-redirect-url',
    defaultMessage: 'No redirect location found in response',
  },
  redirectUrl: {
    id: 'auth.authorize.redirect-url',
    defaultMessage: 'You will be redirected to <redirect-url>{url}</redirect-url>',
  },
  title: {
    id: 'auth.authorize.authorize-app-name',
    defaultMessage: 'Authorize {appName}',
  },
})

const data = useNuxtApp()

const router = useNativeRoute()
const auth = await useAuth()
const { scopesToDefinitions } = useScopes()

const clientId = router.query?.client_id || false
const redirectUri = router.query?.redirect_uri || false
const scope = router.query?.scope || false
const state = router.query?.state || false

const getFlowIdAuthorization = async () => {
  const query = {
    client_id: clientId,
    redirect_uri: redirectUri,
    scope,
  }
  if (state) {
    query.state = state
  }

  const authorization = await useBaseFetch('oauth/authorize', {
    method: 'GET',
    internal: true,
    query,
  }) // This will contain the flow_id and oauth_client_id for accepting the oauth on behalf of the user

  if (typeof authorization === 'string') {
    await navigateTo(authorization, {
      external: true,
    })
  }

  return authorization
}

const {
  data: authorizationData,
  pending,
  error,
} = await useAsyncData('authorization', getFlowIdAuthorization)

const { data: app } = await useAsyncData('oauth/app/' + clientId, () =>
  useBaseFetch('oauth/app/' + clientId, {
    method: 'GET',
    internal: true,
  })
)

const scopeDefinitions = scopesToDefinitions(BigInt(authorizationData.value?.requested_scopes || 0))

const { data: createdBy } = await useAsyncData('user/' + app.value.created_by, () =>
  useBaseFetch('user/' + app.value.created_by, {
    method: 'GET',
    apiVersion: 3,
  })
)

const onAuthorize = async () => {
  try {
    const res = await useBaseFetch('oauth/accept', {
      method: 'POST',
      internal: true,
      body: {
        flow: authorizationData.value.flow_id,
      },
    })

    if (typeof res === 'string') {
      navigateTo(res, {
        external: true,
      })
      return
    }

    throw new Error(formatMessage(messages.noRedirectUrlError))
  } catch (error) {
    data.$notify({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
}

const onReject = async () => {
  try {
    const res = await useBaseFetch('oauth/reject', {
      method: 'POST',
      body: {
        flow: authorizationData.value.flow_id,
      },
    })

    if (typeof res === 'string') {
      navigateTo(res, {
        external: true,
      })
      return
    }

    throw new Error(formatMessage(messages.noRedirectUrlError))
  } catch (error) {
    data.$notify({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
}

definePageMeta({
  middleware: 'auth',
})
</script>

<style scoped lang="scss">
.oauth-items {
  display: flex;
  flex-direction: column;
  gap: var(--gap-xl);
}

.scope-items {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
}

.scope-item {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);
}

.scope-icon {
  display: flex;

  color: var(--color-raised-bg);
  background-color: var(--color-green);
  aspect-ratio: 1;
  border-radius: 50%;
  padding: var(--gap-xs);
}
.title {
  margin-inline: auto;

  h1 {
    margin-bottom: 0 !important;
  }
}
.redirection-notice {
  display: flex;
  flex-direction: column;
  gap: var(--gap-xs);
  text-align: center;

  .redirect-instructions {
    font-size: var(--font-size-sm);
  }

  .redirect-url {
    font-weight: bold;
  }
}

.wide-button {
  width: 100% !important;
}

.button-row {
  display: flex;
  flex-direction: row;
  gap: var(--gap-xs);
  justify-content: center;
}
.auth-info {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
}

.scope-heading {
  margin-bottom: var(--gap-sm);
}

.profile-pics {
  width: 100%;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-evenly;

  .connection-indicator {
    // Make sure the text sits in the middle and is centered.
    // Make the text large, and make sure it's not selectable.
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    user-select: none;

    color: var(--color-primary);
  }
}

.profile-pic {
  width: 6rem;
  height: 6rem;
  border-radius: 50%;
  margin: 0 1rem;
}

.dotted-border-line {
  width: 75%;
  border: 0.1rem dashed var(--color-divider);
}

.connected-items {
  // Display dotted-border-line under profile-pics and centered behind them
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
  margin-top: 1rem;

  // Display profile-pics on top of dotted-border-line
  .profile-pics {
    position: relative;
    z-index: 2;
  }

  // Display dotted-border-line behind profile-pics
  .dotted-border-line {
    position: absolute;
    z-index: 1;
  }
}
</style>
