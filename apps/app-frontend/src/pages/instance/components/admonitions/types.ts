import type { StackedAdmonitionItem } from '@modrinth/ui'

export type InstanceAdmonitionKind =
	| 'shared-instance-stale'
	| 'shared-instance-unavailable'
	| 'shared-instance-wrong-account'

export type InstanceAdmonitionItem = StackedAdmonitionItem & {
	kind: InstanceAdmonitionKind
}

export type SharedInstanceRole = 'owner' | 'member'
