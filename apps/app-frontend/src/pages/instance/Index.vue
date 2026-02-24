<template>
	<div v-if="instance">
		<div class="p-6 pr-2 pb-4" @contextmenu.prevent.stop="(event) => handleRightClick(event)">
			<ExportModal ref="exportModal" :instance="instance" />
			<InstanceSettingsModal ref="settingsModal" :instance="instance" :offline="offline" />
			<UpdateToPlayModal ref="updateToPlayModal" :instance="instance" />
			<ContentPageHeader>
				<template #icon>
					<Avatar
						:src="icon ? icon : undefined"
						:alt="instance.name"
						size="64px"
						:tint-by="instance.path"
					/>
				</template>
				<template #title>
					{{ instance.name }}
					<TagItem
						v-if="isServerInstance"
						v-tooltip="
							`This instance's content is locked and managed by ${projectV3?.name || 'a server project'}`
						"
						class="border !border-solid border-blue bg-highlight-blue !font-medium"
						style="--_color: var(--color-blue)"
					>
						<LockIcon />
						Managed
					</TagItem>
				</template>
				<template #stats>
					<div class="flex items-center flex-wrap gap-2">
						<template v-if="!isServerInstance">
							<div class="flex items-center gap-2 capitalize font-medium">
								{{ instance.loader }} {{ instance.game_version }}
							</div>

							<div class="w-1.5 h-1.5 rounded-full bg-surface-5"></div>

							<div class="flex items-center gap-2 font-medium">
								<template v-if="timePlayed > 0">
									{{ timePlayedHumanized }}
								</template>
								<template v-else> Never played </template>
							</div>
						</template>

						<template v-else>
							<ServerOnlinePlayers v-if="playersOnline !== undefined" :online="playersOnline" />

							<div
								v-if="playersOnline !== undefined && (minecraftServer?.country || ping)"
								class="w-1.5 h-1.5 rounded-full bg-surface-5"
							></div>

							<ServerRegion v-if="minecraftServer?.country" :region="minecraftServer?.country" />

							<ServerPing v-if="ping" :ping="ping" />

							<div
								v-if="modpackContentProjectV3 && (minecraftServer?.country || ping)"
								class="w-1.5 h-1.5 rounded-full bg-surface-5"
							></div>

							<div v-if="projectV3" class="flex gap-1.5 items-center font-medium text-primary">
								Linked to
								<Avatar
									:src="projectV3.icon_url"
									:alt="projectV3.name"
									:tint-by="instance.path"
									size="24px"
								/>
								<router-link
									:to="`/project/${projectV3.slug ?? projectV3.id}`"
									class="hover:underline text-primary"
								>
									{{ projectV3.name }}
								</router-link>
							</div>
						</template>
					</div>
				</template>
				<template #actions>
					<div class="flex gap-2">
						<ButtonStyled
							v-if="
								['installing', 'pack_installing', 'minecraft_installing'].includes(
									instance.install_stage,
								)
							"
							color="brand"
							size="large"
						>
							<button disabled>Installing...</button>
						</ButtonStyled>
						<ButtonStyled
							v-else-if="instance.install_stage !== 'installed'"
							color="brand"
							size="large"
						>
							<button @click="repairInstance()">
								<DownloadIcon />
								Repair
							</button>
						</ButtonStyled>
						<ButtonStyled v-else-if="playing === true" color="red" size="large">
							<button @click="stopInstance('InstancePage')">
								<StopCircleIcon />
								Stop
							</button>
						</ButtonStyled>
						<ButtonStyled
							v-else-if="playing === false && loading === false && !isServerInstance"
							color="brand"
							size="large"
						>
							<button @click="startInstance('InstancePage')">
								<PlayIcon />
								Play
							</button>
						</ButtonStyled>
						<div
							v-else-if="playing === false && loading === false && isServerInstance"
							class="joined-buttons"
						>
							<ButtonStyled color="brand">
								<button @click="playServerProject(instance.linked_data?.project_id)">
									<PlayIcon />
									Play
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<OverflowMenu
									:options="[
										{
											id: 'join_server',
											action: () => playServerProject(instance?.linked_data?.project_id),
										},
										{
											id: 'launch_instance',
											action: () => startInstance('InstancePage'),
										},
									]"
								>
									<DropdownIcon />
									<template #join_server>
										<PlayIcon />
										Join server
									</template>
									<template #launch_instance>
										<PlayIcon />
										Launch instance
									</template>
								</OverflowMenu>
							</ButtonStyled>
						</div>
						<ButtonStyled
							v-else-if="loading === true && playing === false"
							color="brand"
							size="large"
						>
							<button disabled>Loading...</button>
						</ButtonStyled>
						<ButtonStyled size="large" circular>
							<button v-tooltip="'Instance settings'" @click="settingsModal?.show()">
								<SettingsIcon />
							</button>
						</ButtonStyled>
						<ButtonStyled size="large" type="transparent" circular>
							<OverflowMenu
								:options="[
									{
										id: 'open-folder',
										action: () => {
											if (instance) showProfileInFolder(instance.path)
										},
									},
									{
										id: 'export-mrpack',
										action: () => exportModal?.show(),
									},
								]"
							>
								<MoreVerticalIcon />
								<template #share-instance> <UserPlusIcon /> Share instance </template>
								<template #host-a-server> <ServerIcon /> Create a server </template>
								<template #open-folder> <FolderOpenIcon /> Open folder </template>
								<template #export-mrpack> <PackageIcon /> Export modpack </template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</template>
			</ContentPageHeader>
		</div>
		<div class="px-6">
			<NavTabs :links="tabs" />
		</div>
		<div v-if="!!instance" class="p-6 pt-4">
			<RouterView v-slot="{ Component }" :key="instance.path">
				<template v-if="Component">
					<Suspense
						:key="instance.path"
						@pending="loadingBar.startLoading()"
						@resolve="loadingBar.stopLoading()"
					>
						<component
							:is="Component"
							:instance="instance"
							:options="options"
							:offline="offline"
							:playing="playing"
							:versions="modrinthVersions"
							:installed="instance.install_stage !== 'installed'"
							@play="updatePlayState"
							@stop="() => stopInstance('InstanceSubpage')"
						></component>
						<template #fallback>
							<LoadingIndicator />
						</template>
					</Suspense>
				</template>
			</RouterView>
		</div>
		<ContextMenu ref="options" @option-clicked="handleOptionsClick">
			<template #play> <PlayIcon /> Play </template>
			<template #stop> <StopCircleIcon /> Stop </template>
			<template #add_content> <PlusIcon /> Add content </template>
			<template #edit> <EditIcon /> Edit </template>
			<template #copy_path> <ClipboardCopyIcon /> Copy path </template>
			<template #open_folder> <FolderOpenIcon /> Open folder </template>
			<template #copy_link> <ClipboardCopyIcon /> Copy link </template>
			<template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
			<template #copy_names><EditIcon />Copy names</template>
			<template #copy_slugs><HashIcon />Copy slugs</template>
			<template #copy_links><GlobeIcon />Copy links</template>
			<template #toggle><EditIcon />Toggle selected</template>
			<template #disable><XIcon />Disable selected</template>
			<template #enable><CheckCircleIcon />Enable selected</template>
			<template #hide_show><EyeIcon />Show/Hide unselected</template>
			<template #update_all
				><UpdatedIcon />Update {{ selected.length > 0 ? 'selected' : 'all' }}</template
			>
			<template #filter_update><UpdatedIcon />Select Updatable</template>
		</ContextMenu>
	</div>
</template>
<script setup lang="ts">
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	DropdownIcon,
	EditIcon,
	ExternalIcon,
	EyeIcon,
	FolderOpenIcon,
	GlobeIcon,
	HashIcon,
	LockIcon,
	MoreVerticalIcon,
	PackageIcon,
	PlayIcon,
	PlusIcon,
	ServerIcon,
	SettingsIcon,
	StopCircleIcon,
	UpdatedIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ContentPageHeader,
	injectNotificationManager,
	LoadingIndicator,
	OverflowMenu,
	ServerOnlinePlayers,
	ServerPing,
	ServerRegion,
	TagItem,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import duration from 'dayjs/plugin/duration'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'
import UpdateToPlayModal from '@/components/ui/modal/UpdateToPlayModal.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_v3, get_version, get_version_many } from '@/helpers/cache.js'
import { process_listener, profile_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process'
import { finish_install, get, get_full_path, kill, run } from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'
import { playServerProject } from '@/store/install.js'
import { useBreadcrumbs, useLoading } from '@/store/state'
import type { Labrinth } from '@modrinth/api-client'

dayjs.extend(duration)
dayjs.extend(relativeTime)

const { handleError } = injectNotificationManager()
const route = useRoute()

const router = useRouter()
const breadcrumbs = useBreadcrumbs()

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const instance = ref<GameInstance>()
const modrinthVersions = ref<any[]>([])
const playing = ref(false)
const loading = ref(false)
const exportModal = ref<InstanceType<typeof ExportModal>>()
const updateToPlayModal = ref<InstanceType<typeof UpdateToPlayModal>>()

const isServerInstance = ref(false)
const projectV3 = ref<Labrinth.Projects.v3.Project>()
const modpackContentProjectV3 = ref<Labrinth.Projects.v3.Project | null>(null)
const selected = ref<unknown[]>([])

const minecraftServer = computed(() => projectV3.value?.minecraft_server)
const javaServerPingData = computed(() => projectV3.value?.minecraft_java_server_ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)
const ping = computed(() => Math.trunc(Number(javaServerPingData.value?.latency.nanos) / 1000000))

async function fetchInstance() {
	instance.value = await get(route.params.id as string).catch(handleError)

	if (!offline.value && instance.value?.linked_data && instance.value.linked_data.project_id) {
		try {
			projectV3.value = await get_project_v3(
				instance.value.linked_data.project_id,
				'must_revalidate',
			)

			if (projectV3.value && projectV3.value.versions) {
				const versions = await get_version_many(projectV3.value.versions, 'must_revalidate')
				modrinthVersions.value = versions.sort(
					(a: any, b: any) => dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf(),
				)
				if (projectV3.value?.minecraft_server !== undefined) {
					isServerInstance.value = true
					await fetchModpackContent()
				}
			}
		} catch (error: any) {
			handleError(error)
		}
	}

	await updatePlayState()
}

async function fetchModpackContent() {
	modpackContentProjectV3.value = null
	const versionId = instance.value?.linked_data?.version_id
	if (!versionId) return

	const contentVersion = await get_version(versionId, 'bypass')
	const projectId = contentVersion?.project_id
	if (projectId) {
		modpackContentProjectV3.value = await get_project_v3(projectId, 'bypass')
	}
}

async function updatePlayState() {
	const runningProcesses = await get_by_profile_path(route.params.id as string).catch(handleError)

	playing.value = Array.isArray(runningProcesses) && runningProcesses.length > 0
}

await fetchInstance()
watch(
	() => route.params.id,
	async () => {
		if (route.params.id && route.path.startsWith('/instance')) {
			await fetchInstance()
		}
	},
)

const basePath = computed(() => `/instance/${encodeURIComponent(route.params.id as string)}`)

const tabs = computed(() => [
	{
		label: 'Content',
		href: `${basePath.value}`,
	},
	{
		label: 'Worlds',
		href: `${basePath.value}/worlds`,
	},
	{
		label: 'Logs',
		href: `${basePath.value}/logs`,
	},
])

if (instance.value) {
	breadcrumbs.setName(
		'Instance',
		instance.value.name.length > 40
			? instance.value.name.substring(0, 40) + '...'
			: instance.value.name,
	)
	breadcrumbs.setContext({
		name: instance.value.name,
		link: route.path,
		query: route.query,
	})
}

const loadingBar = useLoading()

const options = ref<InstanceType<typeof ContextMenu> | null>(null)

const startInstance = async (context: string) => {
	if (!instance.value) return
	if (updateToPlayModal.value?.hasUpdate) {
		updateToPlayModal.value.show(instance.value)
		return
	}

	loading.value = true
	try {
		await run(route.params.id as string)
		playing.value = true
	} catch (err) {
		handleSevereError(err, { profilePath: route.params.id as string })
	}
	loading.value = false

	trackEvent('InstancePlay', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

const stopInstance = async (context: string) => {
	playing.value = false
	await kill(route.params.id as string).catch(handleError)

	if (!instance.value) return
	trackEvent('InstanceStop', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

const repairInstance = async () => {
	await finish_install(instance.value).catch(handleError)
}

const handleRightClick = (event: MouseEvent) => {
	const baseOptions = [
		{ name: 'add_content' },
		{ type: 'divider' },
		{ name: 'edit' },
		{ name: 'open_folder' },
		{ name: 'copy_path' },
	]

	options.value?.showMenu(
		event,
		instance.value,
		playing.value
			? [
					{
						name: 'stop',
						color: 'danger',
					},
					...baseOptions,
				]
			: [
					{
						name: 'play',
						color: 'primary',
					},
					...baseOptions,
				],
	)
}

const handleOptionsClick = async (args: { option: string; item: unknown }) => {
	switch (args.option) {
		case 'play':
			await startInstance('InstancePageContextMenu')
			break
		case 'stop':
			await stopInstance('InstancePageContextMenu')
			break
		case 'add_content':
			await router.push({
				path: `/browse/${instance.value?.loader === 'vanilla' ? 'datapack' : 'mod'}`,
				query: { i: route.params.id },
			})
			break
		case 'edit':
			await router.push({
				path: `/instance/${encodeURIComponent(route.params.id as string)}/options`,
			})
			break
		case 'open_folder':
			if (instance.value) await showProfileInFolder(instance.value.path)
			break
		case 'copy_path': {
			if (instance.value) {
				const fullPath = await get_full_path(instance.value?.path)
				await navigator.clipboard.writeText(fullPath)
			}
			break
		}
	}
}

const unlistenProfiles = await profile_listener(async (event: any) => {
	if (event.profile_path_id === route.params.id) {
		if (event.event === 'removed') {
			await router.push({
				path: '/',
			})
			return
		}
		instance.value = await get(route.params.id as string).catch(handleError)
	}
})

const unlistenProcesses = await process_listener((e: any) => {
	if (e.event === 'finished' && e.profile_path_id === route.params.id) {
		playing.value = false
	}
})

const icon = computed(() =>
	instance.value?.icon_path ? convertFileSrc(instance.value.icon_path) : null,
)

const settingsModal = ref<InstanceType<typeof InstanceSettingsModal>>()

const timePlayed = computed(() => {
	return instance.value
		? instance.value.recent_time_played + instance.value.submitted_time_played
		: 0
})

const timePlayedHumanized = computed(() => {
	const duration = dayjs.duration(timePlayed.value, 'seconds')
	const hours = Math.floor(duration.asHours())
	if (hours >= 1) {
		return hours + ' hour' + (hours > 1 ? 's' : '')
	}

	const minutes = Math.floor(duration.asMinutes())
	if (minutes >= 1) {
		return minutes + ' minute' + (minutes > 1 ? 's' : '')
	}

	const seconds = Math.floor(duration.asSeconds())
	return seconds + ' second' + (seconds > 1 ? 's' : '')
})

onUnmounted(() => {
	unlistenProcesses()
	unlistenProfiles()
})
</script>

<style scoped lang="scss">
.instance-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

Button {
	width: 100%;
}

.button-group {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
}

.side-cards {
	position: fixed;
	width: 300px;
	display: flex;
	flex-direction: column;

	min-height: calc(100vh - 3.25rem);
	max-height: calc(100vh - 3.25rem);
	overflow-y: auto;
	-ms-overflow-style: none;
	scrollbar-width: none;

	&::-webkit-scrollbar {
		width: 0;
		background: transparent;
	}

	.card {
		min-height: unset;
		margin-bottom: 0;
	}
}

.instance-nav {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	justify-content: center;
	padding: 1rem;
	gap: 0.5rem;
	background: var(--color-raised-bg);
	height: 100%;
}

.name {
	font-size: 1.25rem;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
}

.metadata {
	text-transform: capitalize;
}

.instance-container {
	display: flex;
	flex-direction: row;
	overflow: auto;
	gap: 1rem;
	min-height: 100%;
	padding: 1rem;
}

.instance-info {
	display: flex;
	flex-direction: column;
	width: 100%;
}

.badge {
	display: flex;
	align-items: center;
	font-weight: bold;
	width: fit-content;
	color: var(--color-orange);
}

.pages-list {
	display: flex;
	flex-direction: column;
	gap: var(--gap-xs);

	.btn {
		font-size: 100%;
		font-weight: 400;
		background: inherit;
		transition: all ease-in-out 0.1s;
		width: 100%;
		color: var(--color-primary);
		box-shadow: none;

		&.router-link-exact-active {
			box-shadow: var(--shadow-inset-lg);
			background: var(--color-button-bg);
			color: var(--color-contrast);
		}

		&:hover {
			background-color: var(--color-button-bg);
			color: var(--color-contrast);
			box-shadow: var(--shadow-inset-lg);
			text-decoration: none;
		}

		svg {
			width: 1.3rem;
			height: 1.3rem;
		}
	}
}

.instance-nav {
	display: flex;
	flex-direction: row;
	align-items: flex-start;
	justify-content: left;
	padding: 1rem;
	gap: 0.5rem;
	height: min-content;
	width: 100%;
}

.instance-button {
	width: fit-content;
}

.actions {
	display: flex;
	flex-direction: column;
	justify-content: flex-start;
	gap: 0.5rem;
}

.content {
	margin: 0 1rem 0.5rem 20rem;
	width: calc(100% - 20rem);
	display: flex;
	flex-direction: column;
	overflow: auto;
}

.stats {
	grid-area: stats;
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	gap: var(--gap-md);

	.stat {
		display: flex;
		flex-direction: row;
		align-items: center;
		width: fit-content;
		gap: var(--gap-xs);
		--stat-strong-size: 1.25rem;

		strong {
			font-size: var(--stat-strong-size);
		}

		p {
			margin: 0;
		}

		svg {
			height: var(--stat-strong-size);
			width: var(--stat-strong-size);
		}
	}

	.date {
		margin-top: auto;
	}

	@media screen and (max-width: 750px) {
		flex-direction: row;
		column-gap: var(--gap-md);
		margin-top: var(--gap-xs);
	}

	@media screen and (max-width: 600px) {
		margin-top: 0;

		.stat-label {
			display: none;
		}
	}
}
</style>
