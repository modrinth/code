<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	ChevronDownIcon,
	EditIcon,
	FileIcon,
	PlusIcon,
	ReportIcon,
	ScaleIcon,
	SpinnerIcon,
	VersionIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import { renderString } from '@modrinth/utils'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef, watch } from 'vue'

import { ButtonStyled, Collapsible, OverflowMenu } from '#ui/components'
import type { OverflowMenuOption } from '#ui/components/base'
import { commonMessages } from '#ui/utils'

import { defineMessage, defineMessages, useVIntl } from '../../composables/i18n'
import {
	injectAttributionModeration,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
} from '../../providers'
import type { QuickReply } from '../../providers/attribution-moderation'
import StyledInput from '../base/StyledInput.vue'
import AddFilesToAttributionGroupModal from './AddFilesToAttributionGroupModal.vue'
import AddToExistingExternalProjectModal from './AddToExistingExternalProjectModal.vue'
import AddToGlobalPermissionsDatabaseModal from './AddToGlobalPermissionsDatabaseModal.vue'
import AttributionDisplay from './AttributionDisplay.vue'
import AttributionEditor from './AttributionEditor.vue'
import AttributionModerationDbBadge from './AttributionModerationDbBadge.vue'
import AttributionStatusTag from './AttributionStatusTag.vue'
import {
	attributionLinkToWork,
	createAttributionGroupTitle,
	MODERATION_DB_BADGE,
	parseInitialAttribution,
} from './external-project-utils'
import OriginalPageLink from './OriginalPageLink.vue'

const props = withDefaults(
	defineProps<{
		projectId: string
		group: Labrinth.Attribution.Internal.AttributionGroup
		isModerator?: boolean
	}>(),
	{
		isModerator: false,
	},
)

const collapsedModel = defineModel<boolean>('collapsed')

const collapsed = computed({
	get: () =>
		collapsedModel.value ??
		(!props.isModerator &&
			!!props.group.attribution &&
			props.group.attribution?.moderation_status?.kind !== 'bad_proof'),
	set: (value) => {
		collapsedModel.value = value
	},
})

const emit = defineEmits<{
	(e: 'updated'): void
}>()

const addFilesModalRef = useTemplateRef<typeof AddFilesToAttributionGroupModal>('addFilesModalRef')
const addToGlobalModalRef =
	useTemplateRef<typeof AddToGlobalPermissionsDatabaseModal>('addToGlobalModalRef')
const addToExistingModalRef =
	useTemplateRef<typeof AddToExistingExternalProjectModal>('addToExistingModalRef')

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { allMembers } = injectProjectPageContext()
const attributionModeration = injectAttributionModeration(null)

const attributorMember = computed(() => {
	const userId = props.group.attributed_by
	if (!userId || !allMembers.value) {
		return null
	}
	return allMembers.value.find((member) => member.user.id === userId) ?? null
})

const attributorLink = computed(() => {
	const id = props.group.attributed_by
	if (!id) {
		return null
	}
	const slug = attributorMember.value?.user?.username ?? id
	return `/user/${slug}`
})

const attributorLabel = computed(() => {
	if (attributorMember.value) {
		return attributorMember.value.user.username
	}
	return props.group.attributed_by ?? 'unknown'
})

const messages = defineMessages({
	fileCount: {
		id: 'external-files.permissions-card.file-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	includedInVersions: {
		id: 'external-files.permissions-card.included-in-versions',
		defaultMessage: 'Included in {count, plural, one {# version} other {# versions}}:',
	},
	includedFiles: {
		id: 'external-files.permissions-card.included-files',
		defaultMessage: 'Included files:',
	},
	notUsedInVersions: {
		id: 'external-files.permissions-card.not-used-in-versions',
		defaultMessage: 'These files are not currently used by any version.',
	},
	splitFile: {
		id: 'external-files.permissions-card.split-file',
		defaultMessage: 'Remove from group',
	},
	addFilesToGroup: {
		id: 'external-files.permissions-card.add-files-to-group',
		defaultMessage: 'Add files...',
	},
	moderationReasonLabel: {
		id: 'external-files.permissions-card.moderation-reason',
		defaultMessage: 'Reason',
	},
})

type EditingMode = 'attribution' | 'moderation_review'

const editingMode = ref<EditingMode | null>(null)
const editorResumeKey = ref(0)

const isEditingAttribution = computed(() => editingMode.value === 'attribution')
const isEditingModerationReview = computed(() => editingMode.value === 'moderation_review')

const initialAttribution = computed<Labrinth.Attribution.Internal.AttributionResolution | null>(
	() => parseInitialAttribution(props.group.attribution),
)

const isAttributed = computed(() => initialAttribution.value !== null)
const attributionStatusVariant = computed<
	'pending' | 'attributed' | 'no_permission' | 'proof_rejected' | 'not_allowed'
>(() => {
	if (isAttributed.value && props.group.attribution?.kind === 'no_permission') {
		return 'no_permission'
	} else if (
		isAttributed.value &&
		props.group.attribution?.moderation_status?.kind === 'not_allowed'
	) {
		return 'not_allowed'
	} else if (
		isAttributed.value &&
		props.group.attribution?.moderation_status?.kind === 'bad_proof'
	) {
		return 'proof_rejected'
	} else if (isAttributed.value) {
		return 'attributed'
	}
	return 'pending'
})

const title = computed(() => createAttributionGroupTitle(props.group, formatMessage))
const fileCount = computed(() => props.group.files?.length ?? 0)

const containingVersions = computed(() => {
	const versionIds = new Set<string>()
	for (const file of props.group.files ?? []) {
		for (const versionId of file.versions ?? []) {
			versionIds.add(versionId)
		}
	}
	return props.group.versions?.filter((v) => versionIds.has(v.id))
})

const pendingSplitSha1 = ref<string | null>(null)

const assignFilesMutation = useMutation({
	mutationFn: async (sha1s: string[]) => {
		for (const sha1 of sha1s) {
			await client.labrinth.attribution_internal.assignFileToGroup({
				sha1,
				target_group_id: props.group.id,
				project_id: props.projectId,
			})
		}
	},
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		emit('updated')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.assign-files-error.title',
					defaultMessage: 'Could not add files',
				}),
			),
			text: error.message,
		})
	},
})

const splitFileMutation = useMutation({
	mutationFn: (sha1: string) =>
		client.labrinth.attribution_internal.splitFile({
			sha1,
			project_id: props.projectId,
		}),
	onMutate(sha1) {
		pendingSplitSha1.value = sha1
	},
	onSettled() {
		pendingSplitSha1.value = null
	},
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		emit('updated')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.split-file-error.title',
					defaultMessage: 'Could not split file',
				}),
			),
			text: error.message,
		})
	},
})

function startEditingAttribution() {
	editingMode.value = 'attribution'
	collapsed.value = false
	editorResumeKey.value += 1
}

function startEditingModerationReview() {
	syncReviewReasonInput()
	editingMode.value = 'moderation_review'
}

function stopEditing() {
	editingMode.value = null
}

function cancelModerationReviewEditing() {
	syncReviewReasonInput()
	stopEditing()
}

function handleEditorUpdated() {
	emit('updated')
}

function handleSplitFile(sha1: string) {
	splitFileMutation.mutate(sha1)
}

function handleConfirmAddFiles(sha1s: string[]) {
	assignFilesMutation.mutate(sha1s)
}

async function handleAddFilesToGroup(event: MouseEvent) {
	try {
		const groups = await queryClient.ensureQueryData({
			queryKey: ['project-attribution', props.projectId],
			queryFn: () => client.labrinth.attribution_internal.listProjectAttribution(props.projectId),
		})
		addFilesModalRef.value?.show(event, groups)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.add-files-modal.load-error.title',
					defaultMessage: 'Could not load files',
				}),
			),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}

async function handleModerationDbUpdated() {
	await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
	emit('updated')
}

function handleAddToGlobalDatabase(event: MouseEvent) {
	addToGlobalModalRef.value?.show(event)
}

function handleAddToExistingEntry(event: MouseEvent) {
	addToExistingModalRef.value?.show(event)
}

const originalProjectUrl = computed(
	() => attributionLinkToWork(initialAttribution.value) ?? props.group.flame_project?.url,
)

const moderationStatusKind = computed(
	() => props.group.attribution?.moderation_status?.kind ?? null,
)

const moderationStatusIndicator = computed(() => {
	if (!moderationStatusKind.value) {
		return null
	}
	switch (moderationStatusKind.value) {
		case 'approved':
			return {
				icon: CheckCircleIcon,
				class: 'text-green',
				name: defineMessage({
					id: 'external-files.permissions-card.attribution.moderation-status.passed',
					defaultMessage: 'Passed',
				}),
			}
		case 'bad_proof':
			return {
				icon: XCircleIcon,
				class: 'text-red',
				name: defineMessage({
					id: 'external-files.permissions-card.attribution.moderation-status.rejected-proof',
					defaultMessage: 'Proof rejected',
				}),
			}
		case 'not_allowed':
			return {
				icon: ReportIcon,
				class: 'text-red',
				name: defineMessage({
					id: 'external-files.permissions-card.attribution.moderation-status.content-not-allowed',
					defaultMessage: 'Content not allowed',
				}),
			}
	}
	return null
})

const reviewReasonInput = ref('')

function syncReviewReasonInput() {
	reviewReasonInput.value = props.group.attribution?.moderation_status?.reason ?? ''
}

syncReviewReasonInput()

watch(
	() => props.group.attribution?.moderation_status,
	() => {
		if (!isEditingModerationReview.value) {
			syncReviewReasonInput()
		}
	},
	{ deep: true },
)

const pendingModerationStatusKind =
	ref<Labrinth.Attribution.Internal.AttributionModerationStatusKind | null>(null)

const setModerationStatusMutation = useMutation({
	mutationFn: (kind: Labrinth.Attribution.Internal.AttributionModerationStatusKind) => {
		if (!initialAttribution.value) {
			throw new Error('Attribution is required')
		}
		return client.labrinth.attribution_internal.updateGroup(props.group.id, {
			attribution: {
				...initialAttribution.value,
				moderation_status: {
					kind,
					reason: reviewReasonInput.value.trim(),
				},
			},
		})
	},
	onMutate(kind) {
		pendingModerationStatusKind.value = kind
	},
	onSettled() {
		pendingModerationStatusKind.value = null
	},
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		stopEditing()
		collapsed.value = true
		emit('updated')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.moderation.error.title',
					defaultMessage: 'Could not save moderation review',
				}),
			),
			text: error.message,
		})
	},
})

function handleSetModerationStatus(
	kind: Labrinth.Attribution.Internal.AttributionModerationStatusKind,
) {
	setModerationStatusMutation.mutate(kind)
}

async function handleQuickReply(reply: QuickReply) {
	const message =
		typeof reply.message === 'function' ? await reply.message(undefined) : reply.message
	reviewReasonInput.value = message
}

const visibleQuickReplies = computed<OverflowMenuOption[]>(() => {
	const replies = attributionModeration?.attributionQuickReplies

	if (!replies) return []

	return replies
		.filter((reply) => {
			if (reply.shouldShow === undefined) return true
			return reply.shouldShow(undefined)
		})
		.map(
			(reply) =>
				({
					id: reply.label,
					action: () => handleQuickReply(reply),
				}) as OverflowMenuOption,
		)
})
</script>

<template>
	<div
		class="bg-surface-2 p-0 rounded-2xl flex flex-col border-[1px] border-solid border-surface-5 overflow-hidden"
	>
		<div class="flex items-center bg-surface-3 gap-3">
			<button
				class="flex grow items-center m-0 appearance-none p-4 bg-transparent group transition-all gap-3 text-left min-w-0 outline-offset-[-3px] rounded-2xl"
				:class="{
					'rounded-b-none': !collapsed,
					'rounded-r-none': group.flame_project?.url || isModerator,
				}"
				@click="collapsed = !collapsed"
			>
				<ChevronDownIcon
					class="size-6 text-primary transition-transform duration-300 shrink-0 mb-auto"
					:class="{ 'rotate-180': !collapsed }"
				/>
				<span class="flex flex-col items-start min-w-0 group-active:scale-[0.98]">
					<span class="flex items-center gap-2 min-w-0 flex-wrap">
						<span class="text-contrast truncate font-semibold">{{ title }}</span>
						<component
							:is="moderationStatusIndicator.icon"
							v-if="moderationStatusIndicator"
							v-tooltip="formatMessage(moderationStatusIndicator.name)"
							:class="moderationStatusIndicator.class"
							class="size-5 shrink-0"
							aria-hidden="true"
						/>
						<AttributionStatusTag :variant="attributionStatusVariant" />
						<OriginalPageLink v-if="originalProjectUrl && isModerator" :href="originalProjectUrl" />
					</span>
					<span v-if="fileCount > 1" class="text-secondary text-sm font-normal">
						{{ formatMessage(messages.fileCount, { count: fileCount }) }}
					</span>
				</span>
			</button>
			<div class="mr-4 flex items-center gap-2">
				<AttributionModerationDbBadge v-if="isModerator" :files="group.files" />
				<OriginalPageLink v-else-if="originalProjectUrl" :href="originalProjectUrl" />
			</div>
		</div>

		<Collapsible
			:collapsed="collapsed"
			class="border-0 border-solid border-t border-surface-5 rounded-b-2xl"
		>
			<div class="flex flex-col gap-3 p-4">
				<span class="text-contrast font-semibold">
					{{ formatMessage(messages.includedFiles) }}
				</span>
				<div class="grid grid-cols-2 gap-2">
					<span
						v-for="file in group.files"
						:key="file.sha1"
						class="pl-3 rounded-xl grid grid-cols-[auto_1fr_auto] gap-2 items-start border-[1px] border-solid border-surface-5 bg-surface-2"
						:style="{
							'border-color':
								isModerator && file.moderation_external_license
									? MODERATION_DB_BADGE[file.moderation_external_license?.status]?.color
									: undefined,
							color:
								isModerator && file.moderation_external_license
									? MODERATION_DB_BADGE[file.moderation_external_license?.status]?.color
									: undefined,
						}"
					>
						<FileIcon class="size-4 shrink-0 mt-2.5" />
						<div class="max-w-[22rem] min-w-0 flex flex-col gap-1 py-2">
							<span class="truncate">
								{{ file.name.split('/').pop() }}
							</span>
						</div>
						<div class="flex items-center gap-1 my-auto">
							<ButtonStyled v-if="group.files.length > 1" circular size="small">
								<button
									v-tooltip="formatMessage(messages.splitFile)"
									class="m-1"
									:disabled="splitFileMutation.isPending.value"
									@click="handleSplitFile(file.sha1)"
								>
									<SpinnerIcon
										v-if="splitFileMutation.isPending.value && pendingSplitSha1 === file.sha1"
										class="size-4 shrink-0 animate-spin"
									/>
									<XIcon v-else class="size-4 shrink-0" />
								</button>
							</ButtonStyled>
						</div>
					</span>
					<div>
						<ButtonStyled>
							<button @click="handleAddFilesToGroup($event)">
								<PlusIcon class="size-4 shrink-0" /> {{ formatMessage(messages.addFilesToGroup) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
				<template v-if="(containingVersions?.length ?? 0) > 0">
					<span class="text-contrast font-semibold">
						{{
							formatMessage(messages.includedInVersions, { count: containingVersions?.length ?? 0 })
						}}
					</span>
					<div class="flex flex-wrap gap-2">
						<nuxt-link
							v-for="version in containingVersions"
							:key="version.id"
							:to="`/project/${projectId}/version/${version.id}`"
							target="_blank"
							class="px-3 py-2 rounded-xl flex items-center gap-2 border-[1px] border-solid border-surface-5 bg-surface-3 hover:bg-surface-4"
						>
							<VersionIcon class="size-4 shrink-0" />
							<span class="max-w-[22rem] truncate">
								{{ version.version_number }}
							</span>
						</nuxt-link>
					</div>
				</template>
				<template v-else>
					<span class="text-secondary text-sm">
						{{ formatMessage(messages.notUsedInVersions) }}
					</span>
				</template>
				<AttributionDisplay
					v-if="!isEditingAttribution && initialAttribution"
					:attribution="initialAttribution"
					:attributed-at="group.attributed_at"
					:attributed-by="group.attributed_by"
					:attributor-href="attributorLink"
					:attributor-label="attributorLabel"
					:attributor-avatar-url="attributorMember?.user.avatar_url"
					:moderator="isModerator"
				>
					<template
						v-if="
							group.attribution?.kind !== 'globally_allowed' &&
							(group.attribution?.moderation_status?.kind !== 'not_allowed' || isModerator)
						"
						#actions
					>
						<ButtonStyled>
							<button @click="startEditingAttribution">
								<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
							</button>
						</ButtonStyled>
					</template>
					<template
						v-if="
							(isModerator || group.attribution?.moderation_status) &&
							group.attribution?.kind !== 'globally_allowed'
						"
						#footer
					>
						<div class="flex gap-4 flex-wrap">
							<div>
								<p
									class="font-semibold m-0 flex items-center gap-2"
									:class="isModerator ? 'text-orange' : moderationStatusIndicator?.class"
								>
									<template v-if="isModerator">
										<ScaleIcon class="size-5 shrink-0" />
										Review attribution
									</template>
									<template v-else-if="moderationStatusIndicator">
										<component
											:is="moderationStatusIndicator.icon"
											class="size-5 shrink-0"
											aria-hidden="true"
										/>
										{{ formatMessage(moderationStatusIndicator.name) }}
									</template>
								</p>
								<template v-if="isModerator">
									<template
										v-if="group.attribution?.moderation_status && !isEditingModerationReview"
									>
										<div class="grid grid-cols-[auto_1fr] gap-y-3 gap-x-4 mt-3">
											<div>Status:</div>
											<div
												class="flex items-center gap-1"
												:class="moderationStatusIndicator ? moderationStatusIndicator.class : ''"
											>
												<template v-if="moderationStatusIndicator">
													<component
														:is="moderationStatusIndicator.icon"
														class="size-4 shrink-0"
														aria-hidden="true"
													/>
													{{ formatMessage(moderationStatusIndicator.name) }}
												</template>
											</div>
											<div class="leading-[1.5]">Reason:</div>
											<div>
												<div
													class="markdown-body"
													v-html="
														renderString(group.attribution?.moderation_status?.reason || 'N/A')
													"
												/>
											</div>
										</div>
									</template>
									<template v-else>
										<StyledInput
											v-model="reviewReasonInput"
											multiline
											placeholder="Explanation of review (optional)"
											class="mt-3"
										/>
										<div class="flex items-center gap-2 flex-wrap mt-3">
											<ButtonStyled v-if="visibleQuickReplies.length > 0">
												<OverflowMenu :options="visibleQuickReplies">
													Reply presets
													<ChevronDownIcon />
												</OverflowMenu>
											</ButtonStyled>
											<ButtonStyled color="green" color-fill="text">
												<button
													:disabled="setModerationStatusMutation.isPending.value"
													@click="handleSetModerationStatus('approved')"
												>
													<SpinnerIcon
														v-if="
															setModerationStatusMutation.isPending.value &&
															pendingModerationStatusKind === 'approved'
														"
														class="size-4 shrink-0 animate-spin"
													/>
													<CheckCircleIcon v-else />
													Approve
												</button>
											</ButtonStyled>
											<ButtonStyled color="red" color-fill="text">
												<button
													:disabled="setModerationStatusMutation.isPending.value"
													@click="handleSetModerationStatus('bad_proof')"
												>
													<SpinnerIcon
														v-if="
															setModerationStatusMutation.isPending.value &&
															pendingModerationStatusKind === 'bad_proof'
														"
														class="size-4 shrink-0 animate-spin"
													/>
													<XCircleIcon v-else />
													Reject: Insufficient proof
												</button>
											</ButtonStyled>
											<ButtonStyled color="red" color-fill="text">
												<button
													:disabled="setModerationStatusMutation.isPending.value"
													@click="handleSetModerationStatus('not_allowed')"
												>
													<SpinnerIcon
														v-if="
															setModerationStatusMutation.isPending.value &&
															pendingModerationStatusKind === 'not_allowed'
														"
														class="size-4 shrink-0 animate-spin"
													/>
													<ReportIcon v-else />
													Reject: Not allowed
												</button>
											</ButtonStyled>
											<ButtonStyled v-if="isEditingModerationReview" type="outlined">
												<button
													:disabled="setModerationStatusMutation.isPending.value"
													@click="cancelModerationReviewEditing"
												>
													<XIcon />
													{{ formatMessage(commonMessages.cancelButton) }}
												</button>
											</ButtonStyled>
										</div>
										<div class="flex items-center gap-2 flex-wrap mt-3">
											<ButtonStyled>
												<button @click="handleAddToGlobalDatabase">
													<ScaleIcon /> Add files to database...
												</button>
											</ButtonStyled>
											<ButtonStyled>
												<button @click="handleAddToExistingEntry">
													<ScaleIcon /> Add to existing entry...
												</button>
											</ButtonStyled>
										</div>
									</template>
								</template>
								<div
									v-else-if="group.attribution?.moderation_status?.reason"
									class="flex flex-col gap-2 mt-3"
								>
									<div
										class="markdown-body"
										v-html="renderString(group.attribution?.moderation_status?.reason)"
									/>
								</div>
							</div>
							<div
								v-if="
									isModerator && !isEditingModerationReview && group.attribution?.moderation_status
								"
								class="ml-auto"
							>
								<ButtonStyled color="orange">
									<button @click="startEditingModerationReview">
										<ScaleIcon />
										{{ formatMessage(commonMessages.editButton) }}
									</button>
								</ButtonStyled>
							</div>
						</div>
					</template>
				</AttributionDisplay>

				<div
					v-else
					class="rounded-2xl p-4 mt-2 border-[1px] border-solid border-surface-5 flex flex-col gap-3"
				>
					<AttributionEditor
						:project-id="projectId"
						:group-id="group.id"
						:attribution="group.attribution"
						:flame-project-url="group.flame_project?.url"
						:resume-key="editorResumeKey"
						@updated="handleEditorUpdated"
						@saved="stopEditing"
						@cancel="stopEditing"
					/>
				</div>
			</div>
		</Collapsible>
		<AddFilesToAttributionGroupModal
			ref="addFilesModalRef"
			:group-id="group.id"
			:pending="assignFilesMutation.isPending.value"
			@confirm="handleConfirmAddFiles"
		/>
		<AddToGlobalPermissionsDatabaseModal
			ref="addToGlobalModalRef"
			:group="group"
			@success="handleModerationDbUpdated"
		/>
		<AddToExistingExternalProjectModal
			ref="addToExistingModalRef"
			:group="group"
			@success="handleModerationDbUpdated"
		/>
	</div>
</template>
