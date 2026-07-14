import type { Labrinth } from '@modrinth/api-client'
import { TagIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY } from '@modrinth/ui'

import {
	action,
	dropdown,
	fix,
	group,
	label,
	md,
	option,
	stage,
	stageFn,
	toggle,
} from '../../types/node'

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

export default stageFn((project) =>
	stage('metadata', 'Metadata')
		.hint("Are there any issues with this project's metadata?")
		//TODO: update guidance here
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
		)
		.icon(TagIcon)
		.navigate('/settings/versions')
		.shown(!project?.minecraft_server)
		.children(
			label(
				md(
					`checklist/text/metadata/environment/${(project.environment?.length ?? 0) === 1 ? 'single' : 'multiple'}`,
				),
			),

			group().children(
				toggle('environment', 'Environment')
					.shown(project.project_types.includes('mod') || project.project_types.includes('modpack'))
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('low')
							.message(async (ctx) => {
								const correct_environment = ctx?.state.correct_environment as string | undefined

								//TODO: this should be in markdown files
								let correct_output = ''
								if (correct_environment)
									correct_output = `It looks like this project is probably "${ENVIRONMENTS_COPY[String(ctx?.state.correct_environment)].title.default_message}"`
								else if (correct_environment === 'Mixed')
									correct_output = `It looks like some %PROJECT_VERSIONS_FLINK% of your project should have unique environments from other versions, please ensure *each version* is set correctly.`

								return md('checklist/messages/environment/inaccurate', () => ({
									CORRECT: correct_output,
								}))(ctx)
							})
							.fix(
								fix().project((patch, ctx) => {
									const env = ctx.state.correct_environment as Labrinth.Projects.v3.Environment
									if (!env) return
									patch.environment = env
								}),
							),
					)
					.children(
						group()
							.title('Correct Environment')
							.children(
								dropdown('correct_environment')
									.children(
										...(Object.keys(ENVIRONMENTS_COPY) as Labrinth.Projects.v3.Environment[])
											.filter((id) => id !== 'unknown')
											.map((id) => option(id, ENVIRONMENTS_COPY[id].title.defaultMessage ?? id)),
									)
									.none('Unknown'),
							),
					),

				toggle('loader', `Loader${project.loaders.length > 1 ? "s" : ""}`).children(
					group()
						.title('Loader Issues?')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('medium')
								.message(async (ctx) => {
									const header = await md('checklist/messages/metadata/loader/incorrect')(ctx)
									const selected = ctx.state.loaders
									if (selected instanceof Set && selected.size > 0) {
										const list = [...selected].map((id) => `- ${formatLoaderLabel(id)}`).join('\n')
										return `${header}\n${list}`
									}
									return header
								}),
						)
						.children(
							toggle('incorrect', 'Incorrect').children(
								group('loaders')
									.title('Incorrect Loaders')
									.multiSelect()
									.children(...project.loaders.map((id) => option(id, formatLoaderLabel(id)))),
							),
              //TODO: missing?
						),
				),
			),
		),
)
