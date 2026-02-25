<script setup lang="ts">
import { ArrowLeftRightIcon, EyeIcon, EyeOffIcon, SpinnerIcon, UnlinkIcon } from '@modrinth/assets'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectInstallationSettings } from '../../providers/installation-settings'
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import BulletDivider from '../base/BulletDivider.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Chips from '../base/Chips.vue'
import Combobox from '../base/Combobox.vue'

const { formatMessage } = useVIntl()
const ctx = injectInstallationSettings()

const messages = defineMessages({
	installedModpack: {
		id: 'installation-settings.installed-modpack',
		defaultMessage: 'Installed modpack',
	},
	installationInfo: {
		id: 'installation-settings.installation-info',
		defaultMessage: 'Installation info',
	},
	changeVersion: {
		id: 'installation-settings.change-version',
		defaultMessage: 'Change version',
	},
	unlink: {
		id: 'installation-settings.unlink',
		defaultMessage: 'Unlink',
	},
	platform: {
		id: 'installation-settings.platform',
		defaultMessage: 'Platform',
	},
	gameVersion: {
		id: 'installation-settings.game-version',
		defaultMessage: 'Game version',
	},
	loaderVersion: {
		id: 'installation-settings.loader-version',
		defaultMessage: '{loader} version',
	},
	showAllVersions: {
		id: 'installation-settings.show-all-versions',
		defaultMessage: 'Show all versions',
	},
	hideSnapshots: {
		id: 'installation-settings.hide-snapshots',
		defaultMessage: 'Hide snapshots',
	},
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<template v-if="ctx.isLinked.value">
			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.installedModpack) }}
				</span>
				<div
					v-if="ctx.modpack.value"
					class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3"
				>
					<AutoLink :to="ctx.modpack.value.projectLink" class="shrink-0">
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
							:to="ctx.modpack.value.projectLink"
							class="font-semibold text-contrast"
							:class="ctx.modpack.value.projectLink ? 'hover:underline' : ''"
						>
							{{ ctx.modpack.value.title }}
						</AutoLink>
						<div class="flex items-center gap-1 text-sm font-medium text-primary">
							<AutoLink
								v-if="ctx.modpack.value.owner"
								:to="ctx.modpack.value.owner.link"
								class="flex items-center gap-1 text-primary"
								:class="ctx.modpack.value.owner.link ? 'hover:underline' : ''"
							>
								<Avatar
									:src="ctx.modpack.value.owner.avatarUrl"
									:alt="ctx.modpack.value.owner.name"
									size="1.5rem"
									:circle="ctx.modpack.value.owner.type !== 'organization'"
									no-shadow
								/>
								<span>{{ ctx.modpack.value.owner.name }}</span>
							</AutoLink>
							<BulletDivider v-if="ctx.modpack.value.owner && ctx.modpack.value.versionName" />
							<AutoLink
								v-if="ctx.modpack.value.versionName"
								:to="ctx.modpack.value.versionLink"
								class="text-primary"
								:class="ctx.modpack.value.versionLink ? 'hover:underline' : ''"
							>
								{{ ctx.modpack.value.versionName }}
							</AutoLink>
						</div>
					</div>
				</div>

				<div class="flex flex-wrap gap-2">
					<ButtonStyled>
						<button :disabled="ctx.isBusy.value" @click="ctx.changeVersion()">
							<ArrowLeftRightIcon class="size-5" />
							{{ formatMessage(messages.changeVersion) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button :disabled="ctx.isBusy.value" @click="ctx.unlink()">
							<UnlinkIcon class="size-5" />
							{{ formatMessage(messages.unlink) }}
						</button>
					</ButtonStyled>
					<template v-if="ctx.extraLinkedActions?.value">
						<ButtonStyled
							v-for="(action, i) in ctx.extraLinkedActions.value"
							:key="i"
							:color="action.color"
						>
							<button
								v-tooltip="action.tooltip"
								:disabled="action.disabled || ctx.isBusy.value"
								@click="action.handler()"
							>
								<SpinnerIcon v-if="action.loading" class="animate-spin" />
								<component :is="action.icon" v-else class="size-5" />
								{{ action.loading && action.loadingLabel ? action.loadingLabel : action.label }}
							</button>
						</ButtonStyled>
					</template>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.installationInfo) }}
				</span>
				<div class="flex flex-col gap-2.5 overflow-clip rounded-[20px] bg-surface-2 p-4">
					<div
						v-for="row in ctx.installationInfo.value"
						:key="row.label"
						class="flex items-center justify-between"
					>
						<span class="text-primary">{{ row.label }}</span>
						<span class="font-semibold text-contrast">{{ row.value }}</span>
					</div>
				</div>
			</div>

			<slot name="linked-extra" />
		</template>

		<template v-else>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.installationInfo) }}
				</span>
				<div class="flex flex-col gap-3 rounded-[20px] border border-solid border-surface-5 p-4">
					<div class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.platform) }}
						</span>
						<Chips v-model="ctx.selectedPlatform!.value" :items="ctx.platforms?.value ?? []" />
					</div>

					<div class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.gameVersion) }}
						</span>
						<Combobox
							v-model="ctx.selectedGameVersion!.value"
							:options="ctx.gameVersionOptions?.value ?? []"
							searchable
							sync-with-selection
							placeholder="Select version"
							search-placeholder="Search game version..."
							:display-value="ctx.selectedGameVersion?.value || 'Select version'"
						>
							<template v-if="ctx.showSnapshots && ctx.hasSnapshots?.value" #dropdown-footer>
								<button
									class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
									@mousedown.prevent
									@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
								>
									<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
									<EyeIcon v-else class="size-4" />
									{{
										ctx.showSnapshots.value
											? formatMessage(messages.hideSnapshots)
											: formatMessage(messages.showAllVersions)
									}}
								</button>
							</template>
						</Combobox>
					</div>

					<div v-if="ctx.selectedPlatform?.value !== 'vanilla'" class="flex flex-col gap-2.5">
						<span class="font-semibold text-contrast">
							{{
								formatMessage(messages.loaderVersion, {
									loader: ctx.formattedLoaderName?.value,
								})
							}}
						</span>
						<Combobox
							v-model="ctx.selectedLoaderVersion!.value"
							searchable
							sync-with-selection
							:placeholder="ctx.loaderVersionDisplayValue?.value"
							search-placeholder="Search version..."
							:options="ctx.loaderVersionOptions?.value ?? []"
							:display-value="ctx.loaderVersionDisplayValue?.value"
						/>
					</div>

					<slot name="unlinked-extra" />

					<slot name="save-button" />
				</div>
			</div>
		</template>

		<slot name="extra" />
	</div>
</template>
