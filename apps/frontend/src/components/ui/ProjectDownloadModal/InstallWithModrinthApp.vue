<template>
	<div
		v-if="
			project.project_type !== 'plugin' ||
			project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
		"
		class="modrinth-app-section contents"
	>
		<div class="flex flex-col">
			<a
				class="modrinth-app-install-card flex items-center justify-between gap-3 rounded-2xl border border-solid border-brand-highlight bg-surface-1 px-4 py-3 text-primary no-underline transition-[filter] hover:brightness-110"
				:href="`modrinth://mod/${project.slug}`"
				@click="installWithApp"
			>
				<span class="flex w-full min-w-0 flex-col gap-1">
					<div class="flex items-center justify-between">
						<span class="flex min-w-0 items-center gap-1.5 font-medium text-contrast">
							Install with
							<span class="text-brand">Modrinth App</span>
							<ModrinthIcon aria-hidden="true" class="size-4 flex-shrink-0 text-brand" />
						</span>
						<ExternalIcon
							aria-hidden="true"
							class="size-4 flex-shrink-0 text-contrast transition-colors"
						/>
					</div>
					<span class="truncate text-base text-secondary">
						{{ formatMessage(messages.installWithModrinthAppDescription) }}
					</span>
				</span>
			</a>
			<Accordion ref="getModrinthAppAccordion">
				<nuxt-link class="mt-2 flex justify-center text-brand-blue hover:underline" to="/app">
					{{ formatMessage(messages.dontHaveModrinthApp) }}
				</nuxt-link>
			</Accordion>
		</div>

		<div class="flex items-center gap-4">
			<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
			<span class="flex-shrink-0 text-sm font-medium text-secondary">
				{{ formatMessage(messages.downloadManually) }}
			</span>
			<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
		</div>
	</div>
</template>

<script setup>
import { ExternalIcon, ModrinthIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import Accordion from '~/components/ui/Accordion.vue'

defineOptions({
	name: 'InstallWithModrinthApp',
})

defineProps({
	project: {
		type: Object,
		required: true,
	},
	tags: {
		type: Object,
		required: true,
	},
})

const { formatMessage } = useVIntl()
const getModrinthAppAccordion = ref()

const messages = defineMessages({
	dontHaveModrinthApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Modrinth App?",
	},
	downloadManually: {
		id: 'project.download.manually',
		defaultMessage: 'Download manually',
	},
	installWithModrinthAppDescription: {
		id: 'project.download.install-with-app-description',
		defaultMessage: 'Automatically install the correct version and dependencies.',
	},
})

function installWithApp() {
	setTimeout(() => {
		getModrinthAppAccordion.value?.open()
	}, 1500)
}
</script>

<style lang="scss" scoped>
.modrinth-app-install-card {
	background: radial-gradient(circle at 50% 300%, #0d2f17 0%, var(--surface-1) 72%);
}

@media (hover: none) and (max-width: 767px) {
	.modrinth-app-section {
		display: none;
	}
}
</style>
