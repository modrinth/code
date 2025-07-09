<template>
  <div class="welcome-box has-bot">
    <img :src="WavingRinthbot" alt="Waving Modrinth Bot" class="welcome-box__waving-bot" />
    <div class="welcome-box__top-glow" />
    <div class="welcome-box__body">
      <h1 class="welcome-box__title">
        {{ formatMessage(messages.welcomeLongTitle) }}
      </h1>

      <p class="welcome-box__subtitle">
        <IntlFormatted :message-id="messages.welcomeDescription">
          <template #bold="{ children }">
            <strong>
              <component :is="() => normalizeChildren(children)" />
            </strong>
          </template>
        </IntlFormatted>
      </p>

      <Checkbox
        v-model="subscribe"
        class="subscribe-btn"
        :label="formatMessage(messages.subscribeCheckbox)"
        :description="formatMessage(messages.subscribeCheckbox)"
      />

      <button class="btn btn-primary centered-btn" @click="continueSignUp">
        {{ formatMessage(commonMessages.continueButton) }}
        <RightArrowIcon />
      </button>

      <p class="tos-text">
        <IntlFormatted :message-id="messages.tosLabel">
          <template #terms-link="{ children }">
            <NuxtLink to="/legal/terms" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
          <template #privacy-policy-link="{ children }">
            <NuxtLink to="/legal/privacy" class="text-link">
              <component :is="() => children" />
            </NuxtLink>
          </template>
        </IntlFormatted>
      </p>
    </div>
  </div>
</template>

<script setup>
import { Checkbox, commonMessages } from "@modrinth/ui";
import { RightArrowIcon, WavingRinthbot } from "@modrinth/assets";

const route = useRoute();

const { formatMessage } = useVIntl();

const messages = defineMessages({
  subscribeCheckbox: {
    id: "auth.welcome.checkbox.subscribe",
    defaultMessage: "Subscribe to updates about Modrinth",
  },
  tosLabel: {
    id: "auth.welcome.label.tos",
    defaultMessage:
      "By creating an account, you have agreed to Modrinth's <terms-link>Terms</terms-link> and <privacy-policy-link>Privacy Policy</privacy-policy-link>.",
  },
  welcomeDescription: {
    id: "auth.welcome.description",
    defaultMessage:
      "You’re now part of the awesome community of creators & explorers already building, downloading, and staying up-to-date with awazing mods.",
  },
  welcomeLongTitle: {
    id: "auth.welcome.long-title",
    defaultMessage: "Welcome to Modrinth!",
  },
  welcomeTitle: {
    id: "auth.welcome.title",
    defaultMessage: "Welcome",
  },
});

useHead({
  title: () => `${formatMessage(messages.welcomeTitle)} - Modrinth`,
});

const subscribe = ref(true);

onMounted(async () => {
  await useAuth(route.query.authToken);
  await useUser();
});

async function continueSignUp() {
  if (subscribe.value) {
    try {
      await useBaseFetch("auth/email/subscribe", {
        method: "POST",
      });
    } catch {}
  }

  if (route.query.redirect) {
    await navigateTo(route.query.redirect);
  } else {
    await navigateTo("/dashboard");
  }
}
</script>

<style lang="scss" scoped>
.welcome-box {
  background-color: var(--color-raised-bg);
  border-radius: var(--size-rounded-lg);
  padding: 1.75rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  box-shadow: var(--shadow-card);
  position: relative;

  &.has-bot {
    margin-block: 120px;
  }

  p {
    margin: 0;
  }
  a {
    color: var(--color-brand);
    font-weight: var(--weight-bold);
    &:hover,
    &:focus {
      filter: brightness(1.125);
      text-decoration: underline;
    }
  }

  &__waving-bot {
    --bot-height: 112px;
    position: absolute;
    top: calc(-1 * var(--bot-height));
    right: 5rem;
    height: var(--bot-height);
    width: auto;

    @media (max-width: 768px) {
      --bot-height: 70px;
      right: 2rem;
    }
  }

  &__top-glow {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 1px;
    opacity: 0.4;
    background: linear-gradient(
      to right,
      transparent 2rem,
      var(--color-green) calc(100% - 13rem),
      var(--color-green) calc(100% - 5rem),
      transparent calc(100% - 2rem)
    );
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  &__title {
    font-size: var(--text-32);
    font-weight: var(--weight-extrabold);
    margin: 0;
  }

  &__subtitle {
    font-size: var(--text-18);
  }

  .tos-text {
    font-size: var(--text-14);
    line-height: 1.5;
  }
}
</style>
