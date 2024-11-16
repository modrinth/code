<template>
  <div class="experimental-styles-within">
    <template v-if="flow">
      <label for="two-factor-code">
        <span class="label__title">{{ formatMessage(messages.twoFactorCodeLabel) }}</span>
        <span class="label__description">
          {{ formatMessage(messages.twoFactorCodeLabelDescription) }}
        </span>
      </label>
      <input
        id="two-factor-code"
        v-model="twoFactorCode"
        maxlength="11"
        type="text"
        :placeholder="formatMessage(messages.twoFactorCodeInputPlaceholder)"
        autocomplete="one-time-code"
        autofocus
        @keyup.enter="begin2FASignIn"
      />

      <button class="btn btn-primary continue-btn" @click="begin2FASignIn">
        {{ formatMessage(commonMessages.signInButton) }} <RightArrowIcon />
      </button>
    </template>
    <template v-else>
      <SignInUi
        :completed-turnstile="!!turnstile"
        forgot-password-link="/auth/reset-password"
        :create-account-link="signUpLink"
        third-party-button-handler-type="href"
        :third-party-button-handler="(service) => getAuthUrl(service, redirectTarget)"
        :on-submit="signIn"
      >
        <template #turnstile>
          <NuxtTurnstile
            ref="turnstile"
            v-model="token"
            :options="{ theme: $theme.active === 'light' ? 'light' : 'dark' }"
            data-size="flexible"
          />
        </template>
      </SignInUi>
    </template>
  </div>
</template>

<script setup>
import { RightArrowIcon } from "@modrinth/assets";
import { commonMessages, SignInUi } from "@modrinth/ui";

const { formatMessage } = useVIntl();

const messages = defineMessages({
  additionalOptionsLabel: {
    id: "auth.sign-in.additional-options",
    defaultMessage:
      "<forgot-password-link>Forgot password?</forgot-password-link> • <create-account-link>Create an account</create-account-link>",
  },
  emailUsernameLabel: {
    id: "auth.sign-in.email-username.label",
    defaultMessage: "Email or username",
  },
  passwordLabel: {
    id: "auth.sign-in.password.label",
    defaultMessage: "Password",
  },
  signInWithLabel: {
    id: "auth.sign-in.sign-in-with",
    defaultMessage: "Sign in with",
  },
  signInTitle: {
    id: "auth.sign-in.title",
    defaultMessage: "Sign In",
  },
  twoFactorCodeInputPlaceholder: {
    id: "auth.sign-in.2fa.placeholder",
    defaultMessage: "Enter code...",
  },
  twoFactorCodeLabel: {
    id: "auth.sign-in.2fa.label",
    defaultMessage: "Enter two-factor code",
  },
  twoFactorCodeLabelDescription: {
    id: "auth.sign-in.2fa.description",
    defaultMessage: "Please enter a two-factor code to proceed.",
  },
  usePasswordLabel: {
    id: "auth.sign-in.use-password",
    defaultMessage: "Or use a password",
  },
});

useHead({
  title() {
    return `${formatMessage(messages.signInTitle)} - Modrinth`;
  },
});

const auth = await useAuth();
const route = useNativeRoute();

const redirectTarget = route.query.redirect || "";

if (route.query.code && !route.fullPath.includes("new_account=true")) {
  await finishSignIn();
}

if (auth.value.user) {
  await finishSignIn();
}

const turnstile = ref();

const token = ref("");

const flow = ref(route.query.flow);

const signUpLink = computed(
  () => `/auth/sign-up${route.query.redirect ? `?redirect=${route.query.redirect}` : ""}`,
);

async function signIn(email, password, token) {
  startLoading();
  try {
    const res = await useBaseFetch("auth/login", {
      method: "POST",
      body: {
        username: email,
        password,
        challenge: token,
      },
    });

    if (res.flow) {
      flow.value = res.flow;
    } else {
      await finishSignIn(res.session);
    }
  } catch (err) {
    addNotification({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: "error",
    });
    turnstile.value?.reset();
  }
  stopLoading();
}

const twoFactorCode = ref(null);
async function begin2FASignIn() {
  startLoading();
  try {
    const res = await useBaseFetch("auth/login/2fa", {
      method: "POST",
      body: {
        flow: flow.value,
        code: twoFactorCode.value ? twoFactorCode.value.toString() : twoFactorCode.value,
      },
    });

    await finishSignIn(res.session);
  } catch (err) {
    addNotification({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: "error",
    });
    turnstile.value?.reset();
  }
  stopLoading();
}

async function finishSignIn(token) {
  if (token) {
    await useAuth(token);
    await useUser();
  }

  if (route.query.redirect) {
    const redirect = decodeURIComponent(route.query.redirect);
    await navigateTo(redirect, {
      replace: true,
    });
  } else {
    await navigateTo("/dashboard");
  }
}
</script>
