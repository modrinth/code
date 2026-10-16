import type { Labrinth } from '@modrinth/api-client'
import type { ReviewStateValue } from '@modrinth/moderation'
import { reviewPanels } from '@modrinth/moderation/src/data/issues'
import type {
	Panel,
	PanelNode,
	PanelRegistration,
	ReviewContext,
} from '@modrinth/moderation/src/data/issues/component-builders/types'
import { type ProjectPageContext, provideProjectPageContext } from '@modrinth/ui'
import { reactive, ref } from 'vue'

import { createReviewMessages } from '../../../../providers/project-review/review-messages'
import { createReviewPanels } from '../../../../providers/project-review/review-panels'
import type { createReviewSession } from '../../../../providers/project-review/review-session'

function catalogueNodes(nodes: PanelNode[]): PanelNode[] {
	return nodes.map((node): PanelNode => {
		if (node.type === 'section')
			return { ...node, shown: undefined, children: catalogueNodes(node.children) }
		if (node.type !== 'select') return { ...node, shown: undefined, disabled: undefined }
		const options = node.options
		return {
			...node,
			shown: undefined,
			disabled: undefined,
			options: (context) =>
				(typeof options === 'function' ? options(context) : options).map((option) => ({
					...option,
					shown: undefined,
					disabled: undefined,
				})),
		}
	})
}

function cataloguePanel(panel: Panel): Panel {
	return { ...panel, shown: undefined, children: catalogueNodes(panel.children) }
}

/** Expose every registered option, including mutually exclusive project and selection states. */
const definitions = Object.fromEntries(
	Object.entries(reviewPanels).map(([key, registration]): [string, PanelRegistration] => {
		if (Array.isArray(registration))
			return [key, registration.map((item) => ({ ...item, panel: cataloguePanel(item.panel) }))]
		if ('type' in registration)
			return [
				key,
				{
					...registration,
					items: (context) =>
						registration.items(context).map((item) => ({
							...item,
							panel: cataloguePanel(item.panel),
						})),
				},
			]
		return [key, cataloguePanel(registration)]
	}),
)

const project: Labrinth.Projects.v3.Project = {
	id: 'storybook-issues',
	slug: 'storybook-issues',
	project_types: ['modpack'],
	games: ['minecraft:java-edition'],
	team_id: 'storybook-team',
	name: 'Example project',
	summary: 'An example project for the issue catalogue.',
	description: '',
	published: '2026-01-01T00:00:00.000Z',
	updated: '2026-01-01T00:00:00.000Z',
	status: 'processing',
	license: { id: 'MIT', name: 'MIT License' },
	downloads: 0,
	followers: 0,
	categories: ['adventure', 'optimization', '32x'],
	additional_categories: [],
	loaders: ['fabric'],
	mrpack_loaders: ['fabric'],
	versions: [],
	link_urls: {},
	gallery: [1, 2].map((index) => ({
		id: index,
		url: 'https://cdn.modrinth.com/modrinth-new.png',
		raw_url: 'https://cdn.modrinth.com/modrinth-new.png',
		featured: index === 1,
		name: `Example image ${index}`,
		created: '2026-01-01T00:00:00.000Z',
		ordering: index,
	})),
	thread_id: 'storybook-thread',
	monetization_status: 'monetized',
	side_types_migration_review_status: 'reviewed',
	environment: ['unknown'],
}

const legacyProject: Labrinth.Projects.v2.Project = {
	id: project.id,
	slug: project.slug,
	project_type: 'modpack',
	actualProjectType: 'modpack',
	team: project.team_id,
	organization: null,
	title: project.name,
	description: project.summary,
	body: project.description,
	published: project.published,
	updated: project.updated,
	status: project.status,
	license: project.license,
	client_side: 'required',
	server_side: 'optional',
	downloads: 0,
	followers: 0,
	categories: project.categories,
	additional_categories: [],
	game_versions: [],
	donation_urls: [],
	loaders: project.loaders,
	versions: [],
	gallery: [],
	thread_id: project.thread_id,
	monetization_status: project.monetization_status,
}

export function provideIssueCatalogueProject() {
	const context: ProjectPageContext = {
		projectV2: ref(legacyProject),
		projectV3: ref(project),
		currentMember: ref(null),
		allMembers: ref([]),
		organization: ref(null),
		projectValidation: ref(null),
		projectValidationLoading: ref(false),
		projectLinksNetworkValidationLoading: ref(false),
		versions: ref([]),
		versionsLoading: ref(false),
		versionsLoaded: ref(true),
		dependencies: ref(null),
		dependenciesLoading: ref(false),
		thread: ref(null),
		cdnDownloadReason: ref('standalone'),
		invalidate: async () => {},
		refreshProjectValidation: async () => null,
		loadVersions: () => {},
		loadDependencies: () => {},
		patchProject: async () => false,
		patchProjectV3: async () => false,
		patchIcon: async () => false,
		setProcessing: async () => false,
		createGalleryItem: async () => false,
		editGalleryItem: async () => false,
		deleteGalleryItem: async () => false,
	}
	provideProjectPageContext(context)
}

export function createIssueCatalogue() {
	const state = reactive<Record<string, Record<string, ReviewStateValue>>>({})
	const session: ReturnType<typeof createReviewSession> = {
		read: (_, scope) => state[scope] ?? {},
		write: (_, scope, id, value) => {
			state[scope] = { ...state[scope], [id]: value }
		},
		readProject: () => state,
		clearProject: () => {
			for (const key of Object.keys(state)) Reflect.deleteProperty(state, key)
		},
	}
	const data: Pick<
		ReviewContext,
		| 'projectV2'
		| 'versionsV3'
		| 'versionsV2'
		| 'disclosures'
		| 'members'
		| 'organization'
		| 'organizationMembers'
		| 'wasReviewed'
		| 'hasPreviousStructuredIssues'
		| 'permissions'
	> & { previousLinks: ReadonlyMap<string, string> } = {
		projectV2: legacyProject,
		versionsV3: [],
		versionsV2: [],
		disclosures: [],
		members: [],
		organization: null,
		organizationMembers: [],
		wasReviewed: false,
		hasPreviousStructuredIssues: false,
		permissions: { groups: [], unresolvedCount: 1, loaded: true, loading: false, error: null },
		previousLinks: new Map(),
	}
	const current = ref(project)
	const panels = createReviewPanels(current, session, ref(data), definitions)
	const messages = createReviewMessages(current, ref(legacyProject), panels)
	return {
		...panels,
		message: messages.issueMessage,
		reset: () => session.clearProject(project.id),
	}
}
