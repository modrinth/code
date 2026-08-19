<script setup lang="ts">
import Toggle from '#ui/components/base/Toggle.vue'
import { useVIntl } from '#ui/composables'

import AppearanceSettingsThemeSelector from './components/appearance-settings-theme-selector.vue'
import AppearanceSettingRow from './components/AppearanceSettingRow.vue'
import ProjectLayoutSelector from './components/ProjectLayoutSelector.vue'
import { appearanceSettingsMessages as messages } from './messages'
import { injectAppearanceSettings } from './providers'

const appearance = injectAppearanceSettings()
const { formatMessage } = useVIntl()

const theme = appearance.theme
const currentTheme = theme.current
const themeOptions = theme.options
const systemTheme = theme.system
const syncAcrossDevices = theme.syncAcrossDevices.value
const syncDisabled = theme.syncAcrossDevices.disabled
const advancedRendering = appearance.advancedRendering.value
const nativeDecorations = appearance.nativeDecorations
const nativeDecorationsValue = nativeDecorations?.value
const projectLayouts = appearance.projectLayouts
const projectLayoutValues = projectLayouts?.value
const externalLinksNewTab = appearance.externalLinksNewTab
const externalLinksNewTabValue = externalLinksNewTab?.value
const sidebarPreferences = appearance.sidebarPreferences
const sidebarPreferenceValues = sidebarPreferences?.value
</script>

<template>
	<div>
		<section>
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.colorThemeTitle) }}
				</h2>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.colorThemeDescription) }}
				</p>
			</div>

			<AppearanceSettingsThemeSelector
				class="mt-4"
				:aria-label="formatMessage(messages.colorThemeTitle)"
				:model-value="currentTheme"
				:theme-options="themeOptions"
				:system-theme-color="systemTheme"
				@update:model-value="theme.update"
			/>

			<AppearanceSettingRow
				class="mt-6"
				control-id="sync-theme-across-devices"
				:heading-level="3"
				:title="formatMessage(messages.syncAcrossDevicesTitle)"
				:description="formatMessage(messages.syncAcrossDevicesDescription)"
			>
				<template #default="{ labelledBy }">
					<span
						v-tooltip="
							syncDisabled ? formatMessage(messages.syncAcrossDevicesSignedOutTooltip) : undefined
						"
						class="inline-flex"
					>
						<Toggle
							id="sync-theme-across-devices"
							:model-value="syncDisabled ? false : syncAcrossDevices"
							:disabled="syncDisabled"
							:aria-labelledby="labelledBy"
							@update:model-value="theme.syncAcrossDevices.update"
						/>
					</span>
				</template>
			</AppearanceSettingRow>
		</section>

		<section v-if="projectLayouts" class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.projectListLayoutsTitle) }}
				</h2>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.projectListLayoutsDescription) }}
				</p>
			</div>
			<div class="mt-4 flex flex-col gap-6">
				<ProjectLayoutSelector
					v-for="projectLayout in projectLayoutValues"
					:key="projectLayout.type"
					:title="formatMessage(messages[projectLayout.type])"
					:model-value="projectLayout.layout"
					@update:model-value="projectLayouts.update(projectLayout.type, $event)"
				/>
			</div>
		</section>

		<div class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<div class="flex flex-col gap-6">
				<AppearanceSettingRow
					control-id="advanced-rendering"
					:title="formatMessage(messages.advancedRenderingTitle)"
					:description="formatMessage(messages.advancedRenderingDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="advanced-rendering"
							:model-value="advancedRendering"
							:aria-labelledby="labelledBy"
							@update:model-value="appearance.advancedRendering.update"
						/>
					</template>
				</AppearanceSettingRow>

				<AppearanceSettingRow
					v-if="externalLinksNewTab"
					control-id="external-links-new-tab"
					:title="formatMessage(messages.externalLinksNewTabTitle)"
					:description="formatMessage(messages.externalLinksNewTabDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="external-links-new-tab"
							:model-value="externalLinksNewTabValue ?? false"
							:aria-labelledby="labelledBy"
							@update:model-value="externalLinksNewTab.update"
						/>
					</template>
				</AppearanceSettingRow>

				<template v-if="sidebarPreferences">
					<AppearanceSettingRow
						control-id="search-layout-toggle"
						:title="formatMessage(messages.rightAlignedFiltersSidebarTitle)"
						:description="formatMessage(messages.rightAlignedFiltersSidebarDescription)"
					>
						<template #default="{ labelledBy }">
							<Toggle
								id="search-layout-toggle"
								:model-value="sidebarPreferenceValues?.right_aligned_search ?? false"
								:aria-labelledby="labelledBy"
								@update:model-value="sidebarPreferences.update('right_aligned_search', $event)"
							/>
						</template>
					</AppearanceSettingRow>

					<AppearanceSettingRow
						control-id="project-layout-toggle"
						:title="formatMessage(messages.leftAlignedContentSidebarTitle)"
						:description="formatMessage(messages.leftAlignedContentSidebarDescription)"
					>
						<template #default="{ labelledBy }">
							<Toggle
								id="project-layout-toggle"
								:model-value="sidebarPreferenceValues?.left_aligned_content ?? false"
								:aria-labelledby="labelledBy"
								@update:model-value="sidebarPreferences.update('left_aligned_content', $event)"
							/>
						</template>
					</AppearanceSettingRow>
				</template>

				<AppearanceSettingRow
					v-if="nativeDecorations"
					control-id="native-decorations"
					:title="formatMessage(messages.nativeDecorationsTitle)"
					:description="formatMessage(messages.nativeDecorationsDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="native-decorations"
							:model-value="nativeDecorationsValue ?? false"
							:aria-labelledby="labelledBy"
							@update:model-value="nativeDecorations.update"
						/>
					</template>
				</AppearanceSettingRow>
			</div>
		</div>
	</div>
</template>
