<template>
  <div class="content">
    <h2>My projects</h2>

    <div class="section-header">
      <h3>Mods</h3>
      <nuxt-link class="create-button" to="/mod/create"
        >Create a new mod</nuxt-link
      >
    </div>
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Role</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Last updated</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="mod in mods" :key="mod.id">
          <td>
            <img class="rounded-md" :src="mod.icon_url" />
          </td>
          <td>{{ mod.title }}</td>
          <td>Owner</td>
          <td>
            <span v-if="mod.status === 'approved'" class="badge green">
              Approved
            </span>
            <span v-if="mod.status === 'rejected'" class="badge red">
              Rejected
            </span>
            <span v-if="mod.status === 'draft'" class="badge yellow">
              Draft
            </span>
            <span v-if="mod.status === 'processing'" class="badge yellow">
              Processing
            </span>
            <span v-if="mod.status === 'unlisted'" class="badge gray">
              Unlisted
            </span>
            <span v-if="mod.status === 'unknown'" class="badge gray">
              Unknown
            </span>
          </td>
          <td>{{ mod.downloads }}</td>
          <td>{{ $dayjs(mod.published).format('YYYY-MM-DD') }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import axios from 'axios'

export default {
  async fetch() {
    try {
      let res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${this.$auth.user.id}/mods`
      )

      if (res.data) {
        res = await axios.get(
          `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`
        )
        this.mods = res.data
      }
    } catch (err) {}
  },
  data() {
    return {
      mods: [],
    }
  },
}
</script>

<style lang="scss">
.section-header {
  display: flex;
  margin-bottom: 1rem;

  & > * {
    margin-right: 1rem;
  }
}

.create-button {
  margin: auto 0;
  padding: 4px 20px;
  border-radius: 5px;
  color: var(--color-grey-5);
  background-color: var(--color-grey-1);
}

table {
  background: var(--color-bg);
  border-collapse: collapse;
  border-radius: 0.5rem;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  table-layout: fixed;
  width: 100%;

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-grey-2);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 5%;
    }

    &:nth-child(2) {
      padding-left: 0;
      width: 30%;
    }
  }

  th {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 1rem 1rem;
    text-transform: uppercase;
  }

  td {
    padding: 0.25rem 1rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}
</style>
