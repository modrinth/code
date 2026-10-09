<script setup lang="ts">
import { getMarginTarget } from '@modrinth/moderation'
import { NavTabs } from '@modrinth/ui'

import { useDiscoverProjectTypeLinks } from '~/composables/use-discover-project-type-links'

const flags = useFeatureFlags()
const modSettings = useModerationSettings()

const { projectTypeLinks, isServerContext } = useDiscoverProjectTypeLinks()
const marginTarget = computed(() => getMarginTarget(modSettings.value))
</script>
<template>
	<div
		class="box-border flex w-full max-w-[1280px] flex-col gap-4 px-6 pb-6"
		:class="`m${marginTarget}-auto`"
	>
		<NavTabs
			v-if="!flags.projectTypesPrimaryNav && !isServerContext"
			:links="projectTypeLinks"
			replace
			class="hidden md:flex"
		/>
		<NuxtPage />
	</div>
</template>
