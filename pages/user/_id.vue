<template>
  <div class="content">
    <div class="user-profile">
      <img :src="user.avatar_url" :alt="user.username" />
      <div class="info">
        <h1>{{ user.username }}</h1>
        <p class="joined-text">Joined {{ $dayjs(user.created).fromNow() }}</p>
        <p v-if="user.bio" class="bio">{{ user.bio }}</p>
        <p v-if="user.role === 'admin'" class="badge red">Admin</p>
        <p v-if="user.role === 'moderator'" class="badge yellow">Moderator</p>
        <p v-if="user.role === 'developer'" class="badge green">Developer</p>
      </div>
    </div>
    <div class="user-mods">
      <SearchResult
        v-for="(result, index) in mods"
        :id="result.mod_id"
        :key="result.mod_id"
        :name="result.title"
        :description="result.description"
        :latest-version="result.versions[0]"
        :created-at="result.published"
        :updated-at="result.updated"
        :downloads="result.downloads.toString()"
        :icon-url="result.icon_url"
        :author-url="result.author_url"
        :page-url="result.page_url"
        :categories="result.categories"
        :is-ad="index === -1"
      />
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '@/components/ModResult'

export default {
  auth: false,
  components: {
    SearchResult,
  },
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.params.id}`
    )
    const user = res.data

    let mods = []
    res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.params.id}/mods`
    )
    if (res.data) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`
      )
      mods = res.data
    }

    return {
      mods,
      user,
    }
  },
}
</script>

<style lang="scss" scoped>
.user-profile {
  @media screen and (min-width: 900px) {
    display: inline-flex;
    text-align: left;
  }
  text-align: center;
  margin-bottom: 20px;
  margin-left: 15px;

  img {
    border-radius: var(--size-rounded-md);
    width: 250px;
    height: 250px;
  }

  .info {
    @media screen and (min-width: 900px) {
      margin-left: 15px;
    }

    h1 {
      margin-bottom: 0;
    }

    .joined-text {
      margin-top: 5px;
      color: var(--color-grey-4);
    }

    .bio {
      margin-top: 5px;
      font-size: 16pt;
    }

    .badge {
      display: inline-block;
    }
  }
}

.user-mods {
  border-top: 1px solid var(--color-grey-1);
  padding-top: 10px;
  margin: 10px;
  * {
    margin-left: 0;
  }
}
</style>
