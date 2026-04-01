<template>
	<div>
		<MessageBanner v-if="flags.developerMode" message-type="warning" class="developer-message">
			<IntlFormatted :message-id="developerModeBanner.description">
				<template #strong="{ children }">
					<strong>
						<component :is="() => normalizeChildren(children)" />
					</strong>
				</template>
			</IntlFormatted>
			<ButtonStyled color="red">
				<button type="button" class="mt-2" @click="disableDeveloperMode">
					{{ formatMessage(developerModeBanner.deactivate) }}
				</button>
			</ButtonStyled>
		</MessageBanner>
		<section>
			<div class="flex flex-col gap-4">
				<div class="flex flex-row flex-wrap items-center justify-between gap-2">
					<Checkbox id="advanced-rendering" v-model="cosmetics.advancedRendering" />
					<label for="advanced-rendering" class="flex-1">
						<span class="mb-1 block font-bold text-contrast">
							{{ formatMessage(toggleFeatures.advancedRenderingTitle) }}
						</span>
						<span class="text-secondary">
							{{ formatMessage(toggleFeatures.advancedRenderingDescription) }}
						</span>
					</label>
				</div>
				<div class="flex flex-row flex-wrap items-center justify-between gap-2">
					<Checkbox
						id="external-links-new-tab"
						v-model="cosmetics.externalLinksNewTab"
						class="shrink-0"
					/>
					<label for="external-links-new-tab" class="flex-1">
						<span class="mb-1 block font-bold text-contrast">
							{{ formatMessage(toggleFeatures.externalLinksNewTabTitle) }}
						</span>
						<span class="text-secondary">
							{{ formatMessage(toggleFeatures.externalLinksNewTabDescription) }}
						</span>
					</label>
				</div>
				<div v-if="false" class="flex flex-row flex-wrap items-center justify-between gap-2">
					<Checkbox
						id="modrinth-app-promos"
						v-model="cosmetics.hideModrinthAppPromos"
						class="shrink-0"
					/>
					<label for="modrinth-app-promos" class="flex-1">
						<span class="mb-1 block font-bold text-contrast">
							{{ formatMessage(toggleFeatures.hideModrinthAppPromosTitle) }}
						</span>
						<span class="text-secondary">
							{{ formatMessage(toggleFeatures.hideModrinthAppPromosDescription) }}
						</span>
					</label>
				</div>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import {
	ButtonStyled,
	Checkbox,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	normalizeChildren,
	useVIntl,
} from '@modrinth/ui'

import MessageBanner from '~/components/ui/MessageBanner.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	headTitle: {
		id: 'settings.head-title',
		defaultMessage: 'Display settings',
	},
})

const developerModeBanner = defineMessages({
	description: {
		id: 'settings.display.banner.developer-mode.description',
		defaultMessage:
			"<strong>Developer mode</strong> is active. This will allow you to view the internal IDs of various things throughout Modrinth that may be helpful if you're a developer using the Modrinth API. Click on the Modrinth logo at the bottom of the page 5 times to toggle developer mode.",
	},
	deactivate: {
		id: 'settings.display.banner.developer-mode.button',
		defaultMessage: 'Deactivate developer mode',
	},
})

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

const notifications = defineMessages({
	developerModeDeactivatedTitle: {
		id: 'settings.display.notification.developer-mode-deactivated.title',
		defaultMessage: 'Developer mode deactivated',
	},
	developerModeDeactivatedText: {
		id: 'settings.display.notification.developer-mode-deactivated.text',
		defaultMessage: 'Developer mode has been disabled',
	},
})

const toggleFeatures = defineMessages({
	title: {
		id: 'settings.display.flags.title',
		defaultMessage: 'Toggle features',
	},
	description: {
		id: 'settings.display.flags.description',
		defaultMessage: 'Enable or disable certain features on this device.',
	},
	advancedRenderingTitle: {
		id: 'settings.display.sidebar.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'settings.display.sidebar.advanced-rendering.description',
		defaultMessage:
			'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
	},
	externalLinksNewTabTitle: {
		id: 'settings.display.sidebar.external-links-new-tab.title',
		defaultMessage: 'Open external links in new tab',
	},
	externalLinksNewTabDescription: {
		id: 'settings.display.sidebar.external-links-new-tab.description',
		defaultMessage:
			'Make links which go outside of Modrinth open in a new tab. No matter this setting, links on the same domain and in Markdown descriptions will open in the same tab, and links on ads and edit pages will open in a new tab.',
	},
	hideModrinthAppPromosTitle: {
		id: 'settings.display.sidebar.hide-app-promos.title',
		defaultMessage: 'Hide Modrinth App promotions',
	},
	hideModrinthAppPromosDescription: {
		id: 'settings.display.sidebar.hide-app-promos.description',
		defaultMessage:
			'Hides the "Get Modrinth App" buttons from primary navigation. The Modrinth App page can still be found on the landing page or in the footer.',
	},
	rightAlignedFiltersSidebarTitle: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.title',
		defaultMessage: 'Right-aligned filters sidebar on search pages',
	},
	rightAlignedFiltersSidebarDescription: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.description',
		defaultMessage: 'Aligns the filters sidebar to the right of the search results.',
	},
	leftAlignedContentSidebarTitle: {
		id: 'settings.display.sidebar.left-aligned-content-sidebar.title',
		defaultMessage: 'Left-aligned sidebar on content pages',
	},
	leftAlignedContentSidebarDescription: {
		id: 'settings.display.sidebar.right-aligned-content-sidebar.description',
		defaultMessage: "Aligns the sidebar to the left of the page's content.",
	},
})

const cosmetics = useCosmetics()
const flags = useFeatureFlags()

function disableDeveloperMode() {
	flags.value.developerMode = !flags.value.developerMode
	saveFeatureFlags()
	addNotification({
		title: formatMessage(notifications.developerModeDeactivatedTitle),
		text: formatMessage(notifications.developerModeDeactivatedText),
		type: 'success',
	})
}
</script>
<style scoped lang="scss">
.project-lists {
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);

	> :first-child .label__title {
		margin-top: 0;
	}

	.preview {
		--_layout-width: 7rem;
		--_layout-height: 4.5rem;
		--_layout-gap: 0.25rem;

		.example-card {
			border-radius: 0.5rem;
			width: var(--_layout-width);
			height: calc((var(--_layout-height) - 3 * var(--_layout-gap)) / 4);
			padding: 0;
		}

		.layout-list-mode {
			display: grid;
			grid-template-columns: 1fr;
			gap: var(--_layout-gap);
		}

		.layout-grid-mode {
			display: grid;
			grid-template-columns: 1fr 1fr 1fr;
			gap: var(--_layout-gap);

			.example-card {
				width: calc((var(--_layout-width) - 2 * var(--_layout-gap)) / 3);
				height: calc((var(--_layout-height) - var(--_layout-gap)) / 2);
			}
		}

		.layout-gallery-mode {
			display: grid;
			grid-template-columns: 1fr 1fr;
			gap: var(--_layout-gap);

			.example-card {
				width: calc((var(--_layout-width) - var(--_layout-gap)) / 2);
				height: calc((var(--_layout-height) - var(--_layout-gap)) / 2);
			}
		}
	}
}

.project-list-layouts {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(9.5rem, 1fr));
	gap: var(--gap-lg);

	.preview-radio .example-card {
		border: 2px solid transparent;
	}

	.preview-radio.selected .example-card {
		border-color: var(--color-brand);
		background-color: var(--color-brand-highlight);
	}

	.preview {
		display: flex;
		align-items: center;
		justify-content: center;
	}
}

.developer-message {
	svg {
		vertical-align: middle;
		margin-bottom: 2px;
		margin-right: 0.5rem;
	}

	.btn {
		margin-top: var(--gap-sm);
	}
}
</style>
