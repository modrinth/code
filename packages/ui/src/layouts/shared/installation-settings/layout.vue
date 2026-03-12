<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftRightIcon,
	CircleAlertIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	HammerIcon,
	PencilIcon,
	SaveIcon,
	SpinnerIcon,
	UnlinkIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Chips from '#ui/components/base/Chips.vue'
import Combobox from '#ui/components/base/Combobox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import ContentDiffModal from './components/ContentDiffModal.vue'
import ConfirmModpackUpdateModal from '../content-tab/components/modals/ConfirmModpackUpdateModal.vue'
import ConfirmReinstallModal from '../content-tab/components/modals/ConfirmReinstallModal.vue'
import ConfirmRepairModal from '../content-tab/components/modals/ConfirmRepairModal.vue'
import ConfirmUnlinkModal from '../content-tab/components/modals/ConfirmUnlinkModal.vue'
import ConfirmLeaveModal from '../content-tab/components/modals/ConfirmLeaveModal.vue'
import ContentUpdaterModal from '../content-tab/components/modals/ContentUpdaterModal.vue'
import { useInstallationForm } from './composables'
import { injectInstallationSettings } from './providers/installation-settings'

const { formatMessage } = useVIntl()
const ctx = injectInstallationSettings()

const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal>>()
const repairModal = ref<InstanceType<typeof ConfirmRepairModal>>()
const reinstallModal = ref<InstanceType<typeof ConfirmReinstallModal>>()
const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()

const contentDiffModal = ref<InstanceType<typeof ContentDiffModal>>()
const modpackUpdateModal = ref<InstanceType<typeof ConfirmModpackUpdateModal>>()
const pendingUpdateVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const isUpdateDowngrade = ref(false)

const form = useInstallationForm(ctx, contentUpdaterModal, contentDiffModal)

function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (form.isSaving.value) {
		e.preventDefault()
		return ''
	}
}

if (typeof window !== 'undefined') {
	watch(() => form.isSaving.value, (saving) => {
		if (saving) {
			window.addEventListener('beforeunload', handleBeforeUnload)
		} else {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		}
	})

	onBeforeUnmount(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload)
	})

	onBeforeRouteLeave(async () => {
		if (form.isSaving.value) {
			return (await confirmLeaveModal.value?.prompt()) ?? false
		}
		return true
	})
}

const disabledPlatforms = computed(() => {
	if (!ctx.lockPlatform || ctx.currentPlatform.value === 'vanilla') return []
	return ctx.availablePlatforms.filter((p) => p !== ctx.currentPlatform.value)
})

const showModpackVersionActions = ctx.showModpackVersionActions ?? true

function handleModpackUpdateRequest(version: Labrinth.Versions.v2.Version) {
	pendingUpdateVersion.value = version
	const currentVersionId = ctx.updaterModalProps.value.currentVersionId
	const currentVersion = form.updatingProjectVersions.value.find((v) => v.id === currentVersionId)
	isUpdateDowngrade.value = currentVersion
		? new Date(version.date_published) < new Date(currentVersion.date_published)
		: false
	modpackUpdateModal.value?.show()
}

function handleModpackUpdateConfirm() {
	if (pendingUpdateVersion.value) {
		form.cancelEditing()
		form.handleUpdaterConfirm(pendingUpdateVersion.value)
		pendingUpdateVersion.value = null
	}
}

function handleModpackUpdateCancel() {
	pendingUpdateVersion.value = null
}

function handleRepair() {
	form.cancelEditing()
	ctx.repair()
}

function handleReinstall() {
	form.cancelEditing()
	ctx.reinstallModpack()
}

function handleUnlink() {
	form.cancelEditing()
	ctx.unlinkModpack()
}

defineExpose({
	cancelEditing: () => form.cancelEditing(),
})

const messages = defineMessages({
	linkedInstanceTitle: {
		id: 'installation-settings.linked-instance.title',
		defaultMessage: 'Linked {projectType, select, server {server project} other {modpack}}',
	},
	reinstallModpackTitle: {
		id: 'installation-settings.reinstall-modpack.title',
		defaultMessage: 'Re-install modpack',
	},
	reinstallModpackDescription: {
		id: 'installation-settings.reinstall-modpack.description',
		defaultMessage:
			"Re-installing the modpack resets the {type, select, server {server's} other {instance's}} content to its original state, removing any mods or content you have added.",
	},
	editInstallationTitle: {
		id: 'installation-settings.edit-installation.title',
		defaultMessage: 'Edit installation',
	},
	unlinkDescription: {
		id: 'installation-settings.unlink.description',
		defaultMessage:
			"Unlinking permanently disconnects this {type, select, server {server} other {instance}} from the {projectType, select, server {server} other {modpack}} project, allowing you to change the loader and Minecraft version, but you won't receive future updates.",
	},
	repairInstanceTitle: {
		id: 'installation-settings.repair.instance-title',
		defaultMessage: 'Repair instance',
	},
	repairInstanceDescription: {
		id: 'installation-settings.repair.instance-description',
		defaultMessage:
			'Reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors.',
	},
	repairServerTitle: {
		id: 'installation-settings.repair.server-title',
		defaultMessage: 'Repair server',
	},
	repairServerDescription: {
		id: 'installation-settings.repair.server-description',
		defaultMessage:
			'Reinstalls the loader and Minecraft dependencies without deleting your content. This may resolve issues if your server is not starting correctly.',
	},
	editWarningInstance: {
		id: 'installation-settings.edit.warning-instance',
		defaultMessage:
			"We don't recommend editing your installation settings after installing content. If you want to edit them, be cautious as it may cause issues.",
	},
	editWarningServer: {
		id: 'installation-settings.edit.warning-server',
		defaultMessage:
			"We don't recommend editing your installation settings after installing content. If you want to edit them reset your server.",
	},
	loaderVersionLabel: {
		id: 'installation-settings.loader-version',
		defaultMessage: '{loader} version',
	},
	searchGameVersionPlaceholder: {
		id: 'installation-settings.search-game-version',
		defaultMessage: 'Search game version...',
	},
	savingLabel: {
		id: 'installation-settings.saving',
		defaultMessage: 'Saving...',
	},
	verifyingLabel: {
		id: 'installation-settings.verifying',
		defaultMessage: 'Verifying...',
	},
	selectPlatformAriaLabel: {
		id: 'installation-settings.aria.select-platform',
		defaultMessage: 'Select platform',
	},
	selectGameVersionAriaLabel: {
		id: 'installation-settings.aria.select-game-version',
		defaultMessage: 'Select game version',
	},
	selectLoaderVersionAriaLabel: {
		id: 'installation-settings.aria.select-loader-version',
		defaultMessage: 'Select {loader} version',
	},
	reinstallingModpackButton: {
		id: 'installation-settings.reinstalling-modpack',
		defaultMessage: 'Reinstalling modpack',
	},
	unlinkButton: {
		id: 'installation-settings.unlink',
		defaultMessage: 'Unlink',
	},
	platformLockTooltip: {
		id: 'installation-settings.platform-lock-tooltip',
		defaultMessage: 'You will need to reset your server to switch loader.',
	},
	confirmVersionChangeHeader: {
		id: 'installation-settings.confirm-version-change-header',
		defaultMessage: 'Review content changes',
	},
	confirmVersionChange: {
		id: 'installation-settings.confirm-version-change',
		defaultMessage: 'Confirm',
	},
	confirmVersionChangeDescription: {
		id: 'installation-settings.confirm-version-change-description',
		defaultMessage:
			'Changing to {gameVersion} will modify the following content on your server.',
	},
	removedIncompatible: {
		id: 'installation-settings.removed-incompatible',
		defaultMessage: 'Removed (Incompatible)',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<!-- Loading state -->
		<div v-if="ctx.loading.value" class="flex items-center justify-center py-12">
			<SpinnerIcon class="size-8 animate-spin text-secondary" />
		</div>

		<template v-else>
			<!-- Installation Info (linked state) -->
			<div v-if="ctx.isLinked.value" class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(commonMessages.installationInfoTitle) }}
				</span>
				<div class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
					<div
						v-for="row in ctx.installationInfo.value"
						:key="row.label"
						class="flex items-center justify-between"
					>
						<span class="text-primary">{{ row.label }}</span>
						<span v-if="row.value" class="font-semibold text-contrast">{{ row.value }}</span>
						<span v-else class="inline-block h-3 w-16 animate-pulse rounded bg-button-border"></span>
					</div>
				</div>
			</div>

			<!-- LINKED -->
			<template v-if="ctx.isLinked.value">
				<!-- Installed Modpack -->
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(commonMessages.installedModpackTitle) }}
					</span>
					<div
						v-if="ctx.modpack.value"
						class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3"
					>
						<AutoLink :to="ctx.modpack.value.link" class="shrink-0">
							<div
								class="size-14 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
							>
								<Avatar
									v-if="ctx.modpack.value.iconUrl"
									:src="ctx.modpack.value.iconUrl"
									:alt="ctx.modpack.value.title"
									size="100%"
									no-shadow
								/>
							</div>
						</AutoLink>
						<div class="flex flex-col gap-1">
							<AutoLink
								:to="ctx.modpack.value.link"
								class="font-semibold text-contrast hover:underline"
							>
								{{ ctx.modpack.value.title }}
							</AutoLink>
							<div class="flex items-center gap-2 text-sm text-secondary">
								<AutoLink
									v-if="ctx.modpack.value.owner"
									:to="
										ctx.modpack.value.owner.type === 'organization'
											? `/organization/${ctx.modpack.value.owner.id}`
											: `/user/${ctx.modpack.value.owner.id}`
									"
									class="flex items-center gap-1.5 hover:underline"
								>
									<Avatar
										:src="ctx.modpack.value.owner.iconUrl"
										:alt="ctx.modpack.value.owner.name"
										size="1.25rem"
										:circle="ctx.modpack.value.owner.type === 'user'"
										no-shadow
									/>
									<span class="font-medium">{{ ctx.modpack.value.owner.name }}</span>
								</AutoLink>
								<template v-if="ctx.modpack.value.owner && ctx.modpack.value.versionNumber">
									&middot;
								</template>
								<span v-if="ctx.modpack.value.versionNumber" class="font-medium">
									{{ ctx.modpack.value.versionNumber }}
								</span>
							</div>
						</div>
					</div>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled v-if="showModpackVersionActions">
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="form.handleChangeModpackVersion()"
							>
								<ArrowLeftRightIcon class="size-5" />
								{{ formatMessage(commonMessages.changeVersionButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<!-- Unlink -->
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{
							formatMessage(messages.linkedInstanceTitle, {
								projectType: showModpackVersionActions ? 'modpack' : 'server',
							})
						}}
					</span>
					<span class="text-primary">
						{{
							formatMessage(messages.unlinkDescription, {
								type: ctx.isServer ? 'server' : 'instance',
								projectType: showModpackVersionActions ? 'modpack' : 'server',
							})
						}}
					</span>
					<div>
						<ButtonStyled color="orange">
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="unlinkModal?.show()"
							>
								<UnlinkIcon class="size-5" />
								{{
									formatMessage(
										showModpackVersionActions
											? commonMessages.unlinkModpackButton
											: messages.unlinkButton,
									)
								}}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<!-- Reinstall -->
				<div v-if="showModpackVersionActions" class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.reinstallModpackTitle) }}
					</span>
					<span class="text-primary">
						{{
							formatMessage(messages.reinstallModpackDescription, {
								type: ctx.isServer ? 'server' : 'instance',
							})
						}}
					</span>
					<div>
						<ButtonStyled color="red">
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="reinstallModal?.show()"
							>
								<SpinnerIcon v-if="ctx.reinstalling?.value" class="animate-spin" />
								<DownloadIcon v-else class="size-5" />
								{{
									ctx.reinstalling?.value
										? formatMessage(messages.reinstallingModpackButton)
										: formatMessage(commonMessages.reinstallModpackButton)
								}}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<!-- Repair -->
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{
							formatMessage(
								ctx.isServer ? messages.repairServerTitle : messages.repairInstanceTitle,
							)
						}}
					</span>
					<span class="text-primary">
						{{
							formatMessage(
								ctx.isServer
									? messages.repairServerDescription
									: messages.repairInstanceDescription,
							)
						}}
					</span>
					<div>
						<ButtonStyled>
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="repairModal?.show()"
							>
								<SpinnerIcon v-if="ctx.repairing?.value" class="animate-spin" />
								<HammerIcon v-else class="size-5" />
								{{
									ctx.repairing?.value
										? formatMessage(commonMessages.repairingButton)
										: formatMessage(commonMessages.repairButton)
								}}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<!-- NOT LINKED -->
			<template v-else>
				<!-- Edit form -->
				<div v-if="form.isEditing.value" class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.editInstallationTitle) }}
					</span>
					<div class="flex flex-col gap-3 rounded-[20px] border border-solid border-surface-5 p-4">
						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">
								{{ formatMessage(commonMessages.platformLabel) }}
							</span>
							<Chips
								v-model="form.selectedPlatform.value"
								:items="ctx.availablePlatforms"
								:disabled-items="disabledPlatforms"
								:disabled-tooltip="formatMessage(messages.platformLockTooltip)"
								:aria-label="formatMessage(messages.selectPlatformAriaLabel)"
							/>
						</div>

						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">
								{{ formatMessage(commonMessages.gameVersionLabel) }}
							</span>
							<Combobox
								v-model="form.selectedGameVersion.value"
								:options="form.gameVersionOptions.value"
								searchable
								sync-with-selection
								:placeholder="formatMessage(commonMessages.selectVersionPlaceholder)"
								:search-placeholder="formatMessage(messages.searchGameVersionPlaceholder)"
								:display-value="
									form.selectedGameVersion.value ||
									formatMessage(commonMessages.selectVersionPlaceholder)
								"
								:aria-label="formatMessage(messages.selectGameVersionAriaLabel)"
							>
								<template v-if="form.hasSnapshots.value" #dropdown-footer>
									<button
										class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
										@mousedown.prevent
										@click="form.showSnapshots.value = !form.showSnapshots.value"
									>
										<EyeOffIcon v-if="form.showSnapshots.value" class="size-4" />
										<EyeIcon v-else class="size-4" />
										{{
											form.showSnapshots.value
												? formatMessage(commonMessages.hideSnapshotsButton)
												: formatMessage(commonMessages.showAllVersionsButton)
										}}
									</button>
								</template>
							</Combobox>
						</div>

						<div v-if="form.selectedPlatform.value !== 'vanilla' && !ctx.hideLoaderVersion" class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">
								{{
									formatMessage(messages.loaderVersionLabel, {
										loader: form.formattedLoaderName.value,
									})
								}}
							</span>
							<Combobox
								v-model="form.selectedLoaderVersion.value"
								searchable
								sync-with-selection
								:placeholder="
									form.loaderVersionDisplayValue.value ||
									formatMessage(commonMessages.selectVersionPlaceholder)
								"
								:search-placeholder="formatMessage(commonMessages.searchVersionPlaceholder)"
								:options="form.loaderVersionOptions.value"
								:display-value="
									form.loaderVersionDisplayValue.value ||
									formatMessage(commonMessages.selectVersionPlaceholder)
								"
								:aria-label="
									formatMessage(messages.selectLoaderVersionAriaLabel, {
										loader: form.formattedLoaderName.value,
									})
								"
							/>
						</div>

						<div class="flex flex-wrap gap-2">
							<ButtonStyled color="brand">
								<button
									class="!shadow-none"
									:disabled="!form.isValid.value || !form.hasChanges.value || form.isSaving.value"
									@click="form.save()"
								>
									<SpinnerIcon v-if="form.isSaving.value" class="animate-spin" />
									<SaveIcon v-else />
									{{
										form.isVerifying.value
											? formatMessage(messages.verifyingLabel)
											: form.isSaving.value
												? formatMessage(messages.savingLabel)
												: formatMessage(commonMessages.saveButton)
									}}
								</button>
							</ButtonStyled>
							<ButtonStyled type="outlined">
								<button
									class="!border !border-surface-5 !shadow-none"
									@click="form.cancelEditing()"
								>
									<XIcon />
									{{ formatMessage(commonMessages.cancelButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>

				<!-- Non-editing: installation info + warning + edit button -->
				<div v-if="!form.isEditing.value" class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(commonMessages.installationInfoTitle) }}
					</span>
					<div class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
						<div
							v-for="row in ctx.installationInfo.value"
							:key="row.label"
							class="flex items-center justify-between"
						>
							<span class="text-primary">{{ row.label }}</span>
							<span class="font-semibold text-contrast">{{ row.value }}</span>
						</div>
					</div>
					<div class="flex items-start gap-2">
						<CircleAlertIcon class="mt-0.5 size-5 shrink-0 text-orange" />
						<span class="text-primary">
							{{
								formatMessage(
									ctx.isServer ? messages.editWarningServer : messages.editWarningInstance,
								)
							}}
						</span>
					</div>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled color="orange">
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="form.isEditing.value = true"
							>
								<PencilIcon class="size-5" />
								{{ formatMessage(commonMessages.editButton) }}
							</button>
						</ButtonStyled>
						<slot name="unlinked-extra-buttons" />
					</div>
				</div>

				<!-- Repair section -->
				<div v-if="ctx.currentPlatform.value !== 'vanilla'" class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">
						{{
							formatMessage(
								ctx.isServer ? messages.repairServerTitle : messages.repairInstanceTitle,
							)
						}}
					</span>
					<span class="text-primary">
						{{
							formatMessage(
								ctx.isServer
									? messages.repairServerDescription
									: messages.repairInstanceDescription,
							)
						}}
					</span>
					<div>
						<ButtonStyled>
							<button
								class="!shadow-none"
								:disabled="ctx.isBusy.value"
								@click="repairModal?.show()"
							>
								<SpinnerIcon v-if="ctx.repairing?.value" class="animate-spin" />
								<HammerIcon v-else class="size-5" />
								{{
									ctx.repairing?.value
										? formatMessage(commonMessages.repairingButton)
										: formatMessage(commonMessages.repairButton)
								}}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<slot name="extra" />
		</template>
	</div>

	<!-- Modals -->
	<Teleport to="body">
		<ContentUpdaterModal
			v-if="form.updatingModpack.value"
			ref="contentUpdaterModal"
			:versions="form.updatingProjectVersions.value"
			:current-game-version="ctx.updaterModalProps.value.currentGameVersion"
			:current-loader="ctx.updaterModalProps.value.currentLoader"
			:current-version-id="ctx.updaterModalProps.value.currentVersionId"
			:is-app="ctx.isApp"
			:is-modpack="true"
			:project-icon-url="ctx.updaterModalProps.value.projectIconUrl"
			:project-name="ctx.updaterModalProps.value.projectName"
			:loading="form.loadingVersions.value"
			:loading-changelog="form.loadingChangelog.value"
			@update="handleModpackUpdateRequest"
			@cancel="form.resetUpdateState()"
			@version-select="form.handleUpdaterVersionSelect"
			@version-hover="form.handleUpdaterVersionHover"
		/>
		<ConfirmModpackUpdateModal
			ref="modpackUpdateModal"
			:downgrade="isUpdateDowngrade"
			:server="ctx.isServer"
			@confirm="handleModpackUpdateConfirm"
			@cancel="handleModpackUpdateCancel"
		/>
		<ConfirmRepairModal ref="repairModal" :server="ctx.isServer" @repair="handleRepair" />
		<ConfirmReinstallModal
			ref="reinstallModal"
			:server="ctx.isServer"
			@reinstall="handleReinstall"
		/>
		<ConfirmUnlinkModal ref="unlinkModal" :server="ctx.isServer" @unlink="handleUnlink" />

		<ContentDiffModal
			v-if="form.pendingPreview.value"
			ref="contentDiffModal"
			:header="formatMessage(messages.confirmVersionChangeHeader)"
			:description="
				formatMessage(messages.confirmVersionChangeDescription, {
					gameVersion: form.pendingPreview.value.newGameVersion,
				})
			"
			:admonition-header="formatMessage(messages.confirmVersionChangeHeader)"
			:diffs="form.pendingPreview.value.diffs"
			:has-unknown-content="form.pendingPreview.value.hasUnknownContent"
			:confirm-label="formatMessage(messages.confirmVersionChange)"
			:confirm-icon="SaveIcon"
			:removed-label="formatMessage(messages.removedIncompatible)"
			:show-backup-creator="ctx.isServer"
			@confirm="form.confirmSave()"
			@cancel="form.cancelPreview()"
		/>

		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<slot name="extra-modals" />
	</Teleport>
</template>
