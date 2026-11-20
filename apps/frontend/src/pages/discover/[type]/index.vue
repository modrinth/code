<script setup lang="ts">
import type { ServerSetupModalHandle } from '@modrinth/ui'
import {
	BrowseInstallHeader,
	BrowsePageLayout,
	BrowseSidebar,
	commonMessages,
	CreationFlowModal,
	defineMessages,
	formatProjectTypeSentence,
	ToolbarTabs,
	useStickyObserver,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import { useDiscoverBrowse } from '~/composables/use-discover-browse'
import { useDiscoverProjectTypeLinks } from '~/composables/use-discover-project-type-links'
import { useServerInstallContentPrefetch } from '~/composables/use-server-install-content'

definePageMeta({
	key: 'discover-type-index',
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	seoTitle: {
		id: 'discover.seo.title',
		defaultMessage: 'Search {projectType}',
	},
	seoTitleWithQuery: {
		id: 'discover.seo.title-with-query',
		defaultMessage: 'Search {projectType} | {query}',
	},
	projectTypes: {
		id: 'discover.project-types',
		defaultMessage: 'Project types',
	},
	seoDescription: {
		id: 'discover.seo.description',
		defaultMessage:
			'Search and browse thousands of Minecraft {projectType} on Modrinth with instant, accurate search results. Our filters help you quickly find the best Minecraft {projectType}.',
	},
})

const prefetchServerInstallContent = useServerInstallContentPrefetch()
const cosmetics = useCosmetics()
const flags = useFeatureFlags()
const auth = await useAuth()
const route = useRoute()
const filtersMenuOpen = ref(false)
const setupModal = ref<ServerSetupModalHandle | null>(null)
await prefetchServerInstallContent()
const { view, projectType, serverId, serverInstall, target } = useDiscoverBrowse({
	setupModal,
	filtersMenuOpen,
})

const { projectTypeLinks, isServerContext } = useDiscoverProjectTypeLinks(
	computed(() => target.value?.loader),
)
const projectTypeTabs = computed(() =>
	projectTypeLinks.value.map((link) => ({ value: link.type, label: link.label, to: link.href })),
)

await view.ready()

const projectTypeLabel = computed(() =>
	formatProjectTypeSentence(formatMessage, projectType.value, 2),
)
const ogTitle = computed(() =>
	view.query.value
		? formatMessage(messages.seoTitleWithQuery, {
				projectType: projectTypeLabel.value,
				query: view.query.value,
			})
		: formatMessage(messages.seoTitle, { projectType: projectTypeLabel.value }),
)
const description = computed(() =>
	formatMessage(messages.seoDescription, { projectType: projectTypeLabel.value }),
)

useSeoMeta({
	description,
	ogTitle,
	ogDescription: description,
})

const stickyInstallHeaderRef = ref<HTMLElement | null>(null)
const { isStuck: isInstallHeaderStuck } = useStickyObserver(
	stickyInstallHeaderRef,
	'DiscoverInstallHeader',
)
</script>
<template>
	<Teleport v-if="flags.searchBackground" to="#absolute-background-teleport">
		<div class="search-background"></div>
	</Teleport>

	<div
		v-if="target"
		ref="stickyInstallHeaderRef"
		class="sticky top-0 z-20 -mx-6 border-0 border-solid border-divider bg-surface-1 px-6 pt-4"
		:class="[isInstallHeaderStuck ? 'border-t' : '']"
	>
		<BrowseInstallHeader
			:target="target"
			:selection="serverInstall.selection"
			divider
			bottom-padding
		/>
		<div
			aria-hidden="true"
			class="pointer-events-none absolute inset-x-0 top-full h-6 bg-gradient-to-b from-surface-1 to-transparent transition-opacity duration-200 motion-reduce:transition-none"
			:class="isInstallHeaderStuck ? 'opacity-100' : 'opacity-0'"
		/>
	</div>

	<ToolbarTabs
		v-if="isServerContext"
		:model-value="String(route.params.type)"
		:tabs="projectTypeTabs"
		:label="formatMessage(messages.projectTypes)"
		replace
		class="w-fit max-w-full"
	/>

	<div
		class="grid min-w-0 gap-3"
		:class="
			cosmetics.rightSearchLayout
				? 'lg:grid-cols-[minmax(0,1fr)_18.75rem]'
				: 'lg:grid-cols-[18.75rem_minmax(0,1fr)]'
		"
	>
		<section
			class="flex min-w-0 flex-col gap-2"
			:class="[
				{ 'mt-6 sm:mt-0': !target },
				cosmetics.rightSearchLayout ? 'lg:order-1' : 'lg:order-2',
			]"
		>
			<BrowsePageLayout :stacked="!isServerContext" />
		</section>

		<aside
			class="min-w-0"
			:class="cosmetics.rightSearchLayout ? 'lg:order-2' : 'lg:order-1'"
			:aria-label="formatMessage(commonMessages.filtersLabel)"
		>
			<BrowseSidebar>
				<template #prepend>
					<AdPlaceholder v-if="!auth.user && !target" />
				</template>
			</BrowseSidebar>
		</aside>
	</div>

	<CreationFlowModal
		v-if="serverId && serverInstall.from.value !== 'onboarding'"
		ref="setupModal"
		:type="serverInstall.from.value === 'reset-server' ? 'reset-server' : 'server-onboarding'"
		:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
		:show-snapshot-toggle="true"
		:on-back="serverInstall.onFlowBack"
		@hide="serverInstall.onFlowHide"
		@browse-modpacks="() => {}"
		@create="serverInstall.onFlowCreate"
	/>
</template>
<style lang="scss" scoped>
.search-background {
	width: 100%;
	height: 20rem;
	background-image: url('https://minecraft.wiki/images/Tiny_Takeover_Key_Art.png?025dc');
	background-size: cover;
	background-position: 50% 25%;
	pointer-events: none;
	mask-image: linear-gradient(to bottom, black, transparent);
	opacity: 0.25;
}
</style>
