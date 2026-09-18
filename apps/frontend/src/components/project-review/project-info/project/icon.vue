<template>
	<Avatar v-if="project" :src="project.icon_url" :alt="project.name" size="5rem" no-shadow />
</template>

<script setup lang="ts">
import { Avatar } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

const { project } = injectProjectReviewPageContext()
</script>
