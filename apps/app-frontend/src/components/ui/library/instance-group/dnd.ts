export type InstanceGroupMove = {
	instanceId: string
	fromGroup: string | null
	toGroup: string
}

export type MoveInstanceBetweenGroups = (move: InstanceGroupMove) => Promise<void>
