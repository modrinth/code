<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="danger"
		max-width="560px"
		:on-hide="() => backupCreator?.cancelBackup()"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{
					visibleItems.length === 1
						? formatMessage(messages.singleAdmonitionBody, {
								project:
									visibleItems[0]?.project.title ?? formatMessage(commonMessages.unknownLabel),
								context: contextLabel,
							})
						: formatMessage(messages.bulkAdmonitionBody, { context: contextLabel })
				}}
			</Admonition>

			<div v-if="visibleItems.length > 0" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.deletingLabel) }}</span>
				<div class="relative">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showDeletingTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="deletingListRef"
						class="flex flex-col gap-2 overflow-y-auto max-h-[212px]"
						@scroll="checkDeletingScrollState"
					>
						<div v-for="item in visibleItems" :key="item.id" :class="modalContentCardClasses">
							<ContentCardItem
								:project="item.project"
								:project-link="item.projectLink"
								:version="item.version"
								:version-link="item.versionLink"
								:owner="item.owner"
								hide-actions
								inline
							/>
						</div>
					</div>
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showDeletingBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</div>

			<div v-if="visibleDependents.length > 0" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.affectedDependentsLabel, { count: visibleDependents.length })
				}}</span>
				<div class="relative">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showDependentTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="dependentListRef"
						class="flex max-h-[212px] flex-col gap-2 overflow-y-auto"
						@scroll="checkDependentScrollState"
					>
						<div
							v-for="dependent in visibleDependents"
							:key="dependent.item.id"
							:class="modalContentCardClasses"
						>
							<ContentCardItem
								:project="dependent.item.project"
								:project-link="dependent.item.projectLink"
								:version="dependent.item.version"
								:version-link="dependent.item.versionLink"
								:owner="dependent.item.owner"
								hide-actions
								inline
							>
								<template #title-badges>
									<span class="flex min-w-0 flex-wrap items-center gap-1">
										<span
											v-for="dependency in dependent.dependencies"
											:key="dependency.id"
											:title="dependency.project.title"
										>
											<span class="truncate text-xs text-secondary mr-0.5"
												>({{ dependency.project.title }})</span
											>
										</span>
									</span>
								</template>
							</ContentCardItem>
						</div>
					</div>
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showDependentBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</div>

			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						formatMessage(messages.whatHappensLabel)
					}}</span>
					<ul class="m-0 list-disc pl-6 text-primary">
						<li class="leading-6 marker:text-secondary">
							{{ formatMessage(messages.effectDependentContent) }}
						</li>
						<li class="leading-6 marker:text-secondary">
							{{ formatMessage(messages.effectInstance, { context: contextLabel }) }}
						</li>
					</ul>
					<Checkbox
						v-model="disableDependentsAfterDeleting"
						:label="formatMessage(messages.disableDependentsLabel)"
						label-class="font-medium text-primary"
						class="mt-1"
					/>
				</div>
				<InlineBackupCreator
					ref="backupCreator"
					:backup-name="backupName"
					hide-shift-click-hint
					@update:buttons-disabled="buttonsDisabled = $event"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-5" @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="buttonsDisabled || props.actionDisabled"
						@click="confirm"
					>
						<TrashIcon aria-hidden="true" />
						{{ deleteButtonLabel }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useScrollIndicator } from '#ui/composables/scroll-indicator'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

import type { ContentCardTableItem } from '../../types'
import ContentCardItem from '../ContentCardItem.vue'
import InlineBackupCreator from './InlineBackupCreator.vue'

export interface ContentDependencyWarningDependent {
	item: ContentCardTableItem
	dependencies: ContentCardTableItem[]
}

const props = withDefaults(
	defineProps<{
		items?: ContentCardTableItem[]
		dependents?: ContentDependencyWarningDependent[]
		itemType: string
		variant?: 'instance' | 'server'
		backupTip?: string
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		items: () => [],
		dependents: () => [],
		variant: 'instance',
		backupTip: undefined,
		actionDisabled: false,
		actionDisabledTooltip: undefined,
	},
)

const emit = defineEmits<{
	(e: 'delete', disableDependentsAfterDeleting: boolean): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'content.dependency-warning.header',
		defaultMessage: 'Dependency warning',
	},
	admonitionHeader: {
		id: 'content.dependency-warning.admonition-header',
		defaultMessage: 'This content is required by other content',
	},
	singleAdmonitionBody: {
		id: 'content.dependency-warning.single-admonition-body',
		defaultMessage:
			'{project} is installed as a dependency. Deleting it may break your {context} or stop dependent content from loading correctly.',
	},
	bulkAdmonitionBody: {
		id: 'content.dependency-warning.bulk-admonition-body',
		defaultMessage:
			'Some selected projects are installed as dependencies. Deleting them may break your {context} or stop dependent content from loading correctly.',
	},
	deletingLabel: {
		id: 'content.dependency-warning.deleting-label',
		defaultMessage: 'Deleting',
	},
	affectedDependentsLabel: {
		id: 'content.dependency-warning.affected-dependents-label',
		defaultMessage: 'Affected {count, plural, one {project} other {projects}}',
	},
	whatHappensLabel: {
		id: 'content.dependency-warning.what-happens-label',
		defaultMessage: 'What happens?',
	},
	effectDependentContent: {
		id: 'content.dependency-warning.effect-dependent-content',
		defaultMessage: 'Dependent content may fail to load or may disable itself',
	},
	effectInstance: {
		id: 'content.dependency-warning.effect-instance',
		defaultMessage: 'Your {context} may crash, refuse to start, or behave unexpectedly',
	},
	deleteAnywayButton: {
		id: 'content.dependency-warning.delete-anyway-button',
		defaultMessage: 'Delete anyway',
	},
	deleteManyAnywayButton: {
		id: 'content.dependency-warning.delete-many-anyway-button',
		defaultMessage: 'Delete {count, number} {itemType} anyway',
	},
	disableDependentsLabel: {
		id: 'content.dependency-warning.disable-dependents-label',
		defaultMessage: 'Disable dependents after deleting',
	},
	instanceContext: {
		id: 'content.dependency-warning.context.instance',
		defaultMessage: 'instance',
	},
	serverContext: {
		id: 'content.dependency-warning.context.server',
		defaultMessage: 'server',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const backupCreator = ref<InstanceType<typeof InlineBackupCreator>>()
const deletingListRef = ref<HTMLElement | null>(null)
const dependentListRef = ref<HTMLElement | null>(null)
const visibleItems = ref<ContentCardTableItem[]>(props.items)
const visibleDependents = ref<ContentDependencyWarningDependent[]>(props.dependents)
const visibleItemType = ref(props.itemType)
const disableDependentsAfterDeleting = ref(false)
const buttonsDisabled = ref(false)
const modalContentCardClasses = 'rounded-xl border border-solid border-surface-5 p-4 !bg-surface-2'
const {
	showTopFade: showDeletingTopFade,
	showBottomFade: showDeletingBottomFade,
	checkScrollState: checkDeletingScrollState,
	forceCheck: forceCheckDeletingScroll,
} = useScrollIndicator(deletingListRef)
const {
	showTopFade: showDependentTopFade,
	showBottomFade: showDependentBottomFade,
	checkScrollState: checkDependentScrollState,
	forceCheck: forceCheckDependentScroll,
} = useScrollIndicator(dependentListRef)

const contextLabel = computed(() =>
	formatMessage(props.variant === 'server' ? messages.serverContext : messages.instanceContext),
)

const backupName = computed(() =>
	props.backupTip ? `Before deletion (${props.backupTip})` : 'Before deletion',
)

const deleteButtonLabel = computed(() => {
	if (visibleItems.value.length <= 1) return formatMessage(messages.deleteAnywayButton)

	return formatMessage(messages.deleteManyAnywayButton, {
		count: visibleItems.value.length,
		itemType: formatContentTypeSentence(
			formatMessage,
			visibleItemType.value,
			visibleItems.value.length,
		),
	})
})

async function show() {
	await nextTick()
	visibleItems.value = props.items
	visibleDependents.value = props.dependents
	visibleItemType.value = props.itemType
	disableDependentsAfterDeleting.value = false
	buttonsDisabled.value = false
	modal.value?.show()
	await nextTick()
	forceCheckDeletingScroll()
	forceCheckDependentScroll()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	if (props.actionDisabled || buttonsDisabled.value) return
	modal.value?.hide()
	emit('delete', disableDependentsAfterDeleting.value)
}

defineExpose({
	show,
	hide,
})
</script>
