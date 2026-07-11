import { ScaleIcon } from '@modrinth/assets'

import { button, group, mdMsg, stage, text } from '../../types/node'

export default stage(
	'post-approval',
	'Post-Approval',
	'Issue warnings, notices, or takedowns?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080c5a13cda0b1e4ae9ed',
	{
		icon: ScaleIcon,
		shown: (project) => project.status === 'approved',
	},
	[
		group().children(
			button('issue_warning', 'Issue warning')
				.weight(3000)
				.suggestedStatus('approved')
				.severity('low')
				.message(mdMsg('post-approval/issue-warning')),

			button('missed_deadline', 'Missed due date')
				.weight(-999)
				.suggestedStatus('flagged')
				.severity('high')
				.message(mdMsg('post-approval/missed-deadline', (ctx) => ({ STATUS: ctx.state.status })))
				.children(text('status', 'What status is the project being set to?').required()),

			button('metadata_issue', 'Incorrect metadata')
				.weight(0)
				.suggestedStatus('approved')
				.severity('low')
				.message(mdMsg('post-approval/metadata-issue'))
				.children(
					button('dependencies', 'Missing Dependencies')
						.weight(1)
						.severity('low')
						.message(mdMsg('misc-metadata/dependencies', (ctx) => ({
							DEPENDENCY_NAME: ctx.state.dependency_name,
							DEPENDENCY_LINK: ctx.state.dependency_link,
						})))
						.children(
							text('dependency_name', 'Dependency name').required(),
							text('dependency_link', 'Dependency link').required(),
						),

					button('mc_versions', 'Game versions')
						.weight(2)
						.severity('low')
						.message(mdMsg('misc-metadata/mc-versions', (ctx) => ({ SPECIFICS: ctx.state.specifics })))
						.children(text('specifics', 'More details about the game versions issue?')),

					button('loaders', 'Loaders')
						.weight(3)
						.severity('low')
						.message(mdMsg('misc-metadata/loaders', (ctx) => ({ SPECIFICS: ctx.state.specifics })))
						.children(text('specifics', 'More details about the loaders issue?')),

					button('license', 'Inconsistent Licensing')
						.weight(4)
						.severity('low')
						.message(mdMsg('misc-metadata/inconsistent-license')),
				),
		),
	],
)
