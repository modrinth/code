import { CopyrightIcon } from '@modrinth/assets'

import type { NodeContext } from '../../types/node'
import { button, group, markdown, mdMsg, stage, text, toggle } from '../../types/node'

function isServerModpack({ project }: NodeContext): boolean {
	return (
		!!project.minecraft_server &&
		project.minecraft_java_server?.content?.kind === 'modpack' &&
		(project.minecraft_java_server?.content as Record<string, unknown>)?.project_id === project.id
	)
}

export default stage(
	'reupload',
	'Reupload',
	'Does the author have proper permissions to post this project?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080d1a0a2cda3ff2ce997',
	[
		group().children(
			button('reupload', 'Re-upload')
				.shown(({ project }) => !project.minecraft_server)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(
					mdMsg('reupload/reupload', (ctx) => ({
						ORIGINAL_PROJECT: ctx.state.original_project,
						ORIGINAL_AUTHOR: ctx.state.original_author,
					})),
				)
				.children(
					text('original_project', 'Original project title').required(),
					text('original_author', 'Original project author').required(),
				),

			button('unclear_fork', 'Unclear Fork')
				.shown(({ project }) => !project.minecraft_server)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('reupload/fork')),

			button('insufficient_fork', 'Insufficient Fork')
				.shown(({ project }) => !project.minecraft_server)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('reupload/insufficient_fork')),

			button('request_proof', 'Proof of permissions')
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('reupload/proof_of_permissions')),

			button('identity_verification', 'Verify Identity')
				.shown(({ project }) => !project.minecraft_server)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(
					mdMsg(
						'reupload/identity-verification/identity_verification',
						(ctx) => ({ PLATFORM: ctx.state.platform }),
					),
				)
				.children(text('platform', 'Where else can the project be found?').required()),

			button('identity_verification_server', 'Verify Identity')
				.shown(({ project }) => !!project.minecraft_server)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(
					mdMsg(
						'reupload/identity-verification/identity_verification-server',
						(ctx) => ({ CONTACT: ctx.state.contact }),
					),
				)
				.children(text('contact', 'Known public contact method').required()),

			button('request_proof_server', 'Reuploaded pack')
				.shown(isServerModpack)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('reupload/custom_server/custom_server_permissions')),

			button('custom_pack_verification', 'Override verification')
				.shown(isServerModpack)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('reupload/custom_server/custom_server_overrides-verification'))
				.children(
					toggle('list', 'List overrides?')
						.weight(1101)
						.message(
							mdMsg(
								'reupload/custom_server/custom_server_overrides-verification-list',
								(ctx) => ({ OVERRIDES: ctx.state.overrides }),
							),
						)
						.children(markdown('overrides', 'Add list of overrides.')),
				),

			button('custom_pack_prohibited', 'Forbidden Overrides')
				.shown(isServerModpack)
				.weight(1100)
				.suggestedStatus('rejected')
				.severity('high')
				.message(
					mdMsg('reupload/custom_server/custom_server_overrides-prohibited', (ctx) => ({
						OVERRIDES: ctx.state.overrides,
					})),
				)
				.children(markdown('overrides', 'Forbidden overrides list').required()),
		),
	],
	{ icon: CopyrightIcon },
)
