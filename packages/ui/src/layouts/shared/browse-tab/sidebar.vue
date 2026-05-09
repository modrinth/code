<script setup lang="ts">
import { InfoIcon, XIcon } from '@modrinth/assets'
import { computed, toValue } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import SearchSidebarFilter from '#ui/components/search/SearchSidebarFilter.vue'
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { injectBrowseManager } from './providers/browse-manager'

const ctx = injectBrowseManager()
const { formatMessage } = useVIntl()

const isApp = computed(() => ctx.variant === 'app')
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))

function closeFiltersMenu() {
	if (ctx.filtersMenuOpen) {
		ctx.filtersMenuOpen.value = false
	}
	window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior })
}

const filterClass = computed(() => {
	if (isApp.value) {
		return 'border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
	}
	if (ctx.filtersMenuOpen?.value) {
		return 'border-0 border-b-[1px] border-solid border-divider last:border-b-0'
	}
	return 'card-shadow rounded-2xl bg-bg-raised'
})

const buttonClass = computed(() => {
	if (isApp.value) {
		return 'button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg'
	}
	return 'button-animation flex flex-col gap-1 px-6 py-4 w-full bg-transparent cursor-pointer border-none'
})

const contentClass = computed(() => (isApp.value ? 'mt-2 mb-3' : 'mb-4 mx-3'))
const innerPanelClass = computed(() => (isApp.value ? 'ml-2 mr-3' : 'p-1'))

function getFilterOpenByDefault(filterId: string): boolean {
	if (ctx.isServerType.value) {
		return ![
			'server_category_minecraft_server_meta',
			'server_category_minecraft_server_community',
			'server_game_version',
			'server_status',
		].includes(filterId)
	}
	if (isApp.value) {
		return filterId.startsWith('category') || filterId === 'environment' || filterId === 'license'
	}
	if (
		lockedMessages.value?.gameVersionShaderMessage &&
		ctx.projectType.value === 'shader' &&
		filterId === 'game_version'
	) {
		return false
	}
	return true
}
</script>

<template>
	<slot name="prepend" />

	<div v-if="ctx.filtersMenuOpen?.value" class="fixed inset-0 z-40 bg-bg" />

	<div
		class="flex flex-col"
		:class="{
			'gap-3': !isApp,
			'fixed inset-0 z-50 m-4 mb-0 overflow-auto rounded-t-3xl bg-bg-raised':
				ctx.filtersMenuOpen?.value,
		}"
	>
		<div
			v-if="ctx.filtersMenuOpen?.value"
			class="sticky top-0 z-10 mx-1 flex items-center justify-between gap-3 border-0 border-b-[1px] border-solid border-divider bg-bg-raised px-6 py-4"
		>
			<h3 class="m-0 text-lg text-contrast">{{ formatMessage(commonMessages.filtersLabel) }}</h3>
			<ButtonStyled circular>
				<button @click="closeFiltersMenu">
					<XIcon />
				</button>
			</ButtonStyled>
		</div>

		<div
			v-if="ctx.showHideInstalled?.value || ctx.showHideSelected?.value"
			:class="
				isApp
					? 'flex flex-col gap-3 border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
					: 'card-shadow flex flex-col gap-3 rounded-2xl bg-bg-raised p-4'
			"
		>
			<Checkbox
				v-if="ctx.showHideInstalled?.value"
				v-model="ctx.hideInstalled!.value"
				:label="
					ctx.hideInstalledLabel?.value ?? formatMessage(commonMessages.hideInstalledContentLabel)
				"
				class="filter-checkbox"
				@update:model-value="ctx.onFilterChange()"
				@click.prevent.stop
			/>
			<Checkbox
				v-if="ctx.showHideSelected?.value"
				v-model="ctx.hideSelected!.value"
				:label="
					ctx.hideSelectedLabel?.value ?? formatMessage(commonMessages.hideSelectedContentLabel)
				"
				class="filter-checkbox"
				@update:model-value="ctx.onFilterChange()"
				@click.prevent.stop
			/>
		</div>

		<template v-if="ctx.isServerType.value">
			<SearchSidebarFilter
				v-for="filterType in ctx.serverFilterTypes.value.filter((f) => f.options.length > 0)"
				:key="`server-filter-${filterType.id}`"
				v-model:selected-filters="ctx.serverCurrentFilters.value"
				v-model:toggled-groups="ctx.serverToggledGroups.value"
				:provided-filters="[]"
				:filter-type="filterType"
				:class="filterClass"
				:button-class="buttonClass"
				:content-class="contentClass"
				:inner-panel-class="innerPanelClass"
				:open-by-default="getFilterOpenByDefault(filterType.id)"
			>
				<template #header>
					<h3 :class="isApp ? 'text-base m-0' : 'm-0 text-lg'">
						{{ filterType.formatted_name }}
					</h3>
				</template>
			</SearchSidebarFilter>
		</template>
		<template v-else>
			<SearchSidebarFilter
				v-for="filter in ctx.filters.value.filter((f) => f.display !== 'none')"
				:key="`filter-${filter.id}`"
				v-model:selected-filters="ctx.currentFilters.value"
				v-model:toggled-groups="ctx.toggledGroups.value"
				v-model:overridden-provided-filter-types="ctx.overriddenProvidedFilterTypes.value"
				:provided-filters="ctx.providedFilters?.value ?? []"
				:filter-type="filter"
				:class="filterClass"
				:button-class="buttonClass"
				:content-class="contentClass"
				:inner-panel-class="innerPanelClass"
				:open-by-default="getFilterOpenByDefault(filter.id)"
			>
				<template #header>
					<h3 :class="isApp ? 'text-base m-0' : 'm-0 text-lg'">
						{{ filter.formatted_name }}
					</h3>
				</template>
				<template
					v-if="
						lockedMessages?.gameVersionShaderMessage &&
						ctx.projectType.value === 'shader' &&
						filter.id === 'game_version'
					"
					#prefix
				>
					<div class="mb-4 grid grid-cols-[auto_1fr] gap-2 px-3 text-sm font-medium text-blue">
						<InfoIcon class="mt-1 size-4" />
						<span>{{ lockedMessages.gameVersionShaderMessage }}</span>
					</div>
				</template>
				<template v-if="lockedMessages?.gameVersion" #locked-game_version>
					{{ lockedMessages.gameVersion }}
				</template>
				<template v-if="lockedMessages?.modLoader" #locked-mod_loader>
					{{ lockedMessages.modLoader }}
				</template>
				<template v-if="lockedMessages?.environment" #locked-environment>
					{{ lockedMessages.environment }}
				</template>
				<template v-if="lockedMessages?.syncButton" #sync-button>
					{{ lockedMessages.syncButton }}
				</template>
			</SearchSidebarFilter>
		</template>
	</div>
</template>
