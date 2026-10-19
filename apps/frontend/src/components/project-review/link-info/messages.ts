import { defineMessages } from '@modrinth/ui'

export const messages = defineMessages({
	detailTooltip: {
		id: 'moderation.project-review.link-info.detailTooltip',
		defaultMessage: '{label}: {value}',
	},
	inviteExpires: {
		id: 'moderation.project-review.link-info.inviteExpires',
		defaultMessage: 'This invite has an expiry date.',
	},
	noExpiry: {
		id: 'moderation.project-review.link-info.noExpiry',
		defaultMessage: 'No expiry',
	},
	unlimitedUses: {
		id: 'moderation.project-review.link-info.unlimitedUses',
		defaultMessage: 'Unlimited uses',
	},
	externalLinks: {
		id: 'moderation.project-review.link-info.externalLinks',
		defaultMessage: 'External links on this page',
	},
	projectUnavailable: {
		id: 'moderation.project-review.link-info.projectUnavailable',
		defaultMessage:
			'This project is not publicly available or is archived. Check its status below.',
	},
	inviteExpired: {
		id: 'moderation.project-review.link-info.inviteExpired',
		defaultMessage: 'This invite has expired.',
	},
	loading: {
		id: 'moderation.project-review.link-info.loading',
		defaultMessage: 'Loading link details…',
	},
	unavailable: {
		id: 'moderation.project-review.link-info.unavailable',
		defaultMessage:
			'Additional details could not be loaded. The service may be unavailable, private, rate limited, or blocked by the browser.',
	},
	unsupported: {
		id: 'moderation.project-review.link-info.unsupported',
		defaultMessage: 'No additional public metadata is available for this link.',
	},
	notFound: {
		id: 'moderation.project-review.link-info.notFound',
		defaultMessage: 'This resource was not found. It may be private, deleted, or expired.',
	},
	archived: {
		id: 'moderation.project-review.link-info.archived',
		defaultMessage: 'This repository is archived; its features may be read-only.',
	},
	issuesDisabled: {
		id: 'moderation.project-review.link-info.issuesDisabled',
		defaultMessage: 'Issues are disabled.',
	},
	wikiDisabled: {
		id: 'moderation.project-review.link-info.wikiDisabled',
		defaultMessage: 'The wiki is disabled.',
	},
	discussionsDisabled: {
		id: 'moderation.project-review.link-info.discussionsDisabled',
		defaultMessage: 'Discussions are disabled.',
	},
	projectStatus: {
		id: 'moderation.project-review.link-info.projectStatus',
		defaultMessage: 'Project status: {status}',
	},
	privateCommunity: {
		id: 'moderation.project-review.link-info.privateCommunity',
		defaultMessage: 'This community is private.',
	},
	repository: {
		id: 'moderation.project-review.link-info.repository',
		defaultMessage: 'Repository',
	},
	user: {
		id: 'moderation.project-review.link-info.user',
		defaultMessage: 'User',
	},
	organization: {
		id: 'moderation.project-review.link-info.organization',
		defaultMessage: 'Organization',
	},
	issues: {
		id: 'moderation.project-review.link-info.issues',
		defaultMessage: 'Issues',
	},
	wiki: {
		id: 'moderation.project-review.link-info.wiki',
		defaultMessage: 'Wiki',
	},
	discussions: {
		id: 'moderation.project-review.link-info.discussions',
		defaultMessage: 'Discussions',
	},
	sponsors: {
		id: 'moderation.project-review.link-info.sponsors',
		defaultMessage: 'Sponsors',
	},
	invite: {
		id: 'moderation.project-review.link-info.invite',
		defaultMessage: 'Server invite',
	},
	server: {
		id: 'moderation.project-review.link-info.server',
		defaultMessage: 'Server',
	},
	channel: {
		id: 'moderation.project-review.link-info.channel',
		defaultMessage: 'Channel',
	},
	dm: {
		id: 'moderation.project-review.link-info.dm',
		defaultMessage: 'DM link',
	},
	project: {
		id: 'moderation.project-review.link-info.project',
		defaultMessage: 'Project',
	},
	video: {
		id: 'moderation.project-review.link-info.video',
		defaultMessage: 'Video',
	},
	playlist: {
		id: 'moderation.project-review.link-info.playlist',
		defaultMessage: 'Playlist',
	},
	community: {
		id: 'moderation.project-review.link-info.community',
		defaultMessage: 'Community',
	},
	document: {
		id: 'moderation.project-review.link-info.document',
		defaultMessage: 'Document',
	},
	donation: {
		id: 'moderation.project-review.link-info.donation',
		defaultMessage: 'Donation profile',
	},
	board: {
		id: 'moderation.project-review.link-info.board',
		defaultMessage: 'Board',
	},
	website: {
		id: 'moderation.project-review.link-info.website',
		defaultMessage: 'Website',
	},
	stars: {
		id: 'moderation.project-review.link-info.stars',
		defaultMessage: 'Stars',
	},
	forks: {
		id: 'moderation.project-review.link-info.forks',
		defaultMessage: 'Forks',
	},
	size: {
		id: 'moderation.project-review.link-info.size',
		defaultMessage: 'Repository size',
	},
	language: {
		id: 'moderation.project-review.link-info.language',
		defaultMessage: 'Language',
	},
	license: {
		id: 'moderation.project-review.link-info.license',
		defaultMessage: 'License',
	},
	lastPush: {
		id: 'moderation.project-review.link-info.lastPush',
		defaultMessage: 'Last push',
	},
	forkOf: {
		id: 'moderation.project-review.link-info.forkOf',
		defaultMessage: 'Fork of',
	},
	followers: {
		id: 'moderation.project-review.link-info.followers',
		defaultMessage: 'Followers',
	},
	repositories: {
		id: 'moderation.project-review.link-info.repositories',
		defaultMessage: 'Public repositories',
	},
	company: {
		id: 'moderation.project-review.link-info.company',
		defaultMessage: 'Company',
	},
	owner: {
		id: 'moderation.project-review.link-info.owner',
		defaultMessage: 'Owner',
	},
	openIssues: {
		id: 'moderation.project-review.link-info.openIssues',
		defaultMessage: 'Open issues',
	},
	closedIssues: {
		id: 'moderation.project-review.link-info.closedIssues',
		defaultMessage: 'Closed issues',
	},
	members: {
		id: 'moderation.project-review.link-info.members',
		defaultMessage: 'Members',
	},
	online: {
		id: 'moderation.project-review.link-info.online',
		defaultMessage: 'Online',
	},
	inviter: {
		id: 'moderation.project-review.link-info.inviter',
		defaultMessage: 'Invited by',
	},
	expires: {
		id: 'moderation.project-review.link-info.expires',
		defaultMessage: 'Expires',
	},
	uses: {
		id: 'moderation.project-review.link-info.uses',
		defaultMessage: 'Uses',
	},
	maxUses: {
		id: 'moderation.project-review.link-info.maxUses',
		defaultMessage: 'Use limit',
	},
	target: {
		id: 'moderation.project-review.link-info.target',
		defaultMessage: 'Invite type',
	},
	stream: {
		id: 'moderation.project-review.link-info.stream',
		defaultMessage: 'Stream',
	},
	activity: {
		id: 'moderation.project-review.link-info.activity',
		defaultMessage: 'Activity',
	},
	created: {
		id: 'moderation.project-review.link-info.created',
		defaultMessage: 'Account created',
	},
	bot: {
		id: 'moderation.project-review.link-info.bot',
		defaultMessage: 'Bot',
	},
	downloads: {
		id: 'moderation.project-review.link-info.downloads',
		defaultMessage: 'Downloads',
	},
	role: {
		id: 'moderation.project-review.link-info.role',
		defaultMessage: 'Role',
	},
	author: {
		id: 'moderation.project-review.link-info.author',
		defaultMessage: 'Author',
	},
	subscribers: {
		id: 'moderation.project-review.link-info.subscribers',
		defaultMessage: 'Subscribers',
	},
	topic: {
		id: 'moderation.project-review.link-info.topic',
		defaultMessage: 'Topic',
	},
	status: {
		id: 'moderation.project-review.link-info.status',
		defaultMessage: 'Status',
	},
	active: {
		id: 'moderation.project-review.link-info.active',
		defaultMessage: 'Active',
	},
	archivedValue: {
		id: 'moderation.project-review.link-info.archivedValue',
		defaultMessage: 'Archived',
	},
	enabled: {
		id: 'moderation.project-review.link-info.enabled',
		defaultMessage: 'Enabled',
	},
	disabled: {
		id: 'moderation.project-review.link-info.disabled',
		defaultMessage: 'Disabled',
	},
})
