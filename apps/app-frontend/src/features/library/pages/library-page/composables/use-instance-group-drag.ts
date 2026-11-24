import { defineMessages, useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, ref } from 'vue'

import type { InstanceGroupMembershipUpdate } from '@/platform/app-lib/instance-groups/types'

import {
	type ActiveInstanceGroupDrag,
	getLibraryInstanceSelectionKey,
	type LibraryInstanceSelection,
	UNGROUPED_GROUP_ID,
} from '../types'
import type { useLibraryGroups } from './use-library-groups'
import type { useLibrarySelection } from './use-library-selection'
import type { useLibraryView } from './use-library-view'

type Point = { x: number; y: number }

const messages = defineMessages({
	alreadyInGroup: {
		id: 'app.library.drag.already-in-group',
		defaultMessage: 'Already in this group',
	},
	duplicateIntoGroup: {
		id: 'app.library.drag.duplicate-into-group',
		defaultMessage:
			'{count, plural, one {Duplicate into group} other {Duplicate # instances to group}}',
	},
	duplicateToGroup: {
		id: 'app.library.drag.duplicate-to-group',
		defaultMessage: 'Duplicate to group',
	},
})

const normalizeGroupId = (groupId: string) => (groupId === UNGROUPED_GROUP_ID ? null : groupId)

const groupSelectionsByInstanceId = (selections: LibraryInstanceSelection[]) => {
	const grouped = new Map<string, LibraryInstanceSelection[]>()
	for (const selection of selections) {
		grouped.set(selection.instanceId, [...(grouped.get(selection.instanceId) ?? []), selection])
	}
	return grouped
}

const getSourceGroupIds = (selections: LibraryInstanceSelection[]) =>
	new Set(
		selections
			.map((selection) => normalizeGroupId(selection.groupId))
			.filter((groupId): groupId is string => groupId !== null),
	)

export function useInstanceGroupDrag({
	view,
	selection,
	groups,
}: {
	view: ReturnType<typeof useLibraryView>
	selection: ReturnType<typeof useLibrarySelection>
	groups: ReturnType<typeof useLibraryGroups>
}) {
	const { formatMessage } = useVIntl()
	const activeInstanceGroupDrag = ref<ActiveInstanceGroupDrag | null>(null)
	const instanceGroupDragTarget = ref<string | null>(null)
	const instanceGroupDragPointer = ref<Point>({ x: 0, y: 0 })
	const isAddingInstanceToGroup = ref(false)

	const activeDraggedInstanceKeys = computed(
		() =>
			new Set(activeInstanceGroupDrag.value?.instances.map(getLibraryInstanceSelectionKey) ?? []),
	)

	const getDraggedInstances = (drag: ActiveInstanceGroupDrag | null) => {
		const draggedInstanceIds = new Set(drag?.instances.map((item) => item.instanceId) ?? [])
		return view.instances.value.filter((instance) => draggedInstanceIds.has(instance.id))
	}

	const updateInstanceGroupDrag = (pointer?: Point, altKey = isAddingInstanceToGroup.value) => {
		if (pointer) {
			instanceGroupDragPointer.value = { x: pointer.x, y: pointer.y }
		}
		isAddingInstanceToGroup.value = altKey
	}

	const startInstanceGroupDrag = (
		instanceId: string,
		groupId: string,
		pointer?: Point,
		altKey = false,
	) => {
		if (view.displayState.value.group !== 'Group') return

		const primaryInstance = { instanceId, groupId }
		const primaryInstanceKey = getLibraryInstanceSelectionKey(primaryInstance)
		const selected = selection.selectedLibraryInstances.value
		const draggedInstances = selected.has(primaryInstanceKey)
			? [
					primaryInstance,
					...[...selected.values()].filter(
						(item) => getLibraryInstanceSelectionKey(item) !== primaryInstanceKey,
					),
				].filter((item) => view.instances.value.some((instance) => instance.id === item.instanceId))
			: [primaryInstance]

		activeInstanceGroupDrag.value = {
			instances: draggedInstances,
			primaryInstanceId: instanceId,
			fromGroup: normalizeGroupId(groupId),
		}
		updateInstanceGroupDrag(pointer, altKey)
	}

	const finishInstanceGroupDrag = () => {
		activeInstanceGroupDrag.value = null
		instanceGroupDragTarget.value = null
		isAddingInstanceToGroup.value = false
	}

	const setInstanceGroupDragTarget = (groupId: string | null) => {
		instanceGroupDragTarget.value = groupId
	}

	const getInstanceGroupDropState = (groupId: string) => {
		const drag = activeInstanceGroupDrag.value
		const selectionsByInstanceId = groupSelectionsByInstanceId(drag?.instances ?? [])
		const draggedInstances = getDraggedInstances(drag)
		const toGroup = normalizeGroupId(groupId)
		const operation = isAddingInstanceToGroup.value ? 'add' : 'move'
		const canAddToGroup = operation !== 'add' || toGroup !== null
		const hasChanges = draggedInstances.some((instance) => {
			if (operation === 'add') {
				return toGroup !== null && !instance.group_ids.includes(toGroup)
			}
			if (toGroup !== null) {
				return !instance.group_ids.includes(toGroup)
			}
			const sourceGroupIds = getSourceGroupIds(selectionsByInstanceId.get(instance.id) ?? [])
			return instance.group_ids.some((sourceGroupId) => sourceGroupIds.has(sourceGroupId))
		})

		return {
			alreadyInGroup: draggedInstances.length > 0 && !hasChanges,
			canDrop: !!drag && draggedInstances.length > 0 && hasChanges && canAddToGroup,
			operation,
		} as const
	}

	const instanceGroupDragStatus = computed(() => {
		const target = instanceGroupDragTarget.value
		if (target) {
			const dropState = getInstanceGroupDropState(target)
			const isSourceGroup = normalizeGroupId(target) === activeInstanceGroupDrag.value?.fromGroup
			if (!isSourceGroup && !dropState.canDrop && dropState.alreadyInGroup) {
				return formatMessage(messages.alreadyInGroup)
			}
			if (!dropState.canDrop || dropState.operation !== 'add') return undefined

			const count = new Set(
				activeInstanceGroupDrag.value?.instances.map((item) => item.instanceId) ?? [],
			).size
			return formatMessage(messages.duplicateIntoGroup, { count })
		}

		return isAddingInstanceToGroup.value ? formatMessage(messages.duplicateToGroup) : undefined
	})

	const moveDraggedInstancesToGroup = async (groupId: string, addToGroup = false) => {
		const drag = activeInstanceGroupDrag.value
		if (!drag || !getInstanceGroupDropState(groupId).canDrop) return false

		const toGroup = normalizeGroupId(groupId)
		const shouldAdd = addToGroup && toGroup !== null
		const selectionsByInstanceId = groupSelectionsByInstanceId(drag.instances)
		const updates: InstanceGroupMembershipUpdate[] = getDraggedInstances(drag).flatMap(
			(instance) => {
				if (toGroup && instance.group_ids.includes(toGroup)) return []

				let nextGroupIds = [...instance.group_ids]
				if (!shouldAdd) {
					const sourceGroupIds = getSourceGroupIds(selectionsByInstanceId.get(instance.id) ?? [])
					nextGroupIds = nextGroupIds.filter((sourceGroupId) => !sourceGroupIds.has(sourceGroupId))
				}
				if (toGroup) nextGroupIds.push(toGroup)

				return [{ instance_id: instance.id, group_ids: nextGroupIds }]
			},
		)

		try {
			await groups.setMemberships.mutateAsync(updates)
		} catch {
			return false
		}

		selection.clearLibraryInstanceSelection()
		return updates.length > 0
	}

	useEventListener(window, 'keydown', (event) => {
		if (event.key === 'Alt' && activeInstanceGroupDrag.value) {
			isAddingInstanceToGroup.value = true
		}
	})
	useEventListener(window, 'keyup', (event) => {
		if (event.key === 'Alt') {
			isAddingInstanceToGroup.value = false
		}
	})
	useEventListener(window, 'blur', () => {
		isAddingInstanceToGroup.value = false
	})

	return {
		activeInstanceGroupDrag,
		activeDraggedInstanceKeys,
		instanceGroupDragTarget,
		instanceGroupDragPointer,
		instanceGroupDragStatus,
		isAddingInstanceToGroup,
		startInstanceGroupDrag,
		updateInstanceGroupDrag,
		finishInstanceGroupDrag,
		setInstanceGroupDragTarget,
		getInstanceGroupDropState,
		moveDraggedInstancesToGroup,
	}
}
