import type { Labrinth } from '@modrinth/api-client'
import { DatabaseIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY, injectProjectPageContext, injectTags } from '@modrinth/ui'
import { computed, defineAsyncComponent } from 'vue'

import {
	appComponent,
	dropdown,
	fix,
	group,
	md,
	option,
	stage,
	text,
	toggle,
} from '../../types/node'
import { requiresEnvironmentInfo } from '../../utils'

const ModrinthProjectSearch = defineAsyncComponent(
	async () => import('../../types/node/components/ModrinthProjectSearch.vue'),
)

const loaderLabels: Record<string, string> = {
	neoforge: 'NeoForge',
	liteloader: 'LiteLoader',
	datapack: 'Data Pack',
	resourcepack: 'Resource Pack',
}

function _formatLoaderLabel(id: string): string {
	return (
		loaderLabels[id] ??
		id
			.split(/[-_]/g)
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ')
	)
}

interface SearchedDependency {
	title?: string
	slug?: string
	projectType?: string
}

/**
 * The search picker (when used) always resolves to real Modrinth links; the plain name/link
 * fields stay as a manual fallback for a dependency that isn't on Modrinth, used only when
 * nothing was added via search.
 */
function formatDependencyLinks(state: Record<string, unknown>): string {
	const links: string[] = []

	const raw = state['dependency-search']
	if (typeof raw === 'string' && raw) {
		try {
			const parsed: unknown = JSON.parse(raw)
			if (Array.isArray(parsed)) {
				for (const dep of parsed as SearchedDependency[]) {
					if (dep?.title && dep.slug && dep.projectType) {
						links.push(`[${dep.title}](https://modrinth.com/${dep.projectType}/${dep.slug})`)
					}
				}
			}
		} catch {
			// Malformed JSON — ignore and fall back to the manual fields below.
		}
	}

	if (links.length === 0 && state['name'] && state['link']) {
		links.push(`[${state['name']}](${state['link']})`)
	}

	return links.join(', ')
}

export default function () {
	const { projectV3: project } = injectProjectPageContext()
	const { loaders: _loaders, gameVersions: _gameVersions } = injectTags()

	const _currentGameVersions = computed(
		() => (project.value.game_versions as string[] | undefined) ?? [],
	)

	return (
		stage('metadata', 'Metadata')
			.hint("Are there any issues with this project's metadata?")
			//TODO: update guidance here
			.guidance(
				'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
			)
			.icon(DatabaseIcon)
			.navigate('/versions')
			.shown(computed(() => !project.value?.minecraft_server))
			.children(
				() => {
					const env = project.value.environment ?? []
					if (env.length === 1) {
						return (
							<div class="markdown-body w-full">
								<strong>Environment:</strong>{' '}
								<code>{ENVIRONMENTS_COPY[env[0]].title.defaultMessage ?? env[0]}</code>
							</div>
						)
					}
					return (
						<div class="markdown-body w-full">
							<strong>Unique environments:</strong> {env.length}
							<br />
							<strong>Environments:</strong>{' '}
							<code>
								{env.map((id) => ENVIRONMENTS_COPY[id].title.defaultMessage ?? id).join(', ')}
							</code>
						</div>
					)
				},

				group().children(
					toggle('environment', 'Environment')
						.shown(computed(() => requiresEnvironmentInfo(project.value.project_types)))
						.suggestedStatus('flagged')
						.rawMessage(async (state) => {
							const correctEnvironment = state?.['correct-environment'] as string | undefined

							let correct = ''
							if (correctEnvironment === 'mixed')
								correct = await md('checklist/messages/metadata/environment/mixed')(state)
							else if (correctEnvironment)
								correct = await md('checklist/messages/metadata/environment/correction', () => ({
									SUGGESTED_ENVIRONMENT:
										ENVIRONMENTS_COPY[correctEnvironment]?.title.defaultMessage ??
										correctEnvironment,
								}))(state)

							return md('checklist/messages/metadata/environment/inaccurate', () => ({
								CORRECT: correct,
							}))(state)
						})
						.fix(
							fix().project((patch, state) => {
								const env = state['correct-environment'] as Labrinth.Projects.v3.Environment
								if (!env || state['correct-environment'] === 'mixed') return
								patch.environment = env
							}),
						)
						.children(
							group()
								.title('Correct Environment')
								.children(
									dropdown('correct-environment')
										.options(
											...(Object.keys(ENVIRONMENTS_COPY) as Labrinth.Projects.v3.Environment[])
												.filter((id) => id !== 'unknown')
												.map((id) => option(id, ENVIRONMENTS_COPY[id].title.defaultMessage ?? id)),
											option('mixed', 'Mixed'),
										)
										.none('Unknown'),
								),
						),

					toggle('dependencies', 'Dependencies')
						.suggestedStatus('flagged')
						.message((state) => ({
							DEPENDENCIES: formatDependencyLinks(state),
						}))
						.children(
							appComponent('dependency-search', ModrinthProjectSearch)
								.title('On Modrinth: ')
								.props(() => ({ multiple: true })),
							(
								<h2>External Dependencies</h2>
							), // TODO: ADD FUNCTION FOR ADDING MULTIPLE ENTRIES
							text('name').title('Dependency name'),
							text('link').title('Dependency link'),
						),

					toggle('game-versions', 'Game Versions')
						.suggestedStatus('flagged')
						.message((state) => ({
							SPECIFICS: state.specifics,
						}))
						.children(text('specifics').title('More details about the game versions issue?')),

					toggle('loaders', 'Loaders')
						.suggestedStatus('rejected')
						.shown(!project.value.minecraft_server)
						.message((state) => ({
							SPECIFICS: state.specifics,
						}))
						.children(text('specifics').title('More details about the loaders issue?')),

					toggle('license', 'Licensing').suggestedStatus('flagged').message(),
					// toggle('loader', 'Loaders (WIP)')
					// 	.suggestedStatus('flagged')
					// 	.rawMessage(async (state) => {
					// 		const selected =
					// 			state.loaders instanceof Set ? state.loaders : new Set(project.value.loaders)
					// 		const current = new Set(project.value.loaders)
					// 		const isCorrected =
					// 			selected.size !== current.size || [...selected].some((id) => !current.has(id))
					//
					// 		let correct = ''
					// 		if (isCorrected) {
					// 			const list = [...selected].map((id) => formatLoaderLabel(id)).join(', ')
					// 			correct = await md('checklist/messages/metadata/loader/correction', () => ({
					// 				LOADERS: list || 'none',
					// 			}))(state)
					// 		}
					//
					// 		return md('checklist/messages/metadata/loader/inaccurate', () => ({
					// 			CORRECT: correct,
					// 		}))(state)
					// 	})
					// 	.fix(
					// 		fix().project((patch, state) => {
					// 			const selected =
					// 				state.loaders instanceof Set ? state.loaders : new Set(project.value.loaders)
					// 			const next = [...selected]
					// 			const current = project.value.loaders
					// 			if (next.length === current.length && next.every((id) => current.includes(id)))
					// 				return
					// 			patch.loaders = next
					// 		}),
					// 	)
					// 	.children(
					// 		appComponent('loaders', 'loader-picker')
					// 			.valueKind('set')
					// 			.initial(() => new Set(project.value.loaders))
					// 			.props((ctx) => ({
					// 				loaders: loaders.value,
					// 				toggleLoader: ctx.toggleSetValue,
					// 			})),
					// 	),
					//
					// toggle('game-version', 'Game Versions (WIP)')
					// 	.suggestedStatus('flagged')
					// 	.rawMessage(async (state) => {
					// 		const selected =
					// 			state['game-versions'] instanceof Set
					// 				? state['game-versions']
					// 				: new Set(currentGameVersions.value)
					// 		const current = new Set(currentGameVersions.value)
					// 		const isCorrected =
					// 			selected.size !== current.size || [...selected].some((id) => !current.has(id))
					//
					// 		let correct = ''
					// 		if (isCorrected) {
					// 			const list = [...selected].join(', ')
					// 			correct = await md('checklist/messages/metadata/game-version/correction', () => ({
					// 				GAME_VERSIONS: list || 'none',
					// 			}))(state)
					// 		}
					//
					// 		return md('checklist/messages/metadata/game-version/inaccurate', () => ({
					// 			CORRECT: correct,
					// 		}))(state)
					// 	})
					// 	.fix(
					// 		fix().project((patch, state) => {
					// 			const selected =
					// 				state['game-versions'] instanceof Set
					// 					? state['game-versions']
					// 					: new Set(currentGameVersions.value)
					// 			const next = [...selected]
					// 			const current = currentGameVersions.value
					// 			if (next.length === current.length && next.every((id) => current.includes(id)))
					// 				return
					// 			patch.game_versions = next
					// 		}),
					// 	)
					// 	.children(
					// 		appComponent('game-versions', 'game-version-picker')
					// 			.valueKind('set')
					// 			.initial(() => new Set(currentGameVersions.value))
					// 			.props(() => ({
					// 				gameVersions: gameVersions.value,
					// 				noHeader: true,
					// 			})),
					// 	),
				),
			)
	)
}
