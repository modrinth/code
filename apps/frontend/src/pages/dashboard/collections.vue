<template>
  <div class="universal-card">
    <CollectionCreateModal ref="modal_creation" />
    <h2 class="text-2xl">{{ formatMessage(commonMessages.collectionsLabel) }}</h2>
    <div class="search-row">
      <div class="iconified-input">
        <label for="search-input" hidden>{{ formatMessage(messages.searchInputLabel) }}</label>
        <SearchIcon aria-hidden="true" />
        <input id="search-input" v-model="filterQuery" type="text" />
        <Button
          v-if="filterQuery"
          class="r-btn"
          aria-label="Clear search"
          @click="() => (filterQuery = '')"
        >
          <XIcon aria-hidden="true" />
        </Button>
      </div>
      <Button color="primary" @click="(event) => $refs.modal_creation.show(event)">
        <PlusIcon aria-hidden="true" />
        {{ formatMessage(messages.createNewButton) }}
      </Button>
    </div>
    <div class="collections-grid">
      <nuxt-link
        v-if="'followed projects'.includes(filterQuery.toLowerCase())"
        :to="`/collection/following`"
        class="universal-card recessed collection"
      >
        <Avatar src="https://cdn.modrinth.com/follow-collection.png" class="icon" />
        <div class="details">
          <span class="title">{{ formatMessage(commonMessages.followedProjectsLabel) }}</span>
          <span class="description">
            {{ formatMessage(messages.followingCollectionDescription) }}
          </span>
          <div class="stat-bar">
            <div class="stats">
              <BoxIcon aria-hidden="true" />
              {{
                formatMessage(messages.projectsCountLabel, {
                  count: formatCompactNumber(user ? user.follows.length : 0),
                })
              }}
            </div>
            <div class="stats">
              <LockIcon aria-hidden="true" />
              <span> {{ formatMessage(commonMessages.privateLabel) }} </span>
            </div>
          </div>
        </div>
      </nuxt-link>
      <nuxt-link
        v-for="collection in orderedCollections.sort(
          (a, b) => new Date(b.created) - new Date(a.created),
        )"
        :key="collection.id"
        :to="`/collection/${collection.id}`"
        class="universal-card recessed collection"
      >
        <Avatar :src="collection.icon_url" class="icon" />
        <div class="details">
          <span class="title">{{ collection.name }}</span>
          <span class="description">
            {{ collection.description }}
          </span>
          <div class="stat-bar">
            <div class="stats">
              <BoxIcon aria-hidden="true" />
              {{
                formatMessage(messages.projectsCountLabel, {
                  count: formatCompactNumber(collection.projects?.length || 0),
                })
              }}
            </div>
            <div class="stats">
              <template v-if="collection.status === 'listed'">
                <GlobeIcon aria-hidden="true" />
                <span> {{ formatMessage(commonMessages.publicLabel) }} </span>
              </template>
              <template v-else-if="collection.status === 'unlisted'">
                <LinkIcon aria-hidden="true" />
                <span> {{ formatMessage(commonMessages.unlistedLabel) }} </span>
              </template>
              <template v-else-if="collection.status === 'private'">
                <LockIcon aria-hidden="true" />
                <span> {{ formatMessage(commonMessages.privateLabel) }} </span>
              </template>
              <template v-else-if="collection.status === 'rejected'">
                <XIcon aria-hidden="true" />
                <span> {{ formatMessage(commonMessages.rejectedLabel) }} </span>
              </template>
            </div>
          </div>
        </div>
      </nuxt-link>
    </div>
  </div>
</template>
<script setup>
import {
  BoxIcon,
  SearchIcon,
  XIcon,
  PlusIcon,
  LinkIcon,
  LockIcon,
  GlobeIcon,
} from "@modrinth/assets";
import { Avatar, Button, commonMessages } from "@modrinth/ui";
import CollectionCreateModal from "~/components/ui/CollectionCreateModal.vue";

const { formatMessage } = useVIntl();
const formatCompactNumber = useCompactNumber();

const messages = defineMessages({
  createNewButton: {
    id: "dashboard.collections.button.create-new",
    defaultMessage: "Create new",
  },
  collectionsLongTitle: {
    id: "dashboard.collections.long-title",
    defaultMessage: "Your collections",
  },
  followingCollectionDescription: {
    id: "collection.description.following",
    defaultMessage: "Auto-generated collection of all the projects you're following.",
  },
  projectsCountLabel: {
    id: "dashboard.collections.label.projects-count",
    defaultMessage: "{count, plural, one {{count} project} other {{count} projects}}",
  },
  searchInputLabel: {
    id: "dashboard.collections.label.search-input",
    defaultMessage: "Search your collections",
  },
});

definePageMeta({
  middleware: "auth",
});

useHead({
  title: () => `${formatMessage(messages.collectionsLongTitle)} - Modrinth`,
});

const auth = await useAuth();
const user = await useUser();

if (import.meta.client) {
  await initUserFollows();
}

const filterQuery = ref("");

const { data: collections } = await useAsyncData(`user/${auth.value.user.id}/collections`, () =>
  useBaseFetch(`user/${auth.value.user.id}/collections`, { apiVersion: 3 }),
);

const orderedCollections = computed(() => {
  if (!collections.value) return [];
  return collections.value
    .sort((a, b) => {
      const aUpdated = new Date(a.updated);
      const bUpdated = new Date(b.updated);
      return bUpdated - aUpdated;
    })
    .filter((collection) => {
      if (!filterQuery.value) return true;
      return collection.name.toLowerCase().includes(filterQuery.value.toLowerCase());
    });
});
</script>
<style lang="scss">
.collections-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);

  @media screen and (max-width: 800px) {
    grid-template-columns: repeat(1, 1fr);
  }

  gap: var(--gap-md);

  .collection {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--gap-md);
    margin-bottom: 0;

    .icon {
      width: 100% !important;
      height: 6rem !important;
      max-width: unset !important;
      max-height: unset !important;
      aspect-ratio: 1 / 1;
      object-fit: cover;
    }

    .details {
      display: flex;
      flex-direction: column;
      gap: var(--gap-sm);

      .title {
        color: var(--color-contrast);
        font-weight: 600;
        font-size: var(--font-size-md);
      }

      .description {
        color: var(--color-secondary);
        font-size: var(--font-size-sm);
      }

      .stat-bar {
        display: flex;
        align-items: center;
        gap: var(--gap-md);
        margin-top: auto;
      }

      .stats {
        display: flex;
        align-items: center;
        gap: var(--gap-xs);

        svg {
          color: var(--color-secondary);
        }
      }
    }
  }
}

.search-row {
  margin-bottom: var(--gap-lg);
  display: flex;
  align-items: center;
  gap: var(--gap-lg) var(--gap-sm);
  flex-wrap: wrap;
  justify-content: center;

  .iconified-input {
    flex-grow: 1;

    input {
      height: 2rem;
    }
  }
}
</style>
