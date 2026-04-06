<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SearchIcon } from '@modrinth/assets'
import { computed, toValue } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import LoadingIndicator from '#ui/components/base/LoadingIndicator.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import SearchFilterControl from '#ui/components/search/SearchFilterControl.vue'
import type { SortType } from '#ui/utils/search'

import BrowseInstallHeader from './header.vue'
import { injectBrowseManager } from './providers/browse-manager'

const ctx = injectBrowseManager()
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))

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
</script>

<template>
	<template v-if="ctx.installContext?.value && ctx.variant !== 'web'">
		<BrowseInstallHeader />
	</template>

	<NavTabs v-if="ctx.showProjectTypeTabs.value" :links="ctx.selectableProjectTypes.value" />

	<StyledInput
		v-model="ctx.query.value"
		:icon="SearchIcon"
		type="text"
		autocomplete="off"
		:placeholder="`Search ${ctx.projectType.value}s...`"
		clearable
		wrapper-class="w-full"
		:input-class="ctx.variant === 'web' ? '!h-12' : 'h-12'"
		@clear="ctx.clearSearch()"
	/>

	<div class="flex flex-wrap items-center gap-2">
		<Combobox
			:model-value="ctx.effectiveCurrentSortType.value"
			:options="sortOptions"
			:class="ctx.variant === 'web' ? '!w-auto flex-grow md:flex-grow-0' : 'max-w-[16rem]'"
			placeholder="Sort by"
			@update:model-value="(val: SortType) => (ctx.effectiveCurrentSortType.value = val)"
		>
			<template #prefix>
				<span class="font-semibold text-primary">Sort by:</span>
			</template>
		</Combobox>

		<Combobox
			:model-value="ctx.maxResults.value"
			:options="maxResultsOptions"
			:class="ctx.variant === 'web' ? '!w-auto flex-grow md:flex-grow-0' : 'max-w-[9rem]'"
			placeholder="View"
			@update:model-value="(val: number) => (ctx.maxResults.value = val)"
		>
			<template #prefix>
				<span class="font-semibold text-primary">View:</span>
			</template>
		</Combobox>

		<div v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value" class="lg:hidden">
			<ButtonStyled>
				<button @click="ctx.filtersMenuOpen.value = true">Filter results...</button>
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
			You are currently offline. Connect to the internet to browse Modrinth!
		</section>
		<section
			v-else-if="
				ctx.isServerType.value
					? ctx.serverHits.value.length === 0
					: ctx.projectHits.value.length === 0
			"
			class="offline"
		>
			<p>No results found for your query!</p>
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
					<template v-if="ctx.getCardActions" #actions>
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
									<component :is="action.icon" />
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
						name: result.author,
						link:
							ctx.variant === 'web'
								? `/user/${result.author}`
								: `https://modrinth.com/user/${result.author}`,
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
					<template v-if="ctx.getCardActions" #actions>
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
									<component :is="action.icon" />
									<template v-if="!action.circular">{{ action.label }}</template>
								</button>
							</ButtonStyled>
						</div>
					</template>
				</ProjectCard>
			</template>
		</ProjectCardList>

		<div :class="ctx.variant === 'web' ? 'pagination-after' : 'flex justify-end'">
			<Pagination
				:page="ctx.currentPage.value"
				:count="ctx.pageCount.value"
				:class="ctx.variant === 'web' ? 'justify-end' : 'pagination-after'"
				@switch-page="ctx.setPage"
			/>
		</div>
	</div>

	<slot name="after" />
</template>
