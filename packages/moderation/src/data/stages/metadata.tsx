import type { Labrinth } from '@modrinth/api-client'
import { DatabaseIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY, injectProjectPageContext, injectTags } from '@modrinth/ui'
import { computed } from 'vue'

import { dropdown, fix, group, md, option, stage, toggle } from '../../types/node'
import { requiresEnvironmentInfo } from '../../utils'

const loaderLabels: Record<string, string> = {
	neoforge: 'NeoForge',
	liteloader: 'LiteLoader',
	datapack: 'Data Pack',
	resourcepack: 'Resource Pack',
}

function formatLoaderLabel(id: string): string {
	return (
		loaderLabels[id] ??
		id
			.split(/[-_]/g)
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ')
	)
}

export default function () {
	const { projectV3: project } = injectProjectPageContext()
	const { loaders } = injectTags()

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
							<br/>
							<strong>Environments:</strong> <code>{env.map(id => ENVIRONMENTS_COPY[id].title.defaultMessage ?? id).join(', ')}</code>
						</div>
					)
				},

				group().children(
					toggle('environment', 'Environment')
						.shown(computed(() => requiresEnvironmentInfo(project.value.project_types)))
						.suggestedStatus('flagged')
						.severity('low')
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
										.children(
											...(Object.keys(ENVIRONMENTS_COPY) as Labrinth.Projects.v3.Environment[])
												.filter((id) => id !== 'unknown')
												.map((id) => option(id, ENVIRONMENTS_COPY[id].title.defaultMessage ?? id)),
											option('mixed', 'Mixed'),
										)
										.none('Unknown'),
								),
						),
					// TODO: chyz, fix pls (make into single set of buttons where current loaders start selected and non current start non selected
					//					toggle('loader', `Loader${project.value.loaders.length > 1 ? 's' : ''}`).children(
					//						group()
					//							.title('Loader Issues?')
					//							.action(
					//								action()
					//									.suggestedStatus('flagged')
					//									.severity('medium')
					//									.message(async (state) => {
					//										//TODO: chyz
					//										//TODO: coolbot this one is a bit of a doozy
					//										const header = await md('checklist/messages/metadata/loader/incorrect')(state)
					//										const selected = state.loaders
					//										if (selected instanceof Set && selected.size > 0) {
					//											const list = [...selected]
					//												.map((id) => `- ${formatLoaderLabel(id)}`)
					//												.join('\n')
					//											return `${header}\n${list}`
					//										}
					//										return header
					//									}),
					//							)
					//							.children(
					//								toggle('incorrect', 'Incorrect').children(
					//									group()
					//										.title('Incorrect Loaders')
					//										.multiSelect('loaders')
					//										.children(
					//											...project.value.loaders.map((id) => option(id, formatLoaderLabel(id))),
					//										),
					//								),
					// TODO: chyz, this should be the same interface as incorrect, as a corrections scheme, with selected loaders default on.
					//								toggle('missing', 'Missing').children(
					//									group()
					//										.title('Missing Loaders')
					//										.multiSelect('loaders')
					//										.children(
					//											...(() => {
					//												//TODO: chyz maybe this can be done better
					//												// (plugin loaders and datapack are marked as valid for mods which makes this suck)
					//												const existingTypes = new Set(
					//													loaders.value
					//														.filter((l) => project.value.loaders.includes(l.name))
					//														.flatMap((l) => l.supported_project_types),
					//												)
					//												const referenceTypes =
					//													existingTypes.size > 0
					//														? existingTypes
					//														: new Set(project.value.project_types)
					//												return loaders.value
					//													.filter(
					//														(loader) =>
					//															loader.supported_project_types.every((t) => referenceTypes.has(t)) &&
					//															!project.value.loaders.includes(loader.name),
					//													)
					//													.map((loader) => option(loader.name, formatLoaderLabel(loader.name)))
					//											})(),
					//									)/
					// ),
					//							),
					//					),
				),
			)
	)
}
