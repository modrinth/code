import type { Labrinth } from '@modrinth/api-client'
import { TagIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY } from '@modrinth/ui'

import { action, dropdown, fix, group, label, md, option, stage, stageFn, toggle } from '../../types/node'
import { formatEnvironments } from '../../utils'

export default stageFn((project) => stage('metadata', 'Metadata')
	.hint("Are there any issues with this project's metadata?")
	//TODO: update guidance here
	.guidance(
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
	)
	.icon(TagIcon)
	.navigate('/settings/versions')
	.shown(!project?.minecraft_server)
	.children(
		label(md(`checklist/text/metadata/environment/${(project.environment?.length ?? 0) === 1 ? 'single' : 'multiple'}`)),

		toggle('environment', 'Environment')
			.shown(
				project.project_types.includes('mod') || project.project_types.includes('modpack'),
			)
			.action(
				action()
					.suggestedStatus('flagged')
					.severity('low')
					.message(async (ctx) => {
						const correct_environment = ctx?.state.correct_environment as string | undefined

						let correct_output = ''
						if (correct_environment)
							correct_output = `It looks like this project is probably "${formatEnvironments(String(ctx?.state.correct_environment))}"`
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
	),
)
