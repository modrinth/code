import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useInstanceGroupMutations } from '@/features/instances/groups/use-instance-group-mutations'
import { FAVORITES_GROUP_ID } from '@/platform/app-lib/instance-groups/types'

import { UNGROUPED_GROUP_ID } from '../types'
import type { useLibraryView } from './use-library-view'

const messages = defineMessages({
	defaultGroupName: { id: 'app.library.group.default-name', defaultMessage: 'Group {number}' },
})

export function useLibraryGroups(view: ReturnType<typeof useLibraryView>) {
	const { formatMessage } = useVIntl()
	const mutations = useInstanceGroupMutations()
	const groupIdPendingNameEdit = ref<string | null>(null)

	const customGroups = computed(() =>
		view.groups.value.filter((group) => group.id !== FAVORITES_GROUP_ID),
	)
	const creatingGroup = computed(() => mutations.create.isPending.value)
	const reorderingGroups = computed(() => mutations.reorder.isPending.value)

	const getDefaultGroupName = () => {
		const existingNames = new Set(
			customGroups.value.map((group) => group.name.trim().toLowerCase()),
		)
		let groupNumber = existingNames.size + 1
		while (
			existingNames.has(
				formatMessage(messages.defaultGroupName, { number: groupNumber }).toLowerCase(),
			)
		) {
			groupNumber++
		}
		return formatMessage(messages.defaultGroupName, { number: groupNumber })
	}

	const createGroup = (name: string, instanceIds: Iterable<string> = []) =>
		mutations.create.mutateAsync({ name, instanceIds }).then(
			() => true,
			() => false,
		)

	const createDefaultGroup = async (instanceIds: Iterable<string> = []) => {
		if (creatingGroup.value) return false
		try {
			const group = await mutations.create.mutateAsync({
				name: getDefaultGroupName(),
				instanceIds,
			})
			view.displayState.value.group = 'Group'
			groupIdPendingNameEdit.value = group.id
			return true
		} catch {
			return false
		}
	}

	const completePendingGroupNameEdit = (groupId: string) => {
		if (groupIdPendingNameEdit.value === groupId) {
			groupIdPendingNameEdit.value = null
		}
	}

	const renameGroup = (groupId: string, newName: string) => {
		const name = newName.trim()
		if (view.groups.value.find((group) => group.id === groupId)?.name === name) {
			return Promise.resolve(true)
		}
		return mutations.rename.mutateAsync({ groupId, name }).then(
			() => true,
			() => false,
		)
	}

	const deleteGroup = (groupId: string) =>
		mutations.remove.mutateAsync(groupId).then(
			() => true,
			() => false,
		)

	const reorderGroups = async (orderedGroupIds: string[]) => {
		if (reorderingGroups.value) return false

		const currentOrder = view.orderedGroupIds.value
		const reorderableGroupIds = new Set([
			...customGroups.value.map((group) => group.id),
			UNGROUPED_GROUP_ID,
		])
		const orderedGroupIdSet = new Set(orderedGroupIds)
		if (
			orderedGroupIdSet.size !== orderedGroupIds.length ||
			orderedGroupIds.some((groupId) => !reorderableGroupIds.has(groupId))
		) {
			return false
		}

		let orderedGroupIndex = 0
		const reorderedGroupIds = currentOrder.map((groupId) =>
			orderedGroupIdSet.has(groupId) ? orderedGroupIds[orderedGroupIndex++] : groupId,
		)
		if (reorderedGroupIds.every((groupId, index) => groupId === currentOrder[index])) {
			return false
		}

		const reorderedCustomGroupIds = reorderedGroupIds.filter(
			(groupId) => groupId !== UNGROUPED_GROUP_ID,
		)
		const customGroupOrderChanged = reorderedCustomGroupIds.some(
			(groupId, index) => groupId !== customGroups.value[index]?.id,
		)
		const previousUngroupedPosition = view.displayState.value.ungroupedGroupPosition
		view.displayState.value.ungroupedGroupPosition = reorderedGroupIds.indexOf(UNGROUPED_GROUP_ID)
		if (!customGroupOrderChanged) return true

		try {
			await mutations.reorder.mutateAsync(reorderedCustomGroupIds)
			return true
		} catch {
			view.displayState.value.ungroupedGroupPosition = previousUngroupedPosition
			return false
		}
	}

	const getGroupIndex = (groupId: string) => view.orderedGroupIds.value.indexOf(groupId)

	const canMoveGroupUp = (groupId: string) => !reorderingGroups.value && getGroupIndex(groupId) > 0

	const canMoveGroupDown = (groupId: string) => {
		const groupIndex = getGroupIndex(groupId)
		return (
			!reorderingGroups.value &&
			groupIndex >= 0 &&
			groupIndex < view.orderedGroupIds.value.length - 1
		)
	}

	const moveGroup = (groupId: string, direction: -1 | 1) => {
		const orderedGroupIds = [...view.orderedGroupIds.value]
		const groupIndex = orderedGroupIds.indexOf(groupId)
		const targetIndex = groupIndex + direction
		if (groupIndex < 0 || targetIndex < 0 || targetIndex >= orderedGroupIds.length) {
			return Promise.resolve(false)
		}

		;[orderedGroupIds[groupIndex], orderedGroupIds[targetIndex]] = [
			orderedGroupIds[targetIndex],
			orderedGroupIds[groupIndex],
		]
		return reorderGroups(orderedGroupIds)
	}

	return {
		setMemberships: mutations.setMemberships,
		creatingGroup,
		reorderingGroups,
		groupIdPendingNameEdit,
		getDefaultGroupName,
		createGroup,
		createDefaultGroup,
		completePendingGroupNameEdit,
		renameGroup,
		deleteGroup,
		reorderGroups,
		canMoveGroupUp,
		canMoveGroupDown,
		moveGroup,
	}
}
