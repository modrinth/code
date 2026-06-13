import type { Archon, UploadState } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import ServerPanelAdmonitions from '../../components/servers/admonitions/ServerPanelAdmonitions.vue'
import { defineMessage } from '../../composables/i18n'
import type { FileOperation } from '../../layouts/shared/files-tab/types'
import { provideModrinthServerContext } from '../../providers'
import type {
	CancelUploadHandler,
	ModrinthServerContext,
	ServerStats,
} from '../../providers/server-context'

const INSTANCE_INFO_ADMONITION_KEY = 'server-instances-info-admonition-dismissed'

const meta = {
	title: 'Servers/ServerPanelAdmonitions',
	component: ServerPanelAdmonitions,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story, context) => ({
			components: { story },
			setup() {
				if (
					typeof window !== 'undefined' &&
					context.parameters.resetInstanceInfoAdmonition === true
				) {
					window.localStorage.removeItem(INSTANCE_INFO_ADMONITION_KEY)
				}

				const router = useRouter()
				onMounted(() => {
					router.replace(
						(context.parameters.routePath as string | undefined) ??
							'/hosting/manage/demo-server/instances/demo-world',
					)
				})
				const serverWorlds =
					(context.parameters.serverWorlds as Archon.Servers.v1.WorldFull[] | undefined) ?? null

				const server = ref({
					server_id: 'demo-server',
					status: 'running',
					upstream: null,
				} as Archon.Servers.v0.Server)

				const stats = ref<ServerStats>({
					current: {
						cpu_percent: 0,
						ram_usage_bytes: 0,
						ram_total_bytes: 1,
						storage_usage_bytes: 0,
						storage_total_bytes: 0,
					},
					past: {
						cpu_percent: 0,
						ram_usage_bytes: 0,
						ram_total_bytes: 1,
						storage_usage_bytes: 0,
						storage_total_bytes: 0,
					},
					graph: { cpu: [], ram: [] },
				})

				const uploadState = ref<UploadState>({
					isUploading: true,
					currentFileName: 'resourcepack.zip',
					currentFileProgress: 0.2,
					uploadedBytes: 20_000,
					totalBytes: 100_000,
					completedFiles: 1,
					totalFiles: 3,
				})

				const fileOp = ref<FileOperation[]>([
					{
						id: 'fs-op-1',
						op: 'extract',
						src: 'story-modpack.mrpack',
						state: 'running',
						progress: 0.35,
						bytes_processed: 2_000_000,
					},
				])

				const serverContext: ModrinthServerContext = {
					get serverId() {
						return 'demo-server'
					},
					worldId: ref(null),
					server,
					serverFull: computed(() =>
						serverWorlds ? ({ worlds: serverWorlds } as Archon.Servers.v1.ServerFull) : null,
					),
					currentUserPermissions: computed(() => 0),
					isConnected: ref(true),
					isWsAuthIncorrect: ref(false),
					powerState: ref('running'),
					powerStateDetails: ref(undefined),
					isServerRunning: computed(() => true),
					stats,
					uptimeSeconds: ref(0),
					isSyncingContent: ref(false),
					busyReasons: computed(() => [
						{ reason: defineMessage({ id: 's.bg', defaultMessage: 'Background task running' }) },
					]),
					fsAuth: ref(null),
					fsOps: ref<Archon.Websocket.v0.FilesystemOperation[]>([]),
					fsQueuedOps: ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([]),
					refreshFsAuth: async () => {},
					uploadState,
					cancelUpload: ref<CancelUploadHandler | null>(async () => {
						await new Promise((resolve) => setTimeout(resolve, 1200))
						uploadState.value = { ...uploadState.value, isUploading: false }
					}),
					activeOperations: computed(() => fileOp.value),
					dismissOperation: async (id) => {
						fileOp.value = fileOp.value.filter((o) => o.id !== id)
					},
				}

				provideModrinthServerContext(serverContext)
				return {}
			},
			template: '<div style="max-width: 720px"><story /></div>',
		}),
	],
} satisfies Meta<typeof ServerPanelAdmonitions>

export default meta

type Story = StoryObj<typeof meta>

export const WithUploadFileOpAndBusy: Story = {}

export const InstanceInfo: Story = {
	parameters: {
		routePath: '/hosting/manage/demo-server/instances',
		resetInstanceInfoAdmonition: true,
		serverWorlds: [{ id: 'demo-world' }],
	},
}
