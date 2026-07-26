<script setup lang="ts">
import { commonProjectTypeCategoryMessages, NavTabs, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

const flags = useFeatureFlags()
const route = useRoute()

const allowTabChanging = computed(() => !route.query.sid)

const selectableProjectTypes = [
	{
		label: formatMessage(commonProjectTypeCategoryMessages.mod),
		href: `/discover/mods`,
		type: 'mods',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.resourcepack),
		href: `/discover/resourcepacks`,
		type: 'resourcepacks',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.datapack),
		href: `/discover/datapacks`,
		type: 'datapacks',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.shader),
		href: `/discover/shaders`,
		type: 'shaders',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.modpack),
		href: `/discover/modpacks`,
		type: 'modpacks',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.plugin),
		href: `/discover/plugins`,
		type: 'plugins',
	},
	{
		label: formatMessage(commonProjectTypeCategoryMessages.server),
		href: `/discover/servers`,
		type: 'servers',
	},
]
</script>
<template>
	<div class="mx-auto box-border flex w-full max-w-[1280px] flex-col gap-4 px-6 pb-6">
		<NavTabs
			v-if="!flags.projectTypesPrimaryNav && allowTabChanging"
			:links="selectableProjectTypes"
			replace
			class="hidden md:flex"
		/>
		<NuxtPage />
	</div>
</template>
