import type { ConfirmLeaveModal } from '@modrinth/ui'
import { createContext } from '@modrinth/ui'
import { isAdmin, isStaff } from '@modrinth/utils'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'

import { useDisclosureEditor } from '~/components/ui/project-settings/disclosures/use-disclosure-editor'
import { useAuthState } from '~/composables/auth'
import { useModerationQueue } from '~/services/moderation/queue'

import { useReviewContent } from './content'
import { useReviewProject } from './project'
import { useReviewQueue } from './queue'
import type { ProjectReviewPageContext } from './types'

export const [injectProjectReviewPageContext, provideProjectReviewPageContext] =
	createContext<ProjectReviewPageContext>('ProjectReviewPage')

export function createProjectReviewPageContext() {
	const route = useRoute()
	const router = useRouter()
	const queue = useModerationQueue()
	const selection = computed(() =>
		typeof route.query.project === 'string' ? route.query.project : '',
	)
	const data = useReviewProject(selection)
	const content = useReviewContent(data.projectId)
	const auth = useAuthState()
	const disclosures = useDisclosureEditor({
		projectId: data.projectId,
		projectTypes: computed(() => data.project.value?.project_types ?? []),
		canEditDisclosures: computed(
			() =>
				!!data.project.value &&
				(data.project.value.versions.length > 0 || data.project.value.minecraft_server != null),
		),
		hasPermission: computed(() => isStaff(auth.value.user)),
		isActingAsModerator: computed(() => isStaff(auth.value.user)),
		isAdminUser: computed(() => isAdmin(auth.value.user)),
	})
	const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal>>()
	async function confirmDiscardDisclosures() {
		if (disclosures.saving.value) return false
		if (!disclosures.hasChanges.value) return true
		if (!(await confirmLeaveModal.value?.prompt())) return false
		disclosures.reset()
		return true
	}
	const navigation = useReviewQueue(data.projectId, queue, confirmDiscardDisclosures)
	onBeforeRouteLeave(confirmDiscardDisclosures)
	onBeforeRouteUpdate((to, from) => {
		if (to.query.project !== from.query.project) return confirmDiscardDisclosures()
	})
	function beforeUnload(event: BeforeUnloadEvent) {
		if (disclosures.hasChanges.value || disclosures.saving.value) event.preventDefault()
	}
	onMounted(() => window.addEventListener('beforeunload', beforeUnload))
	onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))

	onMounted(async () => {
		await queue.ready
		if (selection.value) return
		const id = queue.currentQueue.activeProjectId ?? queue.getCurrentProjectId()
		if (id) await router.replace({ query: { ...route.query, project: id } })
	})
	watch(data.projectId, async (id) => {
		if (!id || navigation.busy.value) return
		await queue.visitProject(id)
	})

	return {
		disclosures,
		confirmLeaveModal,
		...data,
		...content,
		tabCounts: computed(() => ({
			...content.contentCounts.value,
			gallery: data.project.value ? data.gallery.value.length : undefined,
			disclosures: disclosures.disclosuresQuery.isSuccess.value
				? disclosures.currentSnapshot.value.disclosures.length
				: undefined,
			permissions: data.permissions.value.loaded
				? data.permissions.value.awaitingReviewCount
				: undefined,
		})),
		selection,
		queue,
		navigation,
	}
}
