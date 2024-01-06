<template>
  <div class="universal-card">
    <CollectionCreateModal ref="modal_creation" />
    <h2>Collections</h2>
    <div class="search-row">
      <div class="iconified-input">
        <label for="search-input" hidden>Search your collections</label>
        <SearchIcon />
        <input id="search-input" v-model="filterQuery" type="text" />
        <Button v-if="filterQuery" class="r-btn" @click="() => (filterQuery = '')">
          <XIcon />
        </Button>
      </div>
      <Button color="primary" @click="$refs.modal_creation.show()">
        <PlusIcon /> Create new
      </Button>
    </div>
    <div class="collections-grid">
      <nuxt-link
        v-if="'followed projects'.includes(filterQuery)"
        :to="`/collection/following`"
        class="universal-card recessed collection"
      >
        <Avatar src="https://cdn.modrinth.com/follow-collection.png" class="icon" />
        <div class="details">
          <span class="title">Followed projects</span>
          <span class="description">
            Auto-generated collection of all the projects you're following.
          </span>
          <div class="stat-bar">
            <div class="stats"><BoxIcon /> {{ user.follows.length }} projects</div>
            <div class="stats"><LockIcon /> <span> Private </span></div>
          </div>
        </div>
      </nuxt-link>
      <nuxt-link
        v-for="collection in orderedCollections"
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
            <div class="stats"><BoxIcon /> {{ collection.projects?.length || 0 }} projects</div>
            <div class="stats">
              <template v-if="collection.status === 'listed'">
                <WorldIcon />
                <span> Public </span>
              </template>
              <template v-else-if="collection.status === 'unlisted'">
                <LinkIcon />
                <span> Unlisted </span>
              </template>
              <template v-else-if="collection.status === 'private'">
                <LockIcon />
                <span> Private </span>
              </template>
              <template v-else-if="collection.status === 'rejected'">
                <XIcon />
                <span> Rejected </span>
              </template>
            </div>
          </div>
        </div>
      </nuxt-link>
    </div>
  </div>
</template>
<script setup>
import { Avatar, BoxIcon, SearchIcon, XIcon, Button, PlusIcon, LinkIcon, LockIcon } from 'omorphia'
import WorldIcon from '~/assets/images/utils/world.svg'
import CollectionCreateModal from '~/components/ui/CollectionCreateModal.vue'

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'Your collections - Modrinth',
})

const user = await useUser()
const auth = await useAuth()

if (process.client) {
  await initUserFollows()
}

const filterQuery = ref('')

const { data: collections } = await useAsyncData(`user/${auth.value.user.id}/collections`, () =>
  useBaseFetch(`user/${auth.value.user.id}/collections`, { apiVersion: 3 })
)

const orderedCollections = computed(() => {
  if (!collections.value) return []
  return collections.value
    .sort((a, b) => {
      const aUpdated = new Date(a.updated)
      const bUpdated = new Date(b.updated)
      return bUpdated - aUpdated
    })
    .filter((collection) => {
      if (!filterQuery.value) return true
      return collection.name.toLowerCase().includes(filterQuery.value.toLowerCase())
    })
})
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
