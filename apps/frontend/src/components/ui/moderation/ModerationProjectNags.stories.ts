import type { Labrinth } from '@modrinth/api-client'
import { type Nag, nagDefinitions, toProjectNag } from '@modrinth/moderation'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ModerationProjectNags from './ModerationProjectNags.vue'

const categories = [
	'adventure',
	'cursed',
	'decoration',
	'economy',
	'equipment',
	'food',
	'game-mechanics',
	'library',
	'magic',
]

function createProject(status: Labrinth.Projects.v2.ProjectStatus): Labrinth.Projects.v2.Project {
	return {
		id: 'storybook-publishing-checklist',
		slug: 'storybook-publishing-checklist',
		project_type: 'mod',
		actualProjectType: 'mod',
		team: 'storybook-team',
		organization: null,
		title: 'Publishing Checklist Preview',
		description: 'A project fixture for previewing the publishing checklist.',
		body: '',
		published: '2026-01-01T00:00:00.000Z',
		updated: '2026-01-01T00:00:00.000Z',
		status,
		license: {
			id: 'MIT',
			name: 'MIT License',
		},
		client_side: 'required',
		server_side: 'optional',
		downloads: 0,
		followers: 0,
		categories,
		additional_categories: [],
		game_versions: [],
		loaders: [],
		versions: [],
		gallery: [],
		thread_id: 'storybook-thread',
		monetization_status: 'monetized',
	}
}

function createProjectV3(status: Labrinth.Projects.v2.ProjectStatus): Labrinth.Projects.v3.Project {
	return {
		id: 'storybook-publishing-checklist',
		slug: 'storybook-publishing-checklist',
		project_types: ['mod'],
		games: ['minecraft:java-edition'],
		team_id: 'storybook-team',
		name: 'Publishing Checklist Preview',
		summary: 'A short summary.',
		description: '',
		published: '2026-01-01T00:00:00.000Z',
		updated: '2026-01-01T00:00:00.000Z',
		status,
		license: {
			id: 'MIT',
			name: 'MIT License',
		},
		downloads: 0,
		followers: 0,
		categories,
		additional_categories: [],
		loaders: [],
		mrpack_loaders: [],
		versions: [],
		link_urls: {},
		gallery: [],
		thread_id: 'storybook-thread',
		monetization_status: 'monetized',
		side_types_migration_review_status: 'reviewed',
		environment: ['unknown'],
	}
}

const tags = {
	categories: [],
	rejectedStatuses: ['rejected', 'withheld'],
	gameVersions: [],
	loaders: [],
}

const previewValues = {
	count: 3,
	domain: 'example.com',
	fullUrl: 'https://example.com/prohibited-link',
	languageCount: 12,
	length: 12,
	maxTagCount: 8,
	minChars: 50,
	projectType: 'mod',
	status: 'rejected',
	tagCount: 9,
	tags: '16x|32x',
	totalAvailableTags: 20,
	type: 'mod',
	url: 'https://example.com/prohibited-link',
	value: 'example',
}

const suggestionKinds = new Set<Labrinth.Projects.v3.NormalizedProjectNagKind>([
	'add-icon',
	'feature-gallery-image',
	'add-links',
	'add-links-server',
	'select-language',
	'select-tags',
	'check-disclosures',
])

const warningKinds = new Set<Labrinth.Projects.v3.NormalizedProjectNagKind>([
	'missing-alt-text',
	'verify-external-links',
	'too-many-languages',
	'too-many-tags',
	'multiple-resolution-tags',
	'moderator-feedback',
])

const previewNags = Object.keys(nagDefinitions).map((kind) => {
	const normalizedKind = kind as Labrinth.Projects.v3.NormalizedProjectNagKind
	const projectNagKind = kind.replaceAll('-', '_') as Labrinth.Projects.v3.ProjectNagKind
	const severity: Labrinth.Projects.v3.ProjectNagSeverity = suggestionKinds.has(normalizedKind)
		? 'suggestion'
		: warningKinds.has(normalizedKind)
			? 'warning'
			: 'required'
	return toProjectNag(
		{ kind: projectNagKind, severity, details: previewValues },
		previewValues.projectType,
	)
})

const everyNag: Nag[] = [
	...previewNags,
	{
		id: 'resubmit-for-review-preview',
		title: 'Resubmit for review',
		description: () =>
			"Your project has been rejected by Modrinth's staff. Address the moderation team's feedback before resubmitting.",
		status: 'special-submit-action',
		shouldShow: () => true,
		link: {
			path: 'moderation',
			title: 'Visit moderation page',
			shouldShow: () => true,
		},
	},
]

const meta = {
	title: 'Website/Moderation/PublishingChecklist',
	component: ModerationProjectNags,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="mx-auto w-full p-4" style="max-width: 1100px"><story /></div>',
		}),
	],
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		project: createProject('draft'),
		projectV3: createProjectV3('draft'),
		versions: [],
		collapsed: false,
		disableHorizontalScroll: true,
		routeName: 'type-project',
		tags,
	},
	render: (args) => ({
		components: { ModerationProjectNags },
		setup() {
			const collapsed = ref(args.collapsed)
			return { args, collapsed }
		},
		template: /* html */ `
			<ModerationProjectNags
				v-bind="args"
				:collapsed="collapsed"
				@toggle-collapsed="collapsed = !collapsed"
			/>
		`,
	}),
} satisfies Meta<typeof ModerationProjectNags>

export default meta
type Story = StoryObj<typeof meta>

export const EntirePublishingChecklist: Story = {
	parameters: {
		docs: {
			description: {
				story: 'The complete expanded checklist for an unfinished draft project.',
			},
		},
	},
}

export const EveryNag: Story = {
	args: {
		nags: everyNag,
	},
	parameters: {
		docs: {
			description: {
				story:
					'Every publishing-checklist validation nag plus the submit and resubmit actions, including combinations that cannot normally appear together.',
			},
		},
	},
}
