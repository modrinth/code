<template>
	<ProjectCard
		:title="project.title"
		:link="
			() => {
				emit('open')
				$router.push({
					path: `/project/${project.project_id ?? project.id}`,
					query: { i: props.instance ? props.instance.path : undefined },
				})
			}
		"
		:author="{ name: project.author, link: `https://modrinth.com/user/${project.author}` }"
		:icon-url="project.icon_url"
		:summary="project.description"
		:tags="project.display_categories"
		:all-tags="project.categories"
		:downloads="project.downloads"
		:followers="project.follows"
		:date-updated="project.date_modified"
		:banner="project.featured_gallery ?? undefined"
		:color="project.color ?? undefined"
		:environment="
			projectType
				? ['mod', 'modpack'].includes(projectType)
					? {
							clientSide: project.client_side,
							serverSide: project.server_side,
						}
					: undefined
				: undefined
		"
		layout="list"
	>
		<template #actions>
			<ButtonStyled color="brand" type="outlined">
				<button
					:disabled="installed || installing"
					class="shrink-0 no-wrap"
					@click.stop="install()"
				>
					<template v-if="!installed">
						<DownloadIcon v-if="modpack || instance" />
						<PlusIcon v-else />
					</template>
					<CheckIcon v-else />
					{{
						installing
							? 'Installing'
							: installed
								? 'Installed'
								: modpack || instance
									? 'Install'
									: 'Add to an instance'
					}}
				</button>
			</ButtonStyled>
		</template>
	</ProjectCard>
</template>

<script setup>
import { CheckIcon, DownloadIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, ProjectCard } from '@modrinth/ui'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { install as installVersion } from '@/composables/content-install'

dayjs.extend(relativeTime)

const { handleError } = injectNotificationManager()

const router = useRouter()

const props = defineProps({
	backgroundImage: {
		type: String,
		default: null,
	},
	project: {
		type: Object,
		required: true,
	},
	instance: {
		type: Object,
		default: null,
	},
	featured: {
		type: Boolean,
		default: false,
	},
	installed: {
		type: Boolean,
		default: false,
	},
	projectType: {
		type: String,
		default: undefined,
	},
	activeLoader: {
		type: String,
		default: null,
	},
	activeGameVersion: {
		type: String,
		default: null,
	},
})

const emit = defineEmits(['open', 'install'])

const installing = ref(false)

async function install() {
	installing.value = true
	await installVersion(
		props.project.project_id ?? props.project.id,
		null,
		props.instance ? props.instance.path : null,
		'SearchCard',
		(versionId) => {
			installing.value = false
			if (versionId) {
				emit('install', props.project.project_id ?? props.project.id)
			}
		},
		(profile) => {
			router.push(`/instance/${profile}`)
		},
		{
			preferredLoader: props.activeLoader ?? undefined,
			preferredGameVersion: props.activeGameVersion ?? undefined,
		},
	).catch(handleError)
}

const modpack = computed(() => props.project.project_type === 'modpack')
</script>
