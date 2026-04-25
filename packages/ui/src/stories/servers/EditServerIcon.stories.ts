import type { Archon, UploadState } from '@modrinth/api-client'
import type { Stats } from '@modrinth/utils'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, reactive, ref } from 'vue'

import EditServerIcon from '../../components/servers/edit-server-icon/EditServerIcon.vue'
import { provideModrinthServerContext } from '../../providers'
import type { ModrinthServerContext } from '../../providers/server-context'

const meta = {
	title: 'Servers/EditServerIcon',
	component: EditServerIcon,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			setup() {
				const server = ref({
					server_id: 'story-server-id',
					upstream: null,
				} as Archon.Servers.v0.Server)

				const stats = ref<Stats>({
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
					graph: {
						cpu: [],
						ram: [],
					},
				})

				const uploadState = ref<UploadState>({
					isUploading: false,
					currentFileName: null,
					currentFileProgress: 0,
					uploadedBytes: 0,
					totalBytes: 0,
					completedFiles: 0,
					totalFiles: 0,
				})

				const serverContext: ModrinthServerContext = {
					get serverId() {
						return 'story-server-id'
					},
					worldId: ref(null),
					server,
					isConnected: ref(true),
					isWsAuthIncorrect: ref(false),
					powerState: ref('running'),
					powerStateDetails: ref(undefined),
					isServerRunning: computed(() => true),
					stats,
					uptimeSeconds: ref(0),
					backupsState: reactive(new Map()),
					markBackupCancelled: () => {},
					isSyncingContent: ref(false),
					busyReasons: computed(() => []),
					fsAuth: ref(null),
					fsOps: ref<Archon.Websocket.v0.FilesystemOperation[]>([]),
					fsQueuedOps: ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([]),
					refreshFsAuth: async () => {},
					uploadState,
					cancelUpload: ref(null),
					activeOperations: computed(() => []),
					dismissOperation: async () => {},
				}

				provideModrinthServerContext(serverContext)
			},
			template: '<div style="max-width: 320px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof EditServerIcon>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}
