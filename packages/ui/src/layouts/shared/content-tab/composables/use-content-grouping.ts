import { useLocalStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentGroupsManager } from '../providers/content-manager'
import type { ContentItem, ContentType } from '../types'
import { getContentGroupItemKey, UNGROUPED_CONTENT_SECTION_ID } from '../utils/content-groups'
import {
	buildDerivedSections,
	type ContentDerivedGroupBy,
	contentDerivedSectionId,
} from './content-derived-groups'

export type ContentGroupBy = 'custom' | ContentDerivedGroupBy | 'none'

export const contentGroupByOptions: ContentGroupBy[] = [
	'custom',
	'category',
	'author',
	'date',
	'license',
	'none',
]

export interface ContentGroupSection {
	id: string
	groupId: string | null
	name: string
	items: ContentItem[]
}

export interface ContentGroupingOptions {
	manager: ContentGroupsManager | undefined
	activeType: Ref<ContentType>
	visibleItems: Ref<ContentItem[]>
	filtering: Ref<boolean>
	persistKey?: string
}

const messages = defineMessages({
	defaultGroupName: {
		id: 'content.groups.default-name',
		defaultMessage: 'Group {number}',
	},
	ungrouped: {
		id: 'content.groups.ungrouped',
		defaultMessage: 'Ungrouped',
	},
})

function persistedRecord<T>(key: string | undefined, prefix: string) {
	return key
		? useLocalStorage<Record<string, T>>(`${prefix}:${key}`, {})
		: ref<Record<string, T>>({})
}

export function useContentGrouping(options: ContentGroupingOptions) {
	const { formatMessage } = useVIntl()
	const formatMonth = useFormatDateTime({ month: 'long', year: 'numeric' })
	const formatters = { formatMessage, formatMonth: (date: Date) => formatMonth(date) }
	const manager = options.manager

	const supported = computed(() => !!manager?.supportsType(options.activeType.value))

	const groupByByType = persistedRecord<ContentGroupBy>(options.persistKey, 'content-group-by')
	function setGroupBy(type: ContentType, value: ContentGroupBy) {
		groupByByType.value = { ...groupByByType.value, [type]: value }
	}
	const groupBy = computed<ContentGroupBy>({
		get: () => {
			const value = groupByByType.value[options.activeType.value]
			return value && contentGroupByOptions.includes(value) ? value : 'none'
		},
		set: (value) => setGroupBy(options.activeType.value, value),
	})
	const grouped = computed(() => supported.value && groupBy.value !== 'none')
	const customGrouped = computed(() => supported.value && groupBy.value === 'custom')

	const typeGroups = computed(() =>
		(manager?.groups.value ?? []).filter((group) => group.type === options.activeType.value),
	)

	const groupIdByItemKey = computed(() => {
		const map = new Map<string, string>()
		for (const group of typeGroups.value) {
			for (const key of group.itemKeys) map.set(key, group.id)
		}
		return map
	})

	function groupIdOf(item: ContentItem) {
		return groupIdByItemKey.value.get(getContentGroupItemKey(item)) ?? null
	}

	const pendingRenameGroupId = ref<string | null>(null)
	const createdWhileFilteringIds = ref(new Set<string>())

	watch(options.filtering, (filtering) => {
		if (!filtering) createdWhileFilteringIds.value = new Set()
	})

	const sections = computed<ContentGroupSection[]>(() => {
		if (!grouped.value) return []
		if (groupBy.value !== 'custom' && groupBy.value !== 'none') {
			return buildDerivedSections(groupBy.value, options.visibleItems.value, formatters).map(
				(section) => ({ ...section, groupId: null }),
			)
		}
		const itemsByGroup = new Map<string | null, ContentItem[]>()
		for (const item of options.visibleItems.value) {
			const groupId = groupIdOf(item)
			const items = itemsByGroup.get(groupId) ?? []
			items.push(item)
			itemsByGroup.set(groupId, items)
		}

		const custom = typeGroups.value.map<ContentGroupSection>((group) => ({
			id: group.id,
			groupId: group.id,
			name: group.name,
			items: itemsByGroup.get(group.id) ?? [],
		}))
		const ungrouped: ContentGroupSection = {
			id: UNGROUPED_CONTENT_SECTION_ID,
			groupId: null,
			name: formatMessage(messages.ungrouped),
			items: itemsByGroup.get(null) ?? [],
		}

		return [...custom, ungrouped].filter(
			(section) =>
				section.items.length > 0 ||
				(section.groupId !== null &&
					(!options.filtering.value || createdWhileFilteringIds.value.has(section.groupId))),
		)
	})

	function sectionIdOf(item: ContentItem) {
		if (groupBy.value !== 'custom' && groupBy.value !== 'none') {
			return contentDerivedSectionId(groupBy.value, item, formatters)
		}
		return groupIdOf(item) ?? UNGROUPED_CONTENT_SECTION_ID
	}

	const collapsedSections = persistedRecord<boolean>(options.persistKey, 'content-collapsed-groups')
	const sectionKey = (sectionId: string, type = options.activeType.value) => `${type}:${sectionId}`

	function isCollapsed(sectionId: string) {
		return !options.filtering.value && !!collapsedSections.value[sectionKey(sectionId)]
	}

	function setCollapsed(sectionId: string, collapsed: boolean, type = options.activeType.value) {
		const key = sectionKey(sectionId, type)
		const { [key]: _, ...rest } = collapsedSections.value
		collapsedSections.value = collapsed ? { ...rest, [key]: true } : rest
	}

	function defaultGroupName() {
		const names = new Set(typeGroups.value.map((group) => group.name.trim().toLowerCase()))
		let number = typeGroups.value.length + 1
		while (names.has(formatMessage(messages.defaultGroupName, { number }).toLowerCase())) number++
		return formatMessage(messages.defaultGroupName, { number })
	}

	const creating = ref(false)

	async function createGroup(items: ContentItem[] = []) {
		if (!manager || creating.value) return null
		creating.value = true
		const type = options.activeType.value
		try {
			const group = await manager.create(
				type,
				defaultGroupName(),
				items.map(getContentGroupItemKey),
			)
			if (!group) return null
			setGroupBy(type, 'custom')
			setCollapsed(group.id, false, type)
			if (options.filtering.value) {
				createdWhileFilteringIds.value = new Set([...createdWhileFilteringIds.value, group.id])
			}
			if (options.activeType.value === type) pendingRenameGroupId.value = group.id
			return group
		} finally {
			creating.value = false
		}
	}

	function completePendingRename(groupId: string) {
		if (pendingRenameGroupId.value === groupId) pendingRenameGroupId.value = null
	}

	function moveItems(items: ContentItem[], groupId: string | null) {
		if (!manager) return Promise.resolve(false)
		const keys = items.filter((item) => groupIdOf(item) !== groupId).map(getContentGroupItemKey)
		if (keys.length === 0) return Promise.resolve(true)
		return manager.move(options.activeType.value, [...new Set(keys)], groupId)
	}

	function renameGroup(groupId: string, name: string) {
		const trimmed = name.trim()
		if (!manager || !trimmed) return Promise.resolve(false)
		if (typeGroups.value.find((group) => group.id === groupId)?.name === trimmed) {
			return Promise.resolve(true)
		}
		return manager.rename(groupId, trimmed)
	}

	function deleteGroup(groupId: string) {
		if (!manager) return Promise.resolve(false)
		setCollapsed(groupId, false)
		return manager.remove(groupId)
	}

	function groupIndex(groupId: string) {
		return typeGroups.value.findIndex((group) => group.id === groupId)
	}

	function canMoveGroup(groupId: string, direction: -1 | 1) {
		const index = groupIndex(groupId)
		const target = index + direction
		return index >= 0 && target >= 0 && target < typeGroups.value.length
	}

	function moveGroup(groupId: string, direction: -1 | 1) {
		if (!manager || !canMoveGroup(groupId, direction)) return Promise.resolve(false)
		const ids = typeGroups.value.map((group) => group.id)
		const index = groupIndex(groupId)
		;[ids[index], ids[index + direction]] = [ids[index + direction], ids[index]]
		return manager.reorder(options.activeType.value, ids)
	}

	return {
		supported,
		groupBy,
		grouped,
		customGrouped,
		typeGroups,
		sections,
		groupIdOf,
		sectionIdOf,
		isCollapsed,
		setCollapsed,
		creating,
		pendingRenameGroupId,
		completePendingRename,
		createGroup,
		moveItems,
		renameGroup,
		deleteGroup,
		canMoveGroup,
		moveGroup,
	}
}
