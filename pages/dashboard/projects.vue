<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div class="card page-nav">
          <nuxt-link :to="'/dashboard/projects'" class="tab last">
            <ModIcon />
            My mods
          </nuxt-link>
        </div>
        <client-only>
          <EthicalAd type="image" />
        </client-only>
      </div>
      <div class="content">
        <div class="section-header columns">
          <h3 class="column-grow-1">My mods</h3>
          <nuxt-link class="brand-button column" to="/mod/create">
            Create a mod
          </nuxt-link>
        </div>
        <ModCard
          v-for="mod in mods"
          :id="mod.id"
          :key="mod.id"
          :author="mod.author"
          :name="mod.title"
          :description="mod.description"
          :latest-version="mod.latest_version"
          :created-at="mod.published"
          :updated-at="mod.updated"
          :downloads="mod.downloads.toString()"
          :icon-url="mod.icon_url"
          :author-url="mod.author_url"
          :page-url="mod.page_url"
          :categories="mod.categories"
          :edit-mode="true"
          :status="mod.status"
          :is-modrinth="true"
        />
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import EthicalAd from '@/components/EthicalAd'
import ModCard from '@/components/ProjectCard'

import ModIcon from '~/assets/images/sidebar/mod.svg?inline'

export default {
  components: {
    EthicalAd,
    ModCard,
    ModIcon,
  },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local'),
      },
    }

    let res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/mods`,
      config
    )

    res = await axios.get(
      `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`,
      config
    )

    return {
      mods: res.data,
    }
  },
}
</script>

<style lang="scss" scoped>
.section-header {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);
  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }
}

table {
  border-collapse: collapse;
  table-layout: fixed;
  width: 100%;
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-divider);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 7%;
    }

    &:nth-child(2) {
      padding-left: 0;
      width: 30%;
    }
  }

  th {
    color: var(--color-heading);
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 1rem 1rem;
    text-transform: uppercase;
  }

  td {
    padding: 0.75rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}

@media screen and (max-width: 550px) {
  th,
  td {
    &:nth-child(1) {
      img {
        height: 2rem;
        width: 2rem;
      }
    }
    &:nth-child(3) {
      display: none;
    }
  }
}

@media screen and (max-width: 850px) {
  th,
  td {
    &:nth-child(2) {
      width: 25% !important;
    }
    &:nth-child(6) {
      display: none;
    }
  }
}

.mod-name {
  font-weight: bold;
}
</style>
