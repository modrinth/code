<script setup lang="ts">
import { NavTabs } from '@modrinth/ui'

import { useDiscoverProjectTypeLinks } from '~/composables/use-discover-project-type-links'

const flags = useFeatureFlags()

const { projectTypeLinks, isServerContext } = useDiscoverProjectTypeLinks()
</script>
<template>
	<div class="mx-auto box-border flex w-full max-w-[1280px] flex-col gap-4 px-6 pb-6">
		<NavTabs
			v-if="!flags.projectTypesPrimaryNav && !isServerContext"
			:links="projectTypeLinks"
			replace
			class="hidden md:flex"
		/>
		<NuxtPage />
	</div>
</template>
