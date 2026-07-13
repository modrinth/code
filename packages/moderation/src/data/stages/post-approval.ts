import { ScaleIcon } from '@modrinth/assets'

import { action, toggle, group, md, stage, text } from '../../types/node'

export default stage('post-approval', 'Post-Approval')
	.hint('Issue warnings, notices, or takedowns?')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080c5a13cda0b1e4ae9ed')
	.icon(ScaleIcon)
	.shown(({ projectV2 }) => projectV2.status === 'approved')
	.children(
		group().children(
			toggle('issue_warning', 'Issue warning')
				.action(
					action()
						.suggestedStatus('approved')
						.severity('low')
						.message(md('checklist/messages/post-approval/issue-warning')),
				),

			toggle('missed_deadline', 'Missed due date')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('high')
						.message(md('checklist/messages/post-approval/missed-deadline', (ctx) => ({ STATUS: ctx.state.status }))),
				)
				.children(text('status', 'What status is the project being set to?').required()),

			toggle('metadata_issue', 'Incorrect metadata')
				.action(
					action()
						.suggestedStatus('approved')
						.severity('low')
						.message(md('checklist/messages/post-approval/metadata-issue')),
				)
				.children(
					toggle('dependencies', 'Missing Dependencies')
						.action(
							action()
								.severity('low')
								.message(md('checklist/messages/misc-metadata/dependencies', (ctx) => ({
									DEPENDENCY_NAME: ctx.state.dependency_name,
									DEPENDENCY_LINK: ctx.state.dependency_link,
								}))),
						)
						.children(
							text('dependency_name', 'Dependency name').required(),
							text('dependency_link', 'Dependency link').required(),
						),

					toggle('mc_versions', 'Game versions')
						.action(
							action()
								.severity('low')
								.message(md('checklist/messages/misc-metadata/mc-versions', (ctx) => ({ SPECIFICS: ctx.state.specifics }))),
						)
						.children(text('specifics', 'More details about the game versions issue?')),

					toggle('loaders', 'Loaders')
						.action(
							action()
								.severity('low')
								.message(md('checklist/messages/misc-metadata/loaders', (ctx) => ({ SPECIFICS: ctx.state.specifics }))),
						)
						.children(text('specifics', 'More details about the loaders issue?')),

					toggle('license', 'Inconsistent Licensing')
						.action(
							action()
								.severity('low')
								.message(md('checklist/messages/misc-metadata/inconsistent-license')),
						),
				),
		),
	)
