<script setup lang="ts">
import {
	defineMessages,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	formatLoader,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import { isUninstalledSharedInstance, useLibrary } from '@/components/ui/library/use-library'

const { filters, libraryInstances } = useLibrary()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	instanceType: { id: 'app.library.filter.instance-type', defaultMessage: 'Instance type' },
	modpack: { id: 'app.library.filter.instance-type.modpack', defaultMessage: 'Modpack' },
	server: { id: 'app.library.filter.instance-type.server', defaultMessage: 'Server' },
	custom: { id: 'app.library.filter.instance-type.custom', defaultMessage: 'Custom' },
	gameVersion: { id: 'app.library.filter.game-version', defaultMessage: 'Game version' },
	loader: { id: 'app.library.filter.loader', defaultMessage: 'Loader' },
	installState: { id: 'app.library.filter.install-state', defaultMessage: 'Install status' },
	installed: { id: 'app.library.filter.install-state.installed', defaultMessage: 'Installed' },
	notInstalled: {
		id: 'app.library.filter.install-state.not-installed',
		defaultMessage: 'Not installed',
	},
	owner: { id: 'app.library.filter.owner', defaultMessage: 'Owner' },
	ownerMe: { id: 'app.library.filter.owner.me', defaultMessage: 'You' },
	ownerOthers: { id: 'app.library.filter.owner.others', defaultMessage: 'Others' },
	filterBy: { id: 'app.library.filter.label', defaultMessage: 'Filter by' },
	addFilter: { id: 'app.library.filter.add', defaultMessage: 'Add filter' },
	clearFilters: { id: 'app.library.filter.clear', defaultMessage: 'Clear filters' },
})

const hasUninstalledInstances = computed(() =>
	libraryInstances.value.some(isUninstalledSharedInstance),
)
const hasSharedInstances = computed(() =>
	libraryInstances.value.some(
		(instance) => isUninstalledSharedInstance(instance) || !!instance.shared_instance,
	),
)

const filterCategories = computed<DropdownFilterBarCategory[]>(() => [
	{
		key: 'instanceType',
		label: formatMessage(messages.instanceType),
		options: [
			{ value: 'modpack', label: formatMessage(messages.modpack) },
			{ value: 'server', label: formatMessage(messages.server) },
			{ value: 'custom', label: formatMessage(messages.custom) },
		],
	},
	{
		key: 'gameVersion',
		label: formatMessage(messages.gameVersion),
		searchable: new Set(libraryInstances.value.map((instance) => instance.game_version)).size > 5,
		options: [...new Set(libraryInstances.value.map((instance) => instance.game_version))]
			.sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
			.map((version) => ({ value: version, label: version })),
	},
	{
		key: 'loader',
		label: formatMessage(messages.loader),
		options: [...new Set(libraryInstances.value.map((instance) => instance.loader))]
			.map((loader) => ({
				value: loader,
				label: formatLoader(formatMessage, loader),
			}))
			.sort((a, b) => a.label.localeCompare(b.label)),
	},
	...(hasUninstalledInstances.value || filters.value.installState.length > 0
		? [
				{
					key: 'installState',
					label: formatMessage(messages.installState),
					options: [
						{ value: 'installed', label: formatMessage(messages.installed) },
						{ value: 'not_installed', label: formatMessage(messages.notInstalled) },
					],
				},
			]
		: []),
	...(hasSharedInstances.value || filters.value.owner.length > 0
		? [
				{
					key: 'owner',
					label: formatMessage(messages.owner),
					options: [
						{ value: 'me', label: formatMessage(messages.ownerMe) },
						{ value: 'others', label: formatMessage(messages.ownerOthers) },
					],
				},
			]
		: []),
])
</script>

<template>
	<DropdownFilterBar
		v-model="filters"
		:categories="filterCategories"
		use-filter-icon
		:label="formatMessage(messages.filterBy)"
		:add-label="formatMessage(messages.addFilter)"
		:clear-label="formatMessage(messages.clearFilters)"
		apply-immediately
		checkbox-position="right"
	/>
</template>
