export type InstanceGroupMove = {
	instanceId: string
	fromGroup: string | null
	toGroup: string | null
}

export type MoveInstanceBetweenGroups = (move: InstanceGroupMove) => Promise<boolean>
