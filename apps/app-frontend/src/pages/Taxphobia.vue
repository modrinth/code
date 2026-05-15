<script setup lang="ts">
import { SearchIcon } from '@icarus/assets'
import { injectIcarusClient } from '@icarus/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { injectContentInstall } from '@/providers/content-install'
import { PlusIcon } from '@icarus/assets'
import ButtonStyled from '@icarus/ui/src/components/base/ButtonStyled.vue'
import LoadingIndicator from '@icarus/ui/src/components/base/LoadingIndicator.vue'
import StyledInput from '@icarus/ui/src/components/base/StyledInput.vue'
import ProjectCard from '@icarus/ui/src/components/project/card/ProjectCard.vue'
import ProjectCardList from '@icarus/ui/src/components/project/ProjectCardList.vue'

const client = injectIcarusClient()
const { install, installingItems } = injectContentInstall()

const { data: projects, isPending, isError, error } = useQuery({
	queryKey: ['taxphobia', 'fraa2a', 'projects'],
	queryFn: () => client.labrinth.users_v2.getProjects('fraa2a'),
})

const isInstalling = (projectId: string) => {
	for (const items of installingItems.value.values()) {
		if (items.some(item => item.project.id === projectId)) return true
	}
	return false
}

const handleInstall = async (projectId: string) => {
	await install(projectId, null, null, 'TaxphobiaStore').catch(console.error)
}

const searchQuery = ref('')

const filteredProjects = computed(() => {
	if (!projects.value) return []
	if (!searchQuery.value) return projects.value
	const q = searchQuery.value.toLowerCase()
	return projects.value.filter(
		(p) =>
			p.title.toLowerCase().includes(q) ||
			p.description?.toLowerCase().includes(q)
	)
})
</script>

<template>
	<div class="flex flex-col gap-4 p-6 overflow-y-auto w-full h-full max-w-7xl mx-auto">
		<div class="flex items-center justify-between mb-4">
			<div class="flex items-center gap-4">
				<h1 class="m-0 text-3xl font-extrabold text-primary">Store Taxphobia</h1>
				<a
					href="https://modrinth.com/user/fraa2a"
					target="_blank"
					class="text-sm font-semibold text-brand hover:underline mt-1"
				>
					View on Modrinth &rarr;
				</a>
			</div>

			<StyledInput
				v-model="searchQuery"
				:icon="SearchIcon"
				type="text"
				placeholder="Search Taxphobia..."
				clearable
				wrapper-class="w-72"
				input-class="h-10"
			/>
		</div>

		<div v-if="isPending" class="flex justify-center p-12">
			<LoadingIndicator />
		</div>

		<div v-else-if="isError" class="admonition admonition-error">
			Failed to load Taxphobia content. Please check your connection.
			<div class="mt-2 text-xs opacity-75 font-mono bg-bg-raised p-2 rounded">
				{{ error?.message || error }}
			</div>
		</div>

		<div v-else-if="projects && projects.length > 0" class="flex flex-col gap-4">
			<ProjectCardList layout="grid">
				<ProjectCard
					v-for="project in filteredProjects"
					:key="project.id"
					:link="{ path: `/project/${project.slug}` }"
					:title="project.title"
					:icon-url="project.icon_url"
					:author="{ name: 'Taxphobia', link: 'https://modrinth.com/user/fraa2a' }"
					:date-updated="project.updated"
					:date-published="project.published"
					displayed-date="updated"
					:downloads="project.downloads"
					:summary="project.description"
					:tags="project.categories"
					:all-tags="project.categories"
					:followers="project.followers"
					:color="project.color ?? undefined"
					layout="grid"
				>
					<template #actions>
						<div class="flex gap-2">
							<ButtonStyled color="brand" type="outlined">
								<button
									v-tooltip="'Install'"
									:disabled="isInstalling(project.id)"
									@click.prevent.stop="handleInstall(project.id)"
								>
									<PlusIcon :class="isInstalling(project.id) ? 'animate-spin' : ''" />
									Install
								</button>
							</ButtonStyled>
						</div>
					</template>
				</ProjectCard>
			</ProjectCardList>
		</div>

		<div v-else class="flex flex-col items-center gap-3 p-12 text-secondary">
			<p>No projects found from fraa2a.</p>
		</div>
	</div>
</template>

