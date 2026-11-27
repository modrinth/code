import type { AbstractModrinthClient, Archon, Kyros, Labrinth } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

const PAGE_SIZE = 50

type CurseForgeLink = NonNullable<Archon.Content.v1.ModpackSpecCurseForge['link']>

function fileToVersion(
	file: Kyros.FlameAnvil.v1.ProjectFile,
	projectId: number,
): Labrinth.Versions.v2.Version {
	return {
		id: String(file.id),
		project_id: String(projectId),
		author_id: '',
		featured: false,
		name: file.display_name,
		version_number: file.display_name,
		changelog: '',
		date_published: file.file_date,
		downloads: 0,
		version_type: file.release_channel ?? 'release',
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: file.minecraft_versions,
		loaders: file.loaders,
	}
}

/**
 * Lists the available files of a CurseForge modpack as Modrinth-shaped versions, so the
 * version switcher can show them alongside Modrinth modpacks.
 */
export function useCurseForgeModpackFiles(options: {
	client: AbstractModrinthClient
	worldId: Readonly<Ref<string | null>>
	link: Readonly<Ref<CurseForgeLink | null>>
}) {
	const { client, worldId, link } = options
	const queryClient = useQueryClient()

	function filesQueryKey(targetWorldId: string | null, projectId: number | undefined) {
		return ['kyros', 'flameanvil', 'files', targetWorldId, projectId] as const
	}

	async function listFiles(targetWorldId: string, projectId: number) {
		const files: Kyros.FlameAnvil.v1.ProjectFile[] = []
		for (let index = 0; ; index += PAGE_SIZE) {
			const page = await client.kyros.flameanvil_v1.listProjectFiles(targetWorldId, projectId, {
				index,
				page_size: PAGE_SIZE,
			})
			files.push(...page.files)
			if (page.files.length === 0 || index + PAGE_SIZE >= page.total_count) break
		}
		return files.filter((file) => file.available)
	}

	const filesQuery = useQuery({
		queryKey: computed(() => filesQueryKey(worldId.value, link.value?.project_id)),
		queryFn: () => listFiles(worldId.value!, link.value!.project_id),
		enabled: false,
	})

	const versions = computed(() => {
		const current = link.value
		if (!current) return []
		return (filesQuery.data.value ?? []).map((file) => fileToVersion(file, current.project_id))
	})

	async function fetchFiles() {
		const targetWorldId = worldId.value
		const targetLink = link.value
		if (!targetWorldId || !targetLink) return []
		return await queryClient.fetchQuery({
			queryKey: filesQueryKey(targetWorldId, targetLink.project_id),
			queryFn: () => listFiles(targetWorldId, targetLink.project_id),
		})
	}

	async function fetchVersions() {
		const targetLink = link.value
		if (!targetLink) return []
		return (await fetchFiles()).map((file) => fileToVersion(file, targetLink.project_id))
	}

	async function findFile(versionId: string) {
		return (await fetchFiles()).find((file) => String(file.id) === versionId)
	}

	return { versions, fetchVersions, findFile }
}
