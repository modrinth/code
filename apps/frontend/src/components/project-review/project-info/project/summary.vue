<template>
	<p class="m-0 whitespace-pre-wrap break-words leading-[1.5]">
		{{ project?.summary }}
	</p>
</template>

<script setup lang="ts">
import { injectProjectReviewPageContext } from '~/providers/project-review'

const { project } = injectProjectReviewPageContext()
</script>
