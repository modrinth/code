<template>
  <NuxtLayout>
    <div class="main experimental-styles-within">
      <div v-if="is404" class="error-graphic">
        <Logo404 />
      </div>
      <div class="error-box" :class="{ 'has-bot': !is404 }">
        <img v-if="!is404" :src="SadRinthbot" alt="Sad Modrinth bot" class="error-box__sad-bot" />
        <div v-if="!is404" class="error-box__top-glow" />
        <div class="error-box__body">
          <h1 class="error-box__title">{{ formatMessage(errorMessages.title) }}</h1>
          <p v-if="errorMessages.subtitle" class="error-box__subtitle">
            {{ formatMessage(errorMessages.subtitle) }}
          </p>
        </div>
        <div class="error-box__body">
          <p v-if="errorMessages.list_title" class="error-box__list-title">
            {{ formatMessage(errorMessages.list_title) }}
          </p>
          <ul v-if="errorMessages.list_items" class="error-box__list">
            <li v-for="item in errorMessages.list_items" :key="item">
              <IntlFormatted :message-id="item">
                <template #status-link="{ children }">
                  <a href="https://status.modrinth.com" target="_blank" rel="noopener">
                    <component :is="() => children" />
                  </a>
                </template>
                <template #discord-link="{ children }">
                  <a href="https://discord.modrinth.com" target="_blank" rel="noopener">
                    <component :is="() => children" />
                  </a>
                </template>
                <template #tou-link="{ children }">
                  <nuxt-link :to="`/legal/terms`" target="_blank" rel="noopener">
                    <component :is="() => children" />
                  </nuxt-link>
                </template>
              </IntlFormatted>
            </li>
          </ul>
        </div>
        <div v-if="!is404" class="error-box__details">
          <p>Error {{ error.statusCode }}</p>
          <p>{{ error.message }}</p>
        </div>
      </div>
    </div>
  </NuxtLayout>
</template>

<script setup>
import { defineMessage, useVIntl } from "@vintl/vintl";
import { SadRinthbot } from "@modrinth/assets";
import Logo404 from "~/assets/images/404.svg";

const { formatMessage } = useVIntl();

const props = defineProps({
  error: {
    type: Object,
    default() {
      return {
        statusCode: 1000,
        message: "Unknown error",
      };
    },
  },
});

const is404 = computed(() => props.error.statusCode === 404);
const errorMessages = computed(
  () =>
    routeMessages.find((x) => x.match(route))?.messages[props.error.statusCode] ??
    messages[props.error.statusCode] ??
    messages.default,
);

const route = useRoute();

watch(route, () => {
  console.log(route);
});

const messages = {
  404: {
    title: defineMessage({
      id: "error.generic.404.title",
      defaultMessage: "Page not found",
    }),
    subtitle: defineMessage({
      id: "error.generic.404.subtitle",
      defaultMessage: "The page you were looking for doesn't seem to exist.",
    }),
  },
  default: {
    title: defineMessage({
      id: "error.generic.default.title",
      defaultMessage: "Uh oh!",
    }),
    subtitle: defineMessage({
      id: "error.generic.default.subtitle",
      defaultMessage: "Something went wrong.",
    }),
    list_title: defineMessage({
      id: "error.generic.default.list_title",
      defaultMessage: "Please try again in a few minutes.",
    }),
    list_items: [
      defineMessage({
        id: "error.generic.default.list_item.1",
        defaultMessage: "Check if Modrinth is down on our <status-link>Status page</status-link>.",
      }),
      defineMessage({
        id: "error.generic.default.list_item.2",
        defaultMessage:
          "If this keeps happening, you may want to let the Modrinth Team know by joining our <discord-link>Discord server</discord-link>.",
      }),
    ],
  },
};

const PROJECT_PATH_PREFIXES = [
  "/mod/",
  "/datapack/",
  "/resourcepack/",
  "/plugin/",
  "/shader/",
  "/modpack/",
  "/project/",
];

const routeMessages = [
  {
    match: (route) => PROJECT_PATH_PREFIXES.some((prefix) => route.path.startsWith(prefix)),
    messages: {
      404: {
        title: defineMessage({
          id: "error.project.404.title",
          defaultMessage: "Project not found",
        }),
        list_title: defineMessage({
          id: "error.project.404.list_title",
          defaultMessage: "Why?",
        }),
        list_items: [
          defineMessage({
            id: "error.project.404.list_item.1",
            defaultMessage: "You may have mistyped the project's URL.",
          }),
          defineMessage({
            id: "error.project.404.list_item.2",
            defaultMessage:
              "The project's owner may have changed the URL, made the project private, or deleted it.",
          }),
          defineMessage({
            id: "error.project.404.list_item.3",
            defaultMessage:
              "The project may have been taken down by Modrinth's moderation team for violating our <tou-link>Terms of Use</tou-link>.",
          }),
        ],
      },
    },
  },
  {
    match: (route) => route.path.startsWith("/user/"),
    messages: {
      404: {
        title: defineMessage({
          id: "error.user.404.title",
          defaultMessage: "User not found",
        }),
        list_title: defineMessage({
          id: "error.user.404.list_title",
          defaultMessage: "Why?",
        }),
        list_items: [
          defineMessage({
            id: "error.user.404.list_item.1",
            defaultMessage: "You may have mistyped the user's username.",
          }),
          defineMessage({
            id: "error.user.404.list_item.2",
            defaultMessage: "The user may have changed their username or deleted their account.",
          }),
          defineMessage({
            id: "error.user.404.list_item.3",
            defaultMessage:
              "The user's account may have been terminated for violating Modrinth's <tou-link>Terms of Use</tou-link>.",
          }),
        ],
      },
    },
  },
  {
    match: (route) => route.path.startsWith("/organization/"),
    messages: {
      404: {
        title: defineMessage({
          id: "error.organization.404.title",
          defaultMessage: "Organization not found",
        }),
        list_title: defineMessage({
          id: "error.organization.404.list_title",
          defaultMessage: "Why?",
        }),
        list_items: [
          defineMessage({
            id: "error.organization.404.list_item.1",
            defaultMessage: "You may have mistyped the organization's URL.",
          }),
          defineMessage({
            id: "error.organization.404.list_item.2",
            defaultMessage: "The organization's owner may have changed the URL or deleted it.",
          }),
          defineMessage({
            id: "error.organization.404.list_item.3",
            defaultMessage:
              "The organization may have been removed by Modrinth's moderation team for violating our <tou-link>Terms of Use</tou-link>.",
          }),
        ],
      },
    },
  },
  {
    match: (route) => route.path.startsWith("/collection/"),
    messages: {
      404: {
        title: defineMessage({
          id: "error.collection.404.title",
          defaultMessage: "Collection not found",
        }),
        list_title: defineMessage({
          id: "error.collection.404.list_title",
          defaultMessage: "Why?",
        }),
        list_items: [
          defineMessage({
            id: "error.collection.404.list_item.1",
            defaultMessage: "You may have mistyped the collection's URL.",
          }),
          defineMessage({
            id: "error.collection.404.list_item.2",
            defaultMessage: "The collection may be private.",
          }),
          defineMessage({
            id: "error.collection.404.list_item.3",
            defaultMessage:
              "The collection may have been taken down by Modrinth's moderation team for violating our <tou-link>Terms of Use</tou-link>.",
          }),
        ],
      },
    },
  },
];
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

.error-graphic {
  margin-bottom: 2rem;
  display: flex;
  justify-content: center;

  svg {
    fill: var(--color-text);
    color: var(--color-text);
    width: min(15rem, 100%);
    height: auto;
  }
}

.error-box {
  background-color: var(--color-raised-bg);
  border-radius: 1.25rem;
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
    font-weight: 600;

    &:hover,
    &:focus {
      filter: brightness(1.125);
      text-decoration: underline;
    }
  }

  &__sad-bot {
    --_bot-height: 112px;
    position: absolute;
    top: calc(-1 * var(--_bot-height));
    right: 5rem;
    width: auto;
    height: var(--_bot-height);

    @media screen and (max-width: 768px) {
      --_bot-height: 70px;
      right: 2rem;
    }
  }

  &__top-glow {
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

  &__title {
    font-size: 2rem;
    font-weight: 900;
    margin: 0;
  }

  &__subtitle {
    font-size: 1.25rem;
    font-weight: 600;
  }

  &__body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  &__list-title {
    font-weight: 600;
  }

  &__list {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-left: 1.25rem;
  }

  li {
    line-height: 1.5;
  }

  &__details {
    display: flex;
    flex-direction: column;
    color: var(--color-secondary);
    gap: 0.25rem;
    font-weight: 500;
    font-size: 0.875rem;
  }
}
</style>
