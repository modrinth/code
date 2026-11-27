import { CompassIcon, ServerStackIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, formatProjectTypeSentence, useVIntl } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, ref, shallowRef, watch } from 'vue'
import { useRouter } from 'vue-router'

import { getInstanceIconUrl } from '@/helpers/instance'
import { type BreadcrumbDefinition, injectBreadcrumbManager } from '@/shared/breadcrumbs'
import { useCachedServerIcon } from '@/shared/composables/use-cached-server-icon'

const messages = defineMessages({
	discoverProjectType: {
		id: 'app.browse.discover-project-type',
		defaultMessage: 'Discover {projectType}',
	},
	discoverServers: {
		id: 'app.browse.discover-servers',
		defaultMessage: 'Discover servers',
	},
	hosting: {
		id: 'app.browse.breadcrumb.hosting',
		defaultMessage: 'Hosting',
	},
})

export interface BrowseBreadcrumbOptions {
	instance: Readonly<Ref<{ name: string; icon_path?: string | null } | null>>
	serverName: Readonly<Ref<string | undefined>>
	serverBackUrl: Readonly<Ref<string>>
}

export function useBrowseBreadcrumbs({
	instance,
	serverName,
	serverBackUrl,
}: BrowseBreadcrumbOptions) {
	const { formatMessage } = useVIntl()
	const router = useRouter()
	const breadcrumbManager = injectBreadcrumbManager()

	const displayedRoute = shallowRef(router.currentRoute.value)
	watch(
		() => router.currentRoute.value,
		(next) => {
			if (next.path.startsWith('/browse/')) displayedRoute.value = next
		},
		{ immediate: true },
	)

	const query = (key: string) => String(displayedRoute.value.query[key] ?? '')

	const label = computed(() => {
		if (
			displayedRoute.value.query.from === 'worlds' ||
			displayedRoute.value.params.projectType === 'server'
		) {
			return formatMessage(messages.discoverServers)
		}
		return formatMessage(messages.discoverProjectType, {
			projectType: formatProjectTypeSentence(
				formatMessage,
				String(displayedRoute.value.params.projectType ?? ''),
				2,
			),
		})
	})

	const instanceBreadcrumb = {
		slot: 'instance',
		id: () => `instance:${query('i')}`,
		label: () => instance.value?.name ?? formatMessage(commonMessages.loadingLabel),
		visual: () => ({
			type: 'image' as const,
			src: getInstanceIconUrl(instance.value?.icon_path),
			alt: instance.value?.name,
			tintBy: query('i'),
		}),
		to: () => {
			const instancePath = `/instance/${encodeURIComponent(query('i'))}`
			return displayedRoute.value.query.from === 'worlds' ? `${instancePath}/worlds` : instancePath
		},
	} satisfies BreadcrumbDefinition

	const serversBreadcrumb = {
		slot: 'root',
		id: 'servers',
		label: () => formatMessage(messages.hosting),
		to: '/hosting/manage/',
		visual: { type: 'icon', component: ServerStackIcon },
	} satisfies BreadcrumbDefinition

	const serverBreadcrumbTo = ref(serverBackUrl.value)
	watch(serverBackUrl, (value) => {
		if (router.currentRoute.value.path.startsWith('/browse/')) serverBreadcrumbTo.value = value
	})
	const serverIcon = useCachedServerIcon(() => query('sid'))
	const serverBreadcrumb = {
		slot: 'server',
		id: () => `server:${query('sid')}`,
		label: () => serverName.value ?? formatMessage(commonMessages.loadingLabel),
		visual: () => ({
			type: 'image' as const,
			src: serverIcon.value,
			alt: serverName.value,
		}),
		to: serverBreadcrumbTo,
	} satisfies BreadcrumbDefinition

	const browseBreadcrumb = {
		slot: 'browse',
		id: () =>
			`browse:${String(displayedRoute.value.params.projectType ?? '')}:${query('i')}:${query('sid')}:${query('from')}`,
		label,
		to: () => displayedRoute.value.fullPath,
		visual: { type: 'icon', component: CompassIcon },
	} satisfies BreadcrumbDefinition

	function sync() {
		if (displayedRoute.value.query.i) {
			const parent = breadcrumbManager.reset(instanceBreadcrumb)
			breadcrumbManager.push(browseBreadcrumb, { parent })
			return
		}

		if (displayedRoute.value.query.sid) {
			const root = breadcrumbManager.reset(serversBreadcrumb)
			const parent = breadcrumbManager.push(serverBreadcrumb, { parent: root })
			breadcrumbManager.push(browseBreadcrumb, { parent })
			return
		}

		breadcrumbManager.reset(browseBreadcrumb)
	}

	watch(displayedRoute, sync, { immediate: true, flush: 'sync' })
}
