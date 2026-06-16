<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="560px">
		<div class="flex flex-col gap-4">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{
					formatMessage(messages.admonitionBody, {
						project: visibleItem?.project.title ?? formatMessage(commonMessages.unknownLabel),
						context: contextLabel,
					})
				}}
			</Admonition>

			<div v-if="visibleItem" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.deletingLabel) }}</span>
				<div :class="modalContentCardClasses">
					<ContentCardItem
						:project="visibleItem.project"
						:project-link="visibleItem.projectLink"
						:version="visibleItem.version"
						:version-link="visibleItem.versionLink"
						:owner="visibleItem.owner"
						hide-actions
						inline
					/>
				</div>
			</div>

			<div v-if="visibleDependents.length > 0" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.requiredByLabel, { count: visibleDependents.length })
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
							v-if="showTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="dependentListRef"
						class="flex max-h-[232px] flex-col gap-2 overflow-y-auto"
						@scroll="checkScrollState"
					>
						<div
							v-for="dependent in visibleDependents"
							:key="dependent.id"
							:class="modalContentCardClasses"
						>
							<ContentCardItem
								:project="dependent.project"
								:project-link="dependent.projectLink"
								:version="dependent.version"
								:version-link="dependent.versionLink"
								:owner="dependent.owner"
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
							v-if="showBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</div>

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
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-5" @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button
						v-tooltip="props.actionDisabled ? props.actionDisabledTooltip : undefined"
						:disabled="props.actionDisabled"
						@click="confirm"
					>
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.deleteAnywayButton) }}
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
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentCardTableItem } from '../../types'
import ContentCardItem from '../ContentCardItem.vue'

const props = withDefaults(
	defineProps<{
		item: ContentCardTableItem | null
		dependents?: ContentCardTableItem[]
		variant?: 'instance' | 'server'
		actionDisabled?: boolean
		actionDisabledTooltip?: string
	}>(),
	{
		dependents: () => [],
		variant: 'instance',
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
	admonitionBody: {
		id: 'content.dependency-warning.admonition-body',
		defaultMessage:
			'{project} is installed as a dependency. Deleting it may break your {context} or stop dependent content from loading correctly.',
	},
	deletingLabel: {
		id: 'content.dependency-warning.deleting-label',
		defaultMessage: 'Deleting',
	},
	requiredByLabel: {
		id: 'content.dependency-warning.required-by-label',
		defaultMessage: 'Required by {count, plural, one {# project} other {# projects}}',
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
const dependentListRef = ref<HTMLElement | null>(null)
const visibleItem = ref<ContentCardTableItem | null>(props.item)
const visibleDependents = ref<ContentCardTableItem[]>(props.dependents)
const disableDependentsAfterDeleting = ref(false)
const modalContentCardClasses = 'rounded-xl border border-solid border-surface-5 p-3 !bg-surface-2'
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(dependentListRef)

const contextLabel = computed(() =>
	formatMessage(props.variant === 'server' ? messages.serverContext : messages.instanceContext),
)

async function show() {
	await nextTick()
	visibleItem.value = props.item
	visibleDependents.value = props.dependents
	disableDependentsAfterDeleting.value = false
	modal.value?.show()
	await nextTick()
	forceCheck()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	if (props.actionDisabled) return
	modal.value?.hide()
	emit('delete', disableDependentsAfterDeleting.value)
}

defineExpose({
	show,
	hide,
})
</script>
