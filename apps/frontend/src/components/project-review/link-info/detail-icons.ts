import {
	ArchiveIcon,
	BotIcon,
	CalendarIcon,
	CheckIcon,
	CircleDotIcon,
	ClockIcon,
	CodeIcon,
	DownloadIcon,
	GitCommitHorizontalIcon,
	GitForkIcon,
	HashIcon,
	InfoIcon,
	OnlineIndicatorIcon,
	PlayIcon,
	ScaleIcon,
	StarIcon,
	TagCategoryBuilding2Icon,
	TagCategoryLibraryIcon,
	TagIcon,
	TriangleAlertIcon,
	UserIcon,
	UsersIcon,
} from '@modrinth/assets'
import type { Component } from 'vue'

import type { Detail, Label } from './types'

const icons: Partial<Record<Label, Component>> = {
	stars: StarIcon,
	forks: GitForkIcon,
	size: ArchiveIcon,
	language: CodeIcon,
	license: ScaleIcon,
	lastPush: GitCommitHorizontalIcon,
	forkOf: GitForkIcon,
	followers: UsersIcon,
	repositories: TagCategoryLibraryIcon,
	company: TagCategoryBuilding2Icon,
	owner: UserIcon,
	openIssues: CircleDotIcon,
	closedIssues: CheckIcon,
	members: UsersIcon,
	online: OnlineIndicatorIcon,
	inviter: UserIcon,
	channel: HashIcon,
	expires: ClockIcon,
	uses: UsersIcon,
	maxUses: UsersIcon,
	target: PlayIcon,
	created: CalendarIcon,
	bot: BotIcon,
	downloads: DownloadIcon,
	role: UserIcon,
	author: UserIcon,
	subscribers: UsersIcon,
	topic: TagIcon,
	status: InfoIcon,
}

export function detailIcon(entry: Detail): Component {
	if (entry.warning) return TriangleAlertIcon
	if (
		typeof entry.value === 'object' &&
		['noExpiry', 'unlimitedUses'].includes(entry.value.message)
	)
		return CheckIcon
	return icons[entry.label] ?? InfoIcon
}
