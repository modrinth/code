<template>
	<div
		v-if="
			project.project_type !== 'plugin' ||
			project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
		"
		class="modrinth-app-section contents"
	>
		<div class="flex flex-col items-center">
			<ButtonStyled color="brand">
				<a
					class="!min-h-10 w-fit no-underline"
					:href="`modrinth://mod/${project.slug}`"
					@click="installWithApp"
				>
					<ModrinthIcon aria-hidden="true" />
					<span class="min-w-0 text-center">
						{{ formatMessage(messages.installWithModrinthApp) }}
					</span>
				</a>
			</ButtonStyled>
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

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ModrinthIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { ref } from 'vue'

import Accordion from '~/components/ui/Accordion.vue'

defineOptions({
	name: 'InstallWithModrinthApp',
})

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

defineProps<{
	project: DownloadModalProject
}>()

const { formatMessage } = useVIntl()
const tags = useGeneratedState()
const getModrinthAppAccordion = ref<InstanceType<typeof Accordion> | null>(null)

const messages = defineMessages({
	installWithModrinthApp: {
		id: 'project.download.install-with-app',
		defaultMessage: 'Install with Modrinth App',
	},
	dontHaveModrinthApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Modrinth App?",
	},
	downloadManually: {
		id: 'project.download.manually',
		defaultMessage: 'Download manually',
	},
})

function installWithApp() {
	setTimeout(() => {
		getModrinthAppAccordion.value?.open()
	}, 1500)
}
</script>

<style lang="scss" scoped>
@media (hover: none) and (max-width: 767px) {
	.modrinth-app-section {
		display: none;
	}
}
</style>
