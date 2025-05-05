<template>
    <div class="welcome-box has-bot">
      <img
        src="https://cdn-raw.modrinth.com/sad-bot.webp"
        alt="Waving Modrinth Bot"
        class="welcome-box__waving-bot"
      />
      <div class="welcome-box__top-glow" />
      <div class="welcome-box__body">
        <h1 class="welcome-box__title">
          {{ formatMessage(messages.welcomeLongTitle) }}
        </h1>

        <p class="welcome-box__subtitle">
          <IntlFormatted
            :message-id="messages.welcomeDescription"
            :values="{ userCount: formatNumber(stats.authors, false) }"
          >
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

        <button
          class="btn btn-primary continue-btn centered-btn"
          @click="continueSignUp"
        >
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
import { RightArrowIcon } from "@modrinth/assets";
import { formatNumber } from "~/plugins/shorthands.js";

const route = useRoute();

const { formatMessage } = useVIntl();
const { data: stats } = await useAsyncData("statistics", () => useBaseFetch("statistics"));

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
      "You’re now part of the <bold>{userCount}</bold> member community of creators & explorers already building, downloading, and staying up-to-date with the most awesome mods. Welcome!",
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
})

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
.main {
  margin: var(--spacing-card-lg) auto;
  width: calc(100% - 4rem);
  min-height: min(90vh, 30rem);
  display: flex;
  flex-direction: column;
  justify-content: center;
  @media screen and (min-width: 800px) {
    width: 600px;
  }
}

.welcome-box {
  background-color: var(--color-raised-bg);
  border-radius: 1.25rem;
  padding: 1.75rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  box-shadow: var(--shadow-card);
  position: relative;
}

.welcome-box.has-bot {
  margin-block: 120px;
}

.welcome-box p {
  margin: 0;
}

.welcome-box a {
  color: var(--color-brand);
  font-weight: 600;
}

.welcome-box a:hover,
.welcome-box a:focus {
  filter: brightness(1.125);
  text-decoration: underline;
}

.welcome-box__waving-bot {
  --_bot-height: 112px;
  position: absolute;
  top: calc(-1 * var(--_bot-height));
  right: 5rem;
  width: auto;
  height: var(--_bot-height);
}

.welcome-box__top-glow {
  width: 100%;
  height: 1px;
  background: linear-gradient(
      to right,
      transparent 2rem,
      var(--color-green) calc(100% - 13rem),
      var(--color-green) calc(100% - 5rem),
      transparent calc(100% - 2rem)
  );
  position: absolute;
  top: 0;
  left: 0;
  opacity: 0.4;
}

.welcome-box__title {
  font-size: 2rem;
  font-weight: 900;
  margin: 0;
}

.welcome-box__subtitle {
  font-size: 1.1rem;
}

.welcome-box__body {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.tos-text {
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
