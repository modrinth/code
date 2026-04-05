<script setup>
import { Button, injectNotificationManager, ProjectCard } from '@modrinth/ui'
import { ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { get_project_v3, get_version } from '@/helpers/cache.js'
import { injectContentInstall } from '@/providers/content-install'

const { handleError } = injectNotificationManager()
const { install: installVersion } = injectContentInstall()

const confirmModal = ref(null)
const project = ref(null)
const version = ref(null)
const installing = ref(false)

defineExpose({
	async show(event) {
		if (event.event === 'InstallVersion') {
			version.value = await get_version(event.id, 'must_revalidate').catch(handleError)
			project.value = await get_project_v3(version.value.project_id, 'must_revalidate').catch(
				handleError,
			)
		} else {
			project.value = await get_project_v3(event.id, 'must_revalidate').catch(handleError)
			version.value = await get_version(
				project.value.versions[project.value.versions.length - 1],
				'must_revalidate',
			).catch(handleError)
		}
		confirmModal.value.show()
	},
})

async function install() {
	confirmModal.value.hide()
	await installVersion(
		project.value.id,
		version.value.id,
		null,
		'URLConfirmModal',
		() => {},
		() => {},
	).catch(handleError)
}
</script>

<template>
	<ModalWrapper ref="confirmModal" :header="`Install ${project?.name}`">
		<div class="modal-body">
			<ProjectCard
				:title="project.name"
				:link="() => confirmModal.hide()"
				:icon-url="project.icon_url"
				:summary="project.summary"
				:tags="project.display_categories"
				:all-tags="project.categories"
				:downloads="project.downloads"
				:followers="project.follows"
				:date-updated="project.date_modified"
				:banner="project.featured_gallery ?? undefined"
				:color="project.color ?? undefined"
				layout="list"
				class="project-card"
			/>
			<div class="button-row">
				<div class="markdown-body">
					<p>
						Installing <code>{{ version.id }}</code> from Modrinth
					</p>
				</div>
				<div class="button-group">
					<Button :loading="installing" color="primary" @click="install">Install</Button>
				</div>
			</div>
		</div>
	</ModalWrapper>
</template>

<style scoped lang="scss">
.modal-body {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: var(--gap-md);
}

.button-row {
	width: 100%;
	display: flex;
	flex-direction: row;
	justify-content: space-between;
	align-items: center;
	gap: var(--gap-md);
}

.button-group {
	display: flex;
	flex-direction: row;
	gap: var(--gap-sm);
}

.project-card {
	background-color: var(--color-bg);
	width: 100%;

	:deep(.badge) {
		border: 1px solid var(--color-raised-bg);
		background-color: var(--color-accent-contrast);
	}
}
</style>
