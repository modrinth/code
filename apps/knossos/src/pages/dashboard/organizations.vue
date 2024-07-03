<template>
  <div>
    <OrganizationCreateModal ref="createOrgModal" />
    <section class="universal-card">
      <div class="header__row">
        <h2 class="header__title">Organizations</h2>
        <div class="input-group">
          <button class="iconified-button brand-button" @click="openCreateOrgModal">
            <PlusIcon />
            Create organization
          </button>
        </div>
      </div>
      <template v-if="orgs?.length > 0">
        <div class="orgs-grid">
          <nuxt-link
            v-for="org in orgs"
            :key="org.id"
            :to="`/organization/${org.slug}`"
            class="universal-card button-base recessed org"
            :class="{ 'is-disabled': onlyAcceptedMembers(org.members).length === 0 }"
          >
            <Avatar :src="org.icon_url" :alt="org.name" class="icon" />
            <div class="details">
              <div class="title">
                {{ org.name }}
              </div>
              <div class="description">
                {{ org.description }}
              </div>
              <span class="stat-bar">
                <div class="stats">
                  <UsersIcon />
                  <span>
                    {{ onlyAcceptedMembers(org.members).length }} member<template
                      v-if="onlyAcceptedMembers(org.members).length !== 1"
                      >s</template
                    >
                  </span>
                </div>
              </span>
            </div>
          </nuxt-link>
        </div>
      </template>
      <template v-else> Make an organization! </template>
    </section>
  </div>
</template>

<script setup>
import { PlusIcon, Avatar, UsersIcon } from 'omorphia'

import { useAuth } from '~/composables/auth.js'
import OrganizationCreateModal from '~/components/ui/OrganizationCreateModal.vue'

const createOrgModal = ref(null)

const auth = await useAuth()
const uid = computed(() => auth.value.user?.id || null)

const { data: orgs, error } = useAsyncData('organizations', () => {
  if (!uid.value) return Promise.resolve(null)

  return useBaseFetch('user/' + uid.value + '/organizations', {
    apiVersion: 3,
  })
})

const onlyAcceptedMembers = (members) => members.filter((member) => member?.accepted)

if (error.value) {
  createError({
    statusCode: 500,
    message: 'Failed to fetch organizations',
  })
}

const openCreateOrgModal = () => {
  createOrgModal.value?.show()
}
</script>

<style scoped lang="scss">
.project-meta-item {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  padding: var(--spacing-card-sm);

  .project-title {
    margin-bottom: var(--spacing-card-sm);
  }
}

.orgs-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);

  @media screen and (max-width: 750px) {
    grid-template-columns: repeat(1, 1fr);
  }

  gap: var(--gap-md);

  .org {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--gap-md);
    margin-bottom: 0;

    .icon {
      width: 100% !important;
      height: min(6rem, 20vw) !important;
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

.grid-table {
  display: grid;
  grid-template-columns:
    min-content min-content minmax(min-content, 2fr)
    minmax(min-content, 1fr) minmax(min-content, 1fr) min-content;
  border-radius: var(--size-rounded-sm);
  overflow: hidden;
  margin-top: var(--spacing-card-md);

  .grid-table__row {
    display: contents;

    > div {
      display: flex;
      flex-direction: column;
      justify-content: center;
      height: 100%;
      padding: var(--spacing-card-sm);

      // Left edge of table
      &:first-child {
        padding-left: var(--spacing-card-bg);
      }

      // Right edge of table
      &:last-child {
        padding-right: var(--spacing-card-bg);
      }
    }

    &:nth-child(2n + 1) > div {
      background-color: var(--color-table-alternate-row);
    }

    &.grid-table__header > div {
      background-color: var(--color-bg);
      font-weight: bold;
      color: var(--color-text-dark);
      padding-top: var(--spacing-card-bg);
      padding-bottom: var(--spacing-card-bg);
    }
  }
}

.hover-link:hover {
  text-decoration: underline;
}
</style>
