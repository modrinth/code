<script setup lang="ts">
import { getMarginTarget } from '@modrinth/moderation'
import { commonProjectTypeCategoryMessages, NavTabs, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

const flags = useFeatureFlags()
const route = useRoute()
const modSettings = useModerationSettings()

const isServerSetup = computed(
	() => !!route.query.sid && ['onboarding', 'reset-server'].includes(String(route.query.from)),
)
const allowTabChanging = computed(() => !route.query.sid || isServerSetup.value)
const marginTarget = computed(() => getMarginTarget(modSettings.value))

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
const projectTypeLinks = computed(() => {
	const query = new URLSearchParams()
	for (const key of ['sid', 'wid', 'from']) {
		const value = route.query[key]
		if (typeof value === 'string') query.set(key, value)
	}
	return selectableProjectTypes
		.filter(
			(type) =>
				!isServerSetup.value || ['mods', 'plugins', 'modpacks', 'datapacks'].includes(type.type),
		)
		.map((type) => ({
			...type,
			href: isServerSetup.value ? `${type.href}?${query.toString()}` : type.href,
		}))
})
</script>
<template>
	<div
		class="box-border flex w-full max-w-[1280px] flex-col gap-4 px-6 pb-6"
		:class="`m${marginTarget}-auto`"
	>
		<NavTabs
			v-if="(!flags.projectTypesPrimaryNav || isServerSetup) && allowTabChanging"
			:links="projectTypeLinks"
			replace
			:class="isServerSetup ? 'flex' : 'hidden md:flex'"
		/>
		<NuxtPage />
	</div>
</template>
