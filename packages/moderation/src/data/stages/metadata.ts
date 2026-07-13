import type { Labrinth } from '@modrinth/api-client'
import { TagIcon } from '@modrinth/assets'

import { action, check,fix, label, md, select, stage, toggle } from '../../types/node'
import { formatEnvironments } from '../../utils'

const textSingle = md('checklist/text/environment/environment')
const textMultiple = md('checklist/text/environment/environment-multiple')

export default stage('metadata', 'Metadata',)
	.hint("Is this project's metadata accurate?")
	//TODO: update guidance here
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb')
	.icon(TagIcon)
	.navigate('/settings/versions')
	.shown(({ project }) => !project?.minecraft_server)
	.children(
		label((ctx) =>
			(ctx.project.environment?.length ?? 0) === 1 ? textSingle(ctx) : textMultiple(ctx),
		),

		toggle('environment', 'Environment')
			.shown(
				({ project }) =>
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
				select('correct_environment', 'Correct environment')
					.children(
						check('client_only', 'Client-side Only'),

						check('server_only', 'Server-side + Singleplayer'),
						check('dedicated_server_only', 'Dedicated Server Only'),

						check('client_and_server', 'Required on Both'),
						check('server_only_client_optional', 'Client Optional'),
						check('client_only_server_optional', 'Server Optional'),
						check('client_or_server', 'Client or Server'),
						check('client_or_server_prefers_both', 'Client or Server, Prefers Both'),

						check('singleplayer_only', 'Singleplayer'),

						check('mixed', 'Mixed'),
					)
					.dropdown('Unknown'),
			),
	)
