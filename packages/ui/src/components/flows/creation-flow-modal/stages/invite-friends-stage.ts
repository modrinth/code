import { CheckIcon } from '@modrinth/assets'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import InviteFriendsStage from '../components/InviteFriendsStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'invite-friends',
	title: (ctx) =>
		ctx.formatMessage({
			id: 'servers.setup.onboarding.invite.title',
			defaultMessage: 'Invite players to join',
		}),
	stageContent: markRaw(InviteFriendsStage),
	skip: (ctx) => ctx.flowType !== 'server-onboarding',
	nonProgressStage: true,
	hideHeader: true,
	mergeHeader: true,
	beforeHide: (ctx) => {
		if (ctx.inviteCompleted.value) return true
		ctx.completeInvite()
		return false
	},
	leftButtonConfig: null,
	rightButtonConfig: (ctx) => ({
		label: ctx.formatMessage({
			id: 'servers.setup.onboarding.invite.done',
			defaultMessage: 'Done',
		}),
		icon: CheckIcon,
		iconPosition: 'before',
		onClick: ctx.completeInvite,
	}),
	maxWidth: '448px',
	actionsTopMargin: 'sm',
}
