<script setup lang="ts">
import { ScaleIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectProjectPageContext,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

const { projectV3: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const { formatMessage } = useVIntl()
const flags = useFeatureFlags()
const messages = defineMessages({
	enable: {
		id: 'project.settings.monetization.enable',
		defaultMessage: 'Enable monetization',
	},
	disable: {
		id: 'project.settings.monetization.disable',
		defaultMessage: 'Disable monetization',
	},
	allow: {
		id: 'project.settings.monetization.allow',
		defaultMessage: 'Allow monetization',
	},
	forceDisable: {
		id: 'project.settings.monetization.force-disable',
		defaultMessage: 'Force disable',
	},
	confirmEnable: {
		id: 'project.settings.monetization.confirm-enable',
		defaultMessage: 'Enable monetization?',
	},
	confirmDisable: {
		id: 'project.settings.monetization.confirm-disable',
		defaultMessage: 'Disable monetization?',
	},
	confirmAllow: {
		id: 'project.settings.monetization.confirm-allow',
		defaultMessage: 'Allow monetization?',
	},
	confirmForceDisable: {
		id: 'project.settings.monetization.confirm-force-disable',
		defaultMessage: 'Force disable?',
	},
	noPermission: {
		id: 'project.settings.monetization.no-permission',
		defaultMessage: 'You do not have permission to change monetization.',
	},
	description: {
		id: 'project.settings.monetization.description',
		defaultMessage:
			'Projects on Modrinth are automatically enrolled in the <rewards-link>Rewards Program</rewards-link>. If you don’t want to (or can’t for legal reasons) earn revenue from this project, you can disable rewards here.',
	},
	disabledByModerator: {
		id: 'project.settings.monetization.disabled-by-moderator',
		defaultMessage:
			'Monetization has been disabled by a moderator. <support-link>Contact support</support-link> if you believe this is a mistake.',
	},
	forceDisableDescription: {
		id: 'project.settings.monetization.force-disable-description',
		defaultMessage:
			'<strong>{project}</strong> will stop earning revenue from the Rewards Program. Earnings already made are not affected.',
	},
	forceDisableRestriction: {
		id: 'project.settings.monetization.force-disable-restriction',
		defaultMessage:
			'Project members will not be able to re-enable monetization. Only a moderator can allow it again.',
	},
	allowDescription: {
		id: 'project.settings.monetization.allow-description',
		defaultMessage:
			'Allow <strong>{project}</strong> to earn revenue from the Rewards Program again.',
	},
	allowRestriction: {
		id: 'project.settings.monetization.allow-restriction',
		defaultMessage:
			'This removes the moderation restriction. Project members will be able to manage monetization.',
	},
	disableDescription: {
		id: 'project.settings.monetization.disable-description',
		defaultMessage:
			'<strong>{project}</strong> will stop earning revenue from the Rewards Program. Earnings you’ve already made are not affected.',
	},
	disableRestriction: {
		id: 'project.settings.monetization.disable-restriction',
		defaultMessage: 'You can turn monetization back on at any time.',
	},
	enableDescription: {
		id: 'project.settings.monetization.enable-description',
		defaultMessage:
			'<strong>{project}</strong> will start earning revenue through the Rewards Program.',
	},
})
const modal = useTemplateRef('modal')
const mode = ref<'member' | 'moderator'>('member')
const monetized = computed(() => project.value.monetization_status === 'monetized')
const forceDisabled = computed(() => project.value.monetization_status === 'force-demonetized')
const moderatorUi = computed(
	() => isStaff(currentMember.value?.user) && !flags.value.showModeratorProjectMemberUi,
)
const canEdit = computed(
	() =>
		isAdmin(currentMember.value?.user) ||
		Boolean((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)
const memberAction = computed(() =>
	formatMessage(monetized.value ? messages.disable : messages.enable),
)
const moderatorAction = computed(() =>
	formatMessage(forceDisabled.value ? messages.allow : messages.forceDisable),
)
const action = computed(() =>
	mode.value === 'moderator' ? moderatorAction.value : memberAction.value,
)
const confirmationTitle = computed(() =>
	formatMessage(
		mode.value === 'moderator'
			? forceDisabled.value
				? messages.confirmAllow
				: messages.confirmForceDisable
			: monetized.value
				? messages.confirmDisable
				: messages.confirmEnable,
	),
)
const canConfirm = computed(() =>
	mode.value === 'moderator' ? moderatorUi.value : canEdit.value && !forceDisabled.value,
)
const mutation = useMutation({
	mutationFn: async () => {
		if (!canConfirm.value) throw new Error(formatMessage(messages.noPermission))
		const monetization_status =
			mode.value === 'moderator'
				? forceDisabled.value
					? 'monetized'
					: 'force-demonetized'
				: monetized.value
					? 'demonetized'
					: 'monetized'
		await patchProjectV3({ monetization_status })
	},
	onSuccess: () => modal.value?.hide(),
})
function show(nextMode: 'member' | 'moderator', event: MouseEvent) {
	mode.value = nextMode
	mutation.reset()
	modal.value?.show(event)
}
</script>

<template>
	<div class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">{{ memberAction }}</h3>
			<p class="m-0 leading-normal text-secondary">
				<IntlFormatted :message-id="messages.description">
					<template #rewards-link="{ children }">
						<NuxtLink to="/legal/cmp-info" target="_blank" class="text-link">
							<component :is="() => children" />
						</NuxtLink>
					</template>
				</IntlFormatted>
			</p>
			<p v-if="forceDisabled" class="mb-0 mt-2 leading-tight text-secondary">
				<IntlFormatted :message-id="messages.disabledByModerator">
					<template #support-link="{ children }">
						<a
							href="https://support.modrinth.com"
							target="_blank"
							rel="noopener noreferrer"
							class="text-link"
						>
							<component :is="() => children" />
						</a>
					</template>
				</IntlFormatted>
			</p>
		</div>
		<div class="flex shrink-0 flex-wrap items-center gap-2">
			<Button
				v-if="moderatorUi"
				size="sm"
				type="outlined"
				color="orange"
				:disabled="mutation.isPending.value"
				@click="show('moderator', $event)"
				><ScaleIcon aria-hidden="true" />{{ moderatorAction }}</Button
			>
			<Button
				size="sm"
				type="colored"
				:color="monetized ? 'red' : 'brand'"
				:disabled="!canEdit || forceDisabled || mutation.isPending.value"
				@click="show('member', $event)"
				>{{ memberAction }}</Button
			>
		</div>
		<NewModal ref="modal" :header="confirmationTitle" width="500px">
			<div class="flex flex-col gap-3 leading-normal">
				<template v-if="mode === 'moderator' && !forceDisabled">
					<p class="m-0">
						<IntlFormatted
							:message-id="messages.forceDisableDescription"
							:values="{ project: project.name }"
						>
							<template #strong="{ children }">
								<strong><component :is="() => children" /></strong>
							</template>
						</IntlFormatted>
					</p>
					<p class="m-0 text-secondary">
						{{ formatMessage(messages.forceDisableRestriction) }}
					</p>
				</template>
				<template v-else-if="mode === 'moderator'">
					<p class="m-0">
						<IntlFormatted
							:message-id="messages.allowDescription"
							:values="{ project: project.name }"
						>
							<template #strong="{ children }">
								<strong><component :is="() => children" /></strong>
							</template>
						</IntlFormatted>
					</p>
					<p class="m-0 text-secondary">
						{{ formatMessage(messages.allowRestriction) }}
					</p>
				</template>
				<template v-else-if="monetized">
					<p class="m-0">
						<IntlFormatted
							:message-id="messages.disableDescription"
							:values="{ project: project.name }"
						>
							<template #strong="{ children }">
								<strong><component :is="() => children" /></strong>
							</template>
						</IntlFormatted>
					</p>
					<p class="m-0 text-secondary">{{ formatMessage(messages.disableRestriction) }}</p>
				</template>
				<p v-else class="m-0">
					<IntlFormatted
						:message-id="messages.enableDescription"
						:values="{ project: project.name }"
					>
						<template #strong="{ children }">
							<strong><component :is="() => children" /></strong>
						</template>
					</IntlFormatted>
				</p>
				<p v-if="mutation.isError.value" role="alert" class="m-0 text-red">
					{{ mutation.error.value?.message }}
				</p>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button :disabled="mutation.isPending.value" @click="modal?.hide()">{{
						formatMessage(commonMessages.cancelButton)
					}}</Button
					><Button
						type="colored"
						:color="monetized || (mode === 'moderator' && !forceDisabled) ? 'red' : 'brand'"
						:disabled="!canConfirm"
						:loading="mutation.isPending.value"
						@click="mutation.mutate()"
						>{{ action }}</Button
					>
				</div>
			</template>
		</NewModal>
	</div>
</template>
