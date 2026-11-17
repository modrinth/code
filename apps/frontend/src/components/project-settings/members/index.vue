<script setup lang="ts">
import { UserPlusIcon, UserXIcon } from '@modrinth/assets'
import {
	Button,
	commonProjectSettingsMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	SettingsOptionCard,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef, watch } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { removeSelfFromTeam } from '~/helpers/teams.js'

import SettingsSection from '../settings-section.vue'
import InviteDialog from './invite-modal.vue'
import MemberTable from './member-table.vue'
import { hasPermission, projectPermissions } from './permissions'
import RevenueSummary from './revenue-summary.vue'
import type { MemberChanges, MemberDraft } from './types'
import { useMemberErrorNotification } from './use-member-error-notification'

const {
	projectV2: project,
	projectV3,
	organization,
	allMembers,
	currentMember,
	invalidate,
} = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const notifyError = useMemberErrorNotification()
const loading = useLoading()

const messages = defineMessages({
	addMember: { id: 'project.settings.members.add-member', defaultMessage: 'Add member' },
	organizationMembers: {
		id: 'project.settings.members.organization-members',
		defaultMessage: 'Organization members',
	},
	managedBy: {
		id: 'project.settings.members.managed-by-organization',
		defaultMessage:
			'This project is managed by <organization>{name}</organization>. Member defaults are set in the organization settings; override them below.',
	},
	leave: { id: 'project.settings.members.leave.title', defaultMessage: 'Leave project' },
	leaveDescription: {
		id: 'project.settings.members.leave.description',
		defaultMessage: 'Remove yourself as a member of this project.',
	},
	removed: { id: 'project.settings.members.removed.title', defaultMessage: 'Member removed' },
	removedDescription: {
		id: 'project.settings.members.removed.description',
		defaultMessage: "Your project's member has been removed.",
	},
	updated: { id: 'project.settings.members.updated.title', defaultMessage: 'Member(s) updated' },
	updatedDescription: {
		id: 'project.settings.members.updated.description',
		defaultMessage: "Your project's member(s) has been updated.",
	},
})

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const permissions = computed(() => currentMember.value?.permissions ?? null)
const canManageInvites = computed(() =>
	hasPermission(permissions.value, projectPermissions.manageInvites),
)
const inviteDialog = useTemplateRef('inviteDialog')
const teamMembers = ref<MemberDraft[]>([])
const orgMembers = ref<MemberDraft[]>([])
const openMembers = ref<string[]>([])

watch(
	[allMembers, organization, project, currentMember],
	() => {
		const teamByUserId = new Map(allMembers.value.map((member) => [member.user.id, member]))
		orgMembers.value = (organization.value?.members ?? []).map((orgMember) => {
			const member = teamByUserId.get(orgMember.user.id)
			return {
				...(member ?? orgMember),
				is_owner: orgMember.is_owner,
				override: !!member,
				oldOverride: !!member,
			}
		})
		const orgUserIds = new Set(orgMembers.value.map((member) => member.user.id))
		teamMembers.value = allMembers.value
			.filter((member) => !orgUserIds.has(member.user.id))
			.map((member) => ({ ...member }))
	},
	{ immediate: true },
)

function toggleMember(id: string) {
	openMembers.value = openMembers.value.includes(id)
		? openMembers.value.filter((memberId) => memberId !== id)
		: [...openMembers.value, id]
}

function updateDraft(member: MemberDraft, changes: MemberChanges) {
	Object.assign(member, changes)
}

const revenueMembers = computed(() =>
	[...teamMembers.value, ...orgMembers.value].filter((member) => member.accepted),
)
const showRevenue = computed(
	() =>
		!isServerProject.value &&
		(!project.value.organization || !!organization.value) &&
		revenueMembers.value.every((member) => member.payouts_split != null),
)
function revenueWeight(member: MemberDraft) {
	return Math.max(0, Number(member.payouts_split) || 0)
}
const totalWeight = computed(() =>
	showRevenue.value
		? revenueMembers.value.reduce((total, member) => total + revenueWeight(member), 0)
		: 0,
)
function revenuePercentage(member: MemberDraft) {
	if (!member.accepted || !totalWeight.value) return 0
	return Math.round((revenueWeight(member) / totalWeight.value) * 1000) / 10
}

const { mutate: saveMember, isPending: saving } = useMutation({
	mutationFn: async (member: MemberDraft) => {
		const { team } = project.value
		const userId = member.user.id
		const data = { role: member.role, payouts_split: member.payouts_split ?? undefined }

		if (member.oldOverride !== undefined) {
			const override = { ...data, permissions: member.permissions ?? undefined }
			if (member.override && !member.oldOverride) {
				await labrinth.teams_v2.addMember(team, { ...override, user_id: userId })
			} else if (!member.override && member.oldOverride) {
				await labrinth.teams_v2.removeMember(team, userId)
			} else {
				await labrinth.teams_v2.editMember(team, userId, override)
			}
			return
		}

		await labrinth.teams_v2.editMember(
			team,
			userId,
			member.is_owner ? data : { ...data, permissions: member.permissions ?? undefined },
		)
	},
	onSuccess: async () => {
		await invalidate()
		addNotification({
			title: formatMessage(messages.updated),
			text: formatMessage(messages.updatedDescription),
			type: 'success',
		})
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

const { mutate: removeMember, isPending: removing } = useMutation({
	mutationFn: (userId: string) => labrinth.teams_v2.removeMember(project.value.team, userId),
	onSuccess: async () => {
		await invalidate()
		addNotification({
			title: formatMessage(messages.removed),
			text: formatMessage(messages.removedDescription),
			type: 'success',
		})
	},
	onError: notifyError,
	onSettled: () => {
		loading.value = false
	},
})

const busy = computed(() => saving.value || removing.value)

function save(member: MemberDraft) {
	if (busy.value || !hasPermission(permissions.value, projectPermissions.editMember)) return
	if (member.oldOverride === false && !member.override) return
	loading.value = true
	saveMember({ ...member })
}

function remove(member: MemberDraft) {
	if (
		busy.value ||
		!hasPermission(permissions.value, projectPermissions.removeMember) ||
		member.is_owner
	)
		return
	loading.value = true
	removeMember(member.user.id)
}

async function leaveProject() {
	await removeSelfFromTeam(project.value.team)
	navigateTo('/dashboard/projects')
}

const auth = await useAuth()
const canLeaveProject = computed(() =>
	allMembers.value.some(
		(member) => member.user.id === auth.value.user?.id && member.accepted && !member.is_owner,
	),
)
</script>

<template>
	<div data-project-field="members">
		<ProjectIssueCard location="members" class="mb-4" />
		<InviteDialog
			ref="inviteDialog"
			:can-invite="canManageInvites"
			:title="formatMessage(messages.addMember)"
		/>
		<div class="flex flex-col gap-8">
			<SettingsSection :title="formatMessage(commonProjectSettingsMessages.members)">
				<template #actions>
					<Button
						type="colored"
						color="brand"
						:disabled="!canManageInvites"
						@click="inviteDialog?.show($event)"
					>
						<UserPlusIcon /> {{ formatMessage(messages.addMember) }}
					</Button>
				</template>
				<MemberTable
					:members="teamMembers"
					:open-members="openMembers"
					:user-id="auth.user?.id"
					:permissions="permissions"
					:is-server-project="isServerProject"
					:show-revenue="showRevenue"
					:revenue-percentage="revenuePercentage"
					:busy="busy"
					@toggle="toggleMember"
					@update="updateDraft"
					@save="save"
					@remove="remove"
				/>
			</SettingsSection>
			<SettingsSection v-if="organization" :title="formatMessage(messages.organizationMembers)">
				<p>
					<IntlFormatted :message-id="messages.managedBy" :values="{ name: organization.name }">
						<template #organization="{ children }">
							<NuxtLink :to="`/organization/${organization.slug}`">
								<component :is="() => children" />
							</NuxtLink>
						</template>
					</IntlFormatted>
				</p>
				<MemberTable
					v-if="orgMembers.length"
					:members="orgMembers"
					:open-members="openMembers"
					:permissions="permissions"
					:is-server-project="isServerProject"
					:show-revenue="showRevenue"
					:revenue-percentage="revenuePercentage"
					:busy="busy"
					organization
					@toggle="toggleMember"
					@update="updateDraft"
					@save="save"
				/>
			</SettingsSection>
			<RevenueSummary
				v-if="showRevenue"
				:members="revenueMembers"
				:total-weight="totalWeight"
				:revenue-percentage="revenuePercentage"
			/>
			<SettingsOptionCard v-if="canLeaveProject" :title="formatMessage(messages.leave)">
				<p>{{ formatMessage(messages.leaveDescription) }}</p>
				<template #actions>
					<Button type="colored" color="red" @click="leaveProject">
						<UserXIcon /> {{ formatMessage(messages.leave) }}
					</Button>
				</template>
			</SettingsOptionCard>
		</div>
	</div>
</template>
