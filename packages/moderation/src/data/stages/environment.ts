import type { Labrinth } from '@modrinth/api-client'
import { GlobeIcon } from '@modrinth/assets'

import { action, button, fix, group, label, md, select, stage, toggle } from '../../types/node'

const textSingle = md('checklist/text/environment/environment')
const textMultiple = md('checklist/text/environment/environment-multiple')

export default stage(
	'environment',
	'Environment',
	"Is the project's environment information accurate?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
)
	.icon(GlobeIcon)
	.navigate('/settings/versions')
	.shown(({ project }) => !project?.minecraft_server)
	.children(
		label((ctx) =>
			(ctx.project.environment?.length ?? 0) === 1 ? textSingle(ctx) : textMultiple(ctx),
		),

		group().children(
			button('inaccurate', 'Inaccurate')
				.shown(
					({ project }) =>
						project.project_types.includes('mod') || project.project_types.includes('modpack'),
				)
				.action(
					action()
						.weight(800)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/environment/inaccurate'))
						.fix(
							fix().version((version, ctx) => {
								const env = ctx.state.correct_environment as
									| Labrinth.Projects.v3.Environment
									| undefined
								if (!env) return false
								version.environment = env
							}),
						),
				)
				.children(
					select('correct_environment', 'Correct environment')
						.children(
							toggle('client_only', 'Client-side Only'),

							toggle('dedicated_server_only', 'Dedicated Server Only'),
							toggle('server_only', 'Server-side + Singleplayer'),

							toggle('client_and_server', 'Required on Both'),
							toggle('server_only_client_optional', 'Client Optional'),
							toggle('client_only_server_optional', 'Server Optional'),
							toggle('client_or_server', 'Client or Server'),
							toggle('client_or_server_prefers_both', 'Client or Server, Prefers Both'),

							toggle('singleplayer_only', 'Singleplayer'),
						).dropdown('Unknown'),
				),
		),
	)
