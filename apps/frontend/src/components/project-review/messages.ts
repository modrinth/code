import { defineMessages } from '@modrinth/ui'

export const projectReviewMessages = defineMessages({
	title: {
		id: 'moderation.project-review.title',
		defaultMessage: 'Project review',
	},
	left: {
		id: 'moderation.project-review.left',
		defaultMessage: 'Project details',
	},
	right: {
		id: 'moderation.project-review.right',
		defaultMessage: 'Review issues',
	},
	bottom: {
		id: 'moderation.project-review.bottom',
		defaultMessage: 'Review summary',
	},
	description: {
		id: 'moderation.project-review.description',
		defaultMessage: 'Description',
	},
	gallery: {
		id: 'moderation.project-review.gallery',
		defaultMessage: 'Gallery',
	},
	disclosures: {
		id: 'moderation.project-review.disclosures',
		defaultMessage: 'Disclosures',
	},
	versions: {
		id: 'moderation.project-review.versions',
		defaultMessage: 'Versions',
	},
	history: {
		id: 'moderation.project-review.history',
		defaultMessage: 'History',
	},
	'tech-review': {
		id: 'moderation.project-review.tech-review',
		defaultMessage: 'Tech Review',
	},
	showLeft: {
		id: 'moderation.project-review.showLeft',
		defaultMessage: 'Show project details',
	},
	hideLeft: {
		id: 'moderation.project-review.hideLeft',
		defaultMessage: 'Hide project details',
	},
	showRight: {
		id: 'moderation.project-review.showRight',
		defaultMessage: 'Show review issues',
	},
	hideRight: {
		id: 'moderation.project-review.hideRight',
		defaultMessage: 'Hide review issues',
	},
	empty: {
		id: 'moderation.project-review.empty',
		defaultMessage: 'No project selected.',
	},
	loading: {
		id: 'moderation.project-review.loading',
		defaultMessage: 'Loading review workspace…',
	},
	matchesName: {
		id: 'moderation.project-review.matchesName',
		defaultMessage: 'Matches the project name',
	},
	customSlug: {
		id: 'moderation.project-review.customSlug',
		defaultMessage: 'Differs from the project name',
	},
	customLicense: {
		id: 'moderation.project-review.customLicense',
		defaultMessage: 'Custom',
	},
	license: {
		id: 'moderation.project-review.license',
		defaultMessage: 'License',
	},
	tags: {
		id: 'moderation.project-review.tags',
		defaultMessage: 'Categories',
	},
	links: {
		id: 'moderation.project-review.links',
		defaultMessage: 'Project links',
	},
	source: {
		id: 'moderation.project-review.source',
		defaultMessage: 'Source code',
	},
	issues: {
		id: 'moderation.project-review.issues',
		defaultMessage: 'Issue tracker',
	},
	discord: {
		id: 'moderation.project-review.discord',
		defaultMessage: 'Discord invite',
	},
	wiki: {
		id: 'moderation.project-review.wiki',
		defaultMessage: 'Wiki',
	},
	compatibility: {
		id: 'moderation.project-review.compatibility',
		defaultMessage: 'Compatibility',
	},
	gameVersions: {
		id: 'moderation.project-review.gameVersions',
		defaultMessage: 'Game versions',
	},
	platforms: {
		id: 'moderation.project-review.platforms',
		defaultMessage: 'Platforms',
	},
	environments: {
		id: 'moderation.project-review.environments',
		defaultMessage: 'Environments',
	},
	members: {
		id: 'moderation.project-review.members',
		defaultMessage: 'Members',
	},
	owner: {
		id: 'moderation.project-review.owner',
		defaultMessage: 'Owner',
	},
	details: {
		id: 'moderation.project-review.details',
		defaultMessage: 'Details',
	},
	projectType: {
		id: 'moderation.project-review.projectType',
		defaultMessage: 'Project type',
	},
	created: {
		id: 'moderation.project-review.created',
		defaultMessage: 'Created',
	},
	updated: {
		id: 'moderation.project-review.updated',
		defaultMessage: 'Updated',
	},
	submitted: {
		id: 'moderation.project-review.submitted',
		defaultMessage: 'Submitted',
	},
	submissions: {
		id: 'moderation.project-review.submissions',
		defaultMessage: '{count, plural, one {# submission} other {# submissions}}',
	},
	downloads: {
		id: 'moderation.project-review.downloads',
		defaultMessage: 'Downloads',
	},
	requesting: {
		id: 'moderation.project-review.requesting',
		defaultMessage: 'Requesting',
	},
	complete: {
		id: 'moderation.project-review.complete',
		defaultMessage: 'Complete',
	},
	total: {
		id: 'moderation.project-review.total',
		defaultMessage: 'Total',
	},
	back: {
		id: 'moderation.project-review.back',
		defaultMessage: 'Back',
	},
	skip: {
		id: 'moderation.project-review.skip',
		defaultMessage: 'Skip',
	},
	next: {
		id: 'moderation.project-review.next',
		defaultMessage: 'Next',
	},
	finish: {
		id: 'moderation.project-review.finish',
		defaultMessage: 'Finish',
	},
	reset: {
		id: 'moderation.project-review.reset',
		defaultMessage: 'Reset',
	},
	exit: {
		id: 'moderation.project-review.exit',
		defaultMessage: 'Exit',
	},
	retry: {
		id: 'moderation.project-review.retry',
		defaultMessage: 'Retry',
	},
	loadError: {
		id: 'moderation.project-review.loadError',
		defaultMessage: 'Could not load project data.',
	},
	queueError: {
		id: 'moderation.project-review.queueError',
		defaultMessage: 'Could not change projects. Please try again.',
	},
	unavailable: {
		id: 'moderation.project-review.unavailable',
		defaultMessage: 'Unavailable',
	},
	queueNavigation: {
		id: 'moderation.project-review.queueNavigation',
		defaultMessage: 'Review queue navigation',
	},
	approvedCount: {
		id: 'moderation.project-review.approvedCount',
		defaultMessage: '{count} approved',
	},
	archivedCount: {
		id: 'moderation.project-review.archivedCount',
		defaultMessage: '{count} archived',
	},
	unlistedCount: {
		id: 'moderation.project-review.unlistedCount',
		defaultMessage: '{count} unlisted',
	},
	withheldCount: {
		id: 'moderation.project-review.withheldCount',
		defaultMessage: '{count} withheld',
	},
	processingCount: {
		id: 'moderation.project-review.processingCount',
		defaultMessage: '{count} under review',
	},
	draftCount: {
		id: 'moderation.project-review.draftCount',
		defaultMessage: '{count} draft',
	},
	rejectedCount: {
		id: 'moderation.project-review.rejectedCount',
		defaultMessage: '{count} rejected',
	},
	privateCount: {
		id: 'moderation.project-review.privateCount',
		defaultMessage: '{count} private',
	},
	scheduledCount: {
		id: 'moderation.project-review.scheduledCount',
		defaultMessage: '{count} scheduled',
	},
	unknownCount: {
		id: 'moderation.project-review.unknownCount',
		defaultMessage: '{count} unknown',
	},
})
