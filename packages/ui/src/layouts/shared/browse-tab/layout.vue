<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SearchIcon } from '@modrinth/assets'
import { computed, ref, toValue } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import LoadingIndicator from '#ui/components/base/LoadingIndicator.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import SearchFilterControl from '#ui/components/search/SearchFilterControl.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { commonMessages, formatProjectTypeSentence } from '#ui/utils/common-messages'
import type { SortType } from '#ui/utils/search'

import SelectedProjectsFloatingBar from './components/SelectedProjectsFloatingBar.vue'
import BrowseInstallHeader from './header.vue'
import { injectBrowseManager } from './providers/browse-manager'
import BrowseSidebar from './sidebar.vue'

const ctx = injectBrowseManager()
const { formatMessage } = useVIntl()
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))
const stickyInstallHeaderRef = ref<HTMLElement | null>(null)
const { isStuck: isInstallHeaderStuck } = useStickyObserver(
	stickyInstallHeaderRef,
	'BrowseInstallHeader',
)

const sortOptions = computed<ComboboxOption<SortType>[]>(() =>
	ctx.effectiveSortTypes.value.map((st) => ({
		value: st,
		label: st.display,
	})),
)

const maxResultsOptions = computed<ComboboxOption<number>[]>(() =>
	(ctx.maxResultsOptions?.value ?? [5, 10, 15, 20, 50, 100]).map((n) => ({
		value: n,
		label: String(n),
	})),
)

const installHeaderClass = computed(() => [
	'sticky top-0 z-20 border-0 bg-surface-1',
	ctx.variant === 'web'
		? 'normal-page__header browse-install-header-bleed mb-4 flex flex-col gap-2 py-3'
		: '-mx-6 -mt-6 rounded-tl-[--radius-xl] border-b border-solid p-3 border-surface-5',
	ctx.variant !== 'web' && isInstallHeaderStuck.value ? 'border-t' : '',
])
const contentFrameTag = computed(() => (ctx.variant === 'web' ? 'section' : 'div'))
const contentFrameClass = computed(() =>
	ctx.variant === 'web' ? 'normal-page__content browse-page-layout__content' : 'contents',
)
const contentInnerClass = computed(() =>
	ctx.variant === 'web' ? 'flex flex-col gap-3' : 'contents',
)

const messages = defineMessages({
	searchPlaceholder: {
		id: 'browse.search.placeholder',
		defaultMessage: 'Search {projectType}...',
	},
	viewPrefix: {
		id: 'browse.view-prefix',
		defaultMessage: 'View:',
	},
	filterResults: {
		id: 'browse.filter-results',
		defaultMessage: 'Filter results...',
	},
	offline: {
		id: 'browse.offline',
		defaultMessage: 'You are currently offline. Connect to the internet to browse Modrinth!',
	},
	noResults: {
		id: 'browse.no-results',
		defaultMessage: 'No results found for your query!',
	},
})
</script>

<template>
	<div v-if="ctx.installContext?.value" ref="stickyInstallHeaderRef" :class="installHeaderClass">
		<BrowseInstallHeader />
	</div>
	<SelectedProjectsFloatingBar v-if="ctx.installContext?.value" />

	<template v-if="ctx.variant === 'web'">
		<aside
			class="normal-page__sidebar browse-page-layout__sidebar"
			:aria-label="formatMessage(commonMessages.filtersLabel)"
		>
			<slot name="sidebar-prepend" />
			<BrowseSidebar />
		</aside>
	</template>
	<template v-else>
		<Teleport to="#sidebar-teleport-target">
			<BrowseSidebar>
				<template #prepend>
					<slot name="sidebar-prepend" />
				</template>
			</BrowseSidebar>
		</Teleport>
	</template>

	<component :is="contentFrameTag" :class="contentFrameClass">
		<div :class="contentInnerClass">
			<NavTabs v-if="ctx.showProjectTypeTabs.value" :links="ctx.selectableProjectTypes.value" />

			<StyledInput
				v-model="ctx.query.value"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="
					formatMessage(messages.searchPlaceholder, {
						projectType: formatProjectTypeSentence(formatMessage, ctx.projectType.value, 2),
					})
				"
				clearable
				wrapper-class="w-full"
				:input-class="ctx.variant === 'web' ? '!h-12' : 'h-12'"
				@clear="ctx.clearSearch()"
			/>

			<div class="flex flex-wrap items-center gap-2">
				<Combobox
					:model-value="ctx.effectiveCurrentSortType.value"
					:options="sortOptions"
					:class="
						ctx.variant === 'web'
							? '!w-[16rem] min-w-max max-w-full flex-grow md:flex-grow-0'
							: '!w-[16rem] min-w-max max-w-full'
					"
					@update:model-value="(val: SortType) => (ctx.effectiveCurrentSortType.value = val)"
				>
					<template #prefix>
						<span class="font-semibold text-primary">{{
							formatMessage(commonMessages.sortByLabel)
						}}</span>
					</template>
				</Combobox>

				<Combobox
					:model-value="ctx.maxResults.value"
					:options="maxResultsOptions"
					:class="
						ctx.variant === 'web'
							? '!w-[9rem] min-w-max max-w-full flex-grow md:flex-grow-0'
							: '!w-[9rem] min-w-max max-w-full'
					"
					:placeholder="formatMessage(commonMessages.viewLabel)"
					@update:model-value="(val: number) => (ctx.maxResults.value = val)"
				>
					<template #prefix>
						<span class="font-semibold text-primary">{{ formatMessage(messages.viewPrefix) }}</span>
					</template>
				</Combobox>

				<div v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value" class="lg:hidden">
					<ButtonStyled>
						<button @click="ctx.filtersMenuOpen.value = true">
							{{ formatMessage(messages.filterResults) }}
						</button>
					</ButtonStyled>
				</div>

				<ButtonStyled v-if="ctx.cycleDisplayMode" circular>
					<button @click="ctx.cycleDisplayMode!()">
						<slot name="display-mode-icon" />
					</button>
				</ButtonStyled>

				<Pagination
					:page="ctx.currentPage.value"
					:count="ctx.pageCount.value"
					:class="ctx.variant === 'web' ? 'mx-auto sm:ml-auto sm:mr-0' : 'ml-auto'"
					@switch-page="ctx.setPage"
				/>
			</div>

			<SearchFilterControl
				v-if="ctx.isServerType.value"
				v-model:selected-filters="ctx.serverCurrentFilters.value"
				:filters="ctx.serverFilterTypes.value"
				:provided-filters="[]"
				:overridden-provided-filter-types="[]"
			/>
			<SearchFilterControl
				v-else
				v-model:selected-filters="ctx.currentFilters.value"
				:filters="ctx.filters.value.filter((f) => f.display !== 'none')"
				:provided-filters="ctx.providedFilters?.value ?? []"
				:overridden-provided-filter-types="ctx.overriddenProvidedFilterTypes.value"
				:provided-message="lockedMessages?.providedBy"
			/>

			<div class="search">
				<section v-if="ctx.loading.value" class="offline">
					<component :is="ctx.loadingComponent ?? LoadingIndicator" />
				</section>
				<section v-else-if="ctx.offline?.value && ctx.totalHits.value === 0" class="offline">
					{{ formatMessage(messages.offline) }}
				</section>
				<section
					v-else-if="
						ctx.isServerType.value
							? ctx.serverHits.value.length === 0
							: ctx.projectHits.value.length === 0
					"
					class="offline"
				>
					<p>{{ formatMessage(messages.noResults) }}</p>
				</section>

				<ProjectCardList v-else :layout="ctx.effectiveLayout.value">
					<template v-if="ctx.isServerType.value">
						<ProjectCard
							v-for="result in ctx.serverHits.value"
							:key="`server-card-${result.project_id}`"
							:title="result.name"
							:icon-url="result.icon_url || undefined"
							:summary="result.summary"
							:tags="result.categories"
							:link="ctx.getServerProjectLink(result)"
							:server-online-players="result.minecraft_java_server?.ping?.data?.players_online ?? 0"
							:server-region="result.minecraft_server?.region"
							:server-recent-plays="result.minecraft_java_server?.verified_plays_2w ?? 0"
							:server-modpack-content="ctx.getServerModpackContent?.(result)"
							:server-ping="ctx.serverPings?.value?.[result.project_id]"
							:server-status-online="!!result.minecraft_java_server?.ping?.data"
							:hide-online-players-label="ctx.variant === 'app'"
							:hide-recent-plays-label="ctx.variant === 'app'"
							:layout="ctx.effectiveLayout.value"
							:max-tags="2"
							is-server-project
							exclude-loaders
							:color="result.color ?? undefined"
							:banner="result.featured_gallery ?? undefined"
							@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, result)"
							@mouseenter="ctx.onServerProjectHover?.(result)"
							@mouseleave="ctx.onProjectHoverEnd?.()"
						>
							<template v-if="ctx.getCardActions?.(result, ctx.projectType.value)?.length" #actions>
								<div class="flex gap-2">
									<ButtonStyled
										v-for="action in ctx.getCardActions(result, ctx.projectType.value)"
										:key="action.key"
										:color="action.color"
										:type="action.type"
										:circular="action.circular"
									>
										<button
											v-tooltip="action.tooltip"
											:disabled="action.disabled"
											@click.stop="action.onClick"
										>
											<component :is="action.icon" :class="action.iconClass" />
											<template v-if="!action.circular">{{ action.label }}</template>
										</button>
									</ButtonStyled>
								</div>
							</template>
						</ProjectCard>
					</template>
					<template v-else>
						<ProjectCard
							v-for="result in ctx.projectHits.value"
							:key="result.project_id"
							:link="ctx.getProjectLink(result)"
							:title="result.title"
							:icon-url="result.icon_url"
							:author="{
								name: result.organization == null ? result.author : result.organization,
								link:
									result.organization_id == null
										? ctx.variant === 'web'
											? `/user/${result.author_id ?? result.author}`
											: `https://modrinth.com/user/${result.author_id ?? result.author}`
										: ctx.variant === 'web'
											? `/organization/${result.organization_id}`
											: `https://modrinth.com/organization/${result.organization_id}`,
							}"
							:date-updated="result.date_modified"
							:date-published="result.date_created"
							:displayed-date="
								ctx.effectiveCurrentSortType.value.name === 'newest' ? 'published' : 'updated'
							"
							:downloads="result.downloads"
							:summary="result.description"
							:tags="result.display_categories"
							:all-tags="result.categories"
							:deprioritized-tags="ctx.deprioritizedTags.value"
							:exclude-loaders="ctx.excludeLoaders.value"
							:followers="result.follows"
							:banner="result.featured_gallery ?? undefined"
							:color="result.color ?? undefined"
							:environment="
								['mod', 'modpack'].includes(ctx.projectType.value)
									? {
											clientSide: result.client_side as Labrinth.Projects.v2.Environment,
											serverSide: result.server_side as Labrinth.Projects.v2.Environment,
										}
									: undefined
							"
							:layout="ctx.effectiveLayout.value"
							@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, result)"
							@mouseenter="ctx.onProjectHover?.(result)"
							@mouseleave="ctx.onProjectHoverEnd?.()"
						>
							<template v-if="ctx.getCardActions?.(result, ctx.projectType.value)?.length" #actions>
								<div class="flex gap-2">
									<ButtonStyled
										v-for="action in ctx.getCardActions(result, ctx.projectType.value)"
										:key="action.key"
										:color="action.color"
										:type="action.type"
										:circular="action.circular"
									>
										<button
											v-tooltip="action.tooltip"
											:disabled="action.disabled"
											@click.stop="action.onClick"
										>
											<component :is="action.icon" :class="action.iconClass" />
											<template v-if="!action.circular">{{ action.label }}</template>
										</button>
									</ButtonStyled>
								</div>
							</template>
						</ProjectCard>
					</template>
				</ProjectCardList>

				<div :class="ctx.variant === 'web' ? 'pagination-after mt-3' : 'flex justify-end mt-3'">
					<Pagination
						:page="ctx.currentPage.value"
						:count="ctx.pageCount.value"
						:class="ctx.variant === 'web' ? 'justify-end' : 'pagination-after'"
						@switch-page="ctx.setPage"
					/>
				</div>
			</div>

			<slot name="content-after" />
		</div>
	</component>

	<slot name="after" />
</template>

<style scoped>
.browse-install-header-bleed {
	grid-column: 1 / -1;
	margin-inline: -1.5rem;
	padding-inline: 0.75rem !important;
}

.browse-install-header-bleed::after {
	content: '';
	position: absolute;
	right: 50%;
	bottom: 0;
	width: 100vw;
	border-bottom: 1px solid var(--surface-5);
	transform: translateX(50%);
}

.browse-page-layout__content {
	display: contents;
}

.browse-page-layout__sidebar {
	grid-row: 3;
}

@media screen and (min-width: 1024px) {
	.browse-page-layout__content {
		display: block;
	}

	.browse-page-layout__sidebar {
		display: block;
	}
}
</style>
