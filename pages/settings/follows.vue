<template>
  <div v-if="$user.follows.length > 0">
    <ProjectCard
      v-for="project in $user.follows"
      :id="project.id"
      :key="project.id"
      :type="project.project_type"
      :categories="project.categories"
      :created-at="project.published"
      :updated-at="project.updated"
      :description="project.description"
      :downloads="project.downloads ? project.downloads.toString() : '0'"
      :icon-url="project.icon_url"
      :name="project.title"
      :client-side="project.client_side"
      :server-side="project.server_side"
    >
      <button
        class="iconified-button"
        @click="$store.dispatch('user/unfollowProject', project)"
      >
        <HeartIcon />
        Unfollow
      </button>
    </ProjectCard>
  </div>
  <div v-else class="error">
    <FollowIllustration class="icon" />
    <br />
    <span class="text"
      >You don't have any followed projects. <br />
      Why don't you <nuxt-link class="link" to="/mods">search</nuxt-link> for
      new ones?</span
    >
  </div>
</template>

<script>
import ProjectCard from '~/components/ui/ProjectCard'

import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import FollowIllustration from '~/assets/images/illustrations/follow_illustration.svg?inline'

export default {
  components: {
    ProjectCard,
    HeartIcon,
    FollowIllustration,
  },
  async fetch() {
    await this.$store.dispatch('user/fetchFollows')
  },
  head: {
    title: 'Followed projects - Modrinth',
  },
}
</script>
<style lang="scss" scoped></style>
