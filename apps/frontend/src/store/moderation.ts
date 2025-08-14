import { createPinia, defineStore } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

export interface ModerationQueue {
	items: string[]
	total: number
	completed: number
	skipped: number
	lastUpdated: Date
}

const EMPTY_QUEUE: Partial<ModerationQueue> = {
	items: [],

	// TODO: Consider some form of displaying this in the checklist, maybe at the end
	total: 0,
	completed: 0,
	skipped: 0,
}

function createEmptyQueue(): ModerationQueue {
	return { ...EMPTY_QUEUE, lastUpdated: new Date() } as ModerationQueue
}

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export const useModerationStore = defineStore('moderation', {
	state: () => ({
		currentQueue: createEmptyQueue(),
	}),

	getters: {
		queueLength: (state) => state.currentQueue.items.length,
		hasItems: (state) => state.currentQueue.items.length > 0,
		progress: (state) => {
			if (state.currentQueue.total === 0) return 0
			return (state.currentQueue.completed + state.currentQueue.skipped) / state.currentQueue.total
		},
	},

	actions: {
		setQueue(projectIDs: string[]) {
			this.currentQueue = {
				items: [...projectIDs],
				total: projectIDs.length,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		},

		setSingleProject(projectId: string) {
			this.currentQueue = {
				items: [projectId],
				total: 1,
				completed: 0,
				skipped: 0,
				lastUpdated: new Date(),
			}
		},

		completeCurrentProject(projectId: string, status: 'completed' | 'skipped' = 'completed') {
			if (status === 'completed') {
				this.currentQueue.completed++
			} else {
				this.currentQueue.skipped++
			}

			this.currentQueue.items = this.currentQueue.items.filter((id: string) => id !== projectId)
			this.currentQueue.lastUpdated = new Date()

			return this.currentQueue.items.length > 0
		},

		getCurrentProjectId(): string | null {
			return this.currentQueue.items[0] || null
		},

		resetQueue() {
			this.currentQueue = createEmptyQueue()
		},
	},

	persist: {
		key: 'moderation-store',
		serializer: {
			serialize: JSON.stringify,
			deserialize: (value: string) => {
				const parsed = JSON.parse(value)
				if (parsed.currentQueue?.lastUpdated) {
					parsed.currentQueue.lastUpdated = new Date(parsed.currentQueue.lastUpdated)
				}
				return parsed
			},
		},
	},
})
