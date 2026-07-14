import { CopyrightIcon } from '@modrinth/assets'

import type { NodeContext } from '../../types/node'
import { action, check, group, markdown, md, stage, stageFn, text, toggle } from '../../types/node'

function isServerModpack({ project }: NodeContext): boolean {
	return (
		!!project.minecraft_server &&
		project.minecraft_java_server?.content?.kind === 'modpack' &&
		project.minecraft_java_server?.content?.project_id === project.id
	)
}

export default stageFn((project) => stage('reupload', 'Reupload')
	.hint('Does the author have proper permissions to post this project?')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080d1a0a2cda3ff2ce997')
	.icon(CopyrightIcon)
	.children(
		group().children(
			toggle('reupload', 'Re-upload')
				.shown(!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(
							md('checklist/messages/reupload/reupload', (ctx) => ({
								ORIGINAL_PROJECT: ctx.state.original_project,
								ORIGINAL_AUTHOR: ctx.state.original_author,
							})),
						),
				)
				.children(
					text('original_project', 'Original project title').required(),
					text('original_author', 'Original project author').required(),
				),

			toggle('unclear_fork', 'Unclear Fork')
				.shown(!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/reupload/fork')),
				),

			toggle('insufficient_fork', 'Insufficient Fork')
				.shown(!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/reupload/insufficient_fork')),
				),

			toggle('request_proof', 'Proof of permissions')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/reupload/proof_of_permissions')),
				),

			toggle('identity_verification', 'Verify Identity')
				.shown(!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(
							md('checklist/messages/reupload/identity-verification/identity_verification', (ctx) => ({
								PLATFORM: ctx.state.platform,
							})),
						),
				)
				.children(text('platform', 'Where else can the project be found?').required()),

			toggle('identity_verification_server', 'Verify Identity')
				.shown(!!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(
							md('checklist/messages/reupload/identity-verification/identity_verification-server', (ctx) => ({
								CONTACT: ctx.state.contact,
							})),
						),
				)
				.children(text('contact', 'Known public contact method').required()),

			toggle('request_proof_server', 'Reuploaded pack')
				.shown(isServerModpack)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/reupload/custom_server/custom_server_permissions')),
				),

			toggle('custom_pack_verification', 'Override verification')
				.shown(isServerModpack)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/reupload/custom_server/custom_server_overrides-verification')),
				)
				.children(
					check('list', 'List overrides?')
						.action(
							action()
								.message(
									md('checklist/messages/reupload/custom_server/custom_server_overrides-verification-list', (ctx) => ({
										OVERRIDES: ctx.state.overrides,
									})),
								),
						)
						.children(markdown('overrides', 'Add list of overrides.')),
				),

			toggle('custom_pack_prohibited', 'Forbidden Overrides')
				.shown(isServerModpack)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(
							md('checklist/messages/reupload/custom_server/custom_server_overrides-prohibited', (ctx) => ({
								OVERRIDES: ctx.state.overrides,
							})),
						),
				)
				.children(markdown('overrides', 'Forbidden overrides list').required()),
		),
	),
)
