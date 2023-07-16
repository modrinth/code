<template>
  <div v-if="user.follows.length > 0" class="project-list display-mode--list">
    <ProjectCard
      v-for="project in user.follows"
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
      :color="project.color"
    >
      <button class="iconified-button" @click="userUnfollowProject(project)">
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
      Why don't you <nuxt-link class="link" to="/mods">search</nuxt-link> for new ones?</span
    >
  </div>
</template>

<script setup>
import ProjectCard from '~/components/ui/ProjectCard.vue'

import HeartIcon from 'assets/images/utils/heart.svg'
import FollowIllustration from 'assets/images/illustrations/follow_illustration.svg'

const user = await useUser()
if (process.client) {
  await initUserFollows()
}

useHead({ title: 'Followed review - Modrinth' })
definePageMeta({
  middleware: 'auth',
})
</script>
<style lang="scss" scoped></style>
