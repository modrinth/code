import { ScaleIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, stage, text, toggle } from '../../types/node'

//TODO chyz
//TODO coolbot needs discussion
export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('post-approval', 'Post-Approval')
		.hint('Issue warnings, notices, or takedowns?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080c5a13cda0b1e4ae9ed',
		)
		.icon(ScaleIcon)
		.shown(computed(() => project.value.status === 'approved'))
		.children(
			group().children(
				toggle('issue_warning', 'Issue warning').action(
					action().suggestedStatus('approved').severity('low').message(),
				),

				toggle('missed_deadline', 'Missed due date')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('high')
							.message((state) => ({
								STATUS: state.status,
							})),
					)
					.children(
						//TODO: chyz this shouldn't need to be provided by moderator
						text('status').title('What status is the project being set to?').required(),
					),

				toggle('metadata_issue', 'Incorrect metadata')
					.action(action().suggestedStatus('approved').severity('low').message())
					.children(
						toggle('dependencies', 'Missing Dependencies')
							.action(
								action()
									.severity('low')
									.message((state) => ({
										DEPENDENCY_NAME: state.dependency_name,
										DEPENDENCY_LINK: state.dependency_link,
									})),
							)
							.children(
								text('dependency_name').title('Dependency name').required(),
								text('dependency_link').title('Dependency link').required(),
							),

						toggle('mc_versions', 'Game versions')
							.action(
								action()
									.severity('low')
									.message((state) => ({
										SPECIFICS: state.specifics,
									})),
							)
							.children(text('specifics').title('More details about the game versions issue?')),

						toggle('loaders', 'Loaders')
							.action(
								action()
									.severity('low')
									.message((state) => ({
										SPECIFICS: state.specifics,
									})),
							)
							.children(text('specifics').title('More details about the loaders issue?')),

						toggle('license', 'Inconsistent Licensing').action(action().severity('low').message()),
					),
			),
		)
}
