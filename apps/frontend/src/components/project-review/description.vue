<template>
	<div v-if="project" class="markdown-body" v-html="renderString(project.description)" />
	<p v-else>{{ formatMessage(projectReviewMessages.empty) }}</p>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages } from './messages'

const { formatMessage } = useVIntl()
const { project } = injectProjectReviewPageContext()
</script>
