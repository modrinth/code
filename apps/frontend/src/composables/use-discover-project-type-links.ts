import {
	commonProjectTypeCategoryMessages,
	isServerContentEnvironmentProjectType,
	parseServerContentEnvironment,
	useVIntl,
} from '@modrinth/ui'
import type { Ref } from 'vue'

const PLUGIN_LOADERS = ['paper', 'purpur']

/** Project type tabs for the discover pages, filtered to what the current server context allows. */
export function useDiscoverProjectTypeLinks(
	serverLoader?: Readonly<Ref<string | null | undefined>>,
) {
	const { formatMessage } = useVIntl()
	const route = useRoute()

	const isServerContext = computed(() => !!route.query.sid)
	const isServerSetup = computed(
		() =>
			isServerContext.value && ['onboarding', 'reset-server'].includes(String(route.query.from)),
	)
	const serverEnvironment = computed(() => parseServerContentEnvironment(route.query.env))
	const serverSupportsPlugins = computed(() => {
		const loader = serverLoader?.value?.toLowerCase()
		return !!loader && PLUGIN_LOADERS.includes(loader)
	})

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
		for (const key of ['sid', 'wid', 'from', 'env']) {
			const value = route.query[key]
			if (typeof value === 'string') query.set(key, value)
		}
		return selectableProjectTypes
			.filter((type) => {
				if (isServerSetup.value) {
					return ['mods', 'plugins', 'modpacks', 'datapacks'].includes(type.type)
				}
				if (!isServerContext.value) return true
				if (type.type === 'plugins' && !serverSupportsPlugins.value) return false
				return (
					!['servers', 'modpacks'].includes(type.type) &&
					isServerContentEnvironmentProjectType(serverEnvironment.value, type.type.slice(0, -1))
				)
			})
			.map((type) => ({
				...type,
				href: query.size > 0 ? `${type.href}?${query.toString()}` : type.href,
			}))
	})

	return { projectTypeLinks, isServerContext, isServerSetup }
}
