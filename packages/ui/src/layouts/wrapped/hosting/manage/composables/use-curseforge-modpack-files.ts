import type { AbstractModrinthClient, Archon, Kyros, Labrinth } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

const PAGE_SIZE = 50

type CurseForgeLink = Archon.Content.v1.ModpackSpecCurseForge['link']

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
	worldId: Ref<string | null>
	link: Ref<NonNullable<CurseForgeLink> | null>
	enabled: Ref<boolean>
}) {
	const { client, worldId, link } = options
	const filesQuery = useQuery({
		queryKey: computed(() => [
			'kyros',
			'flameanvil',
			'files',
			worldId.value,
			link.value?.project_id,
		]),
		queryFn: async () => {
			const files: Kyros.FlameAnvil.v1.ProjectFile[] = []
			for (let index = 0; ; index += PAGE_SIZE) {
				const page = await client.kyros.flameanvil_v1.listProjectFiles(
					worldId.value!,
					link.value!.project_id,
					{ index, page_size: PAGE_SIZE },
				)
				files.push(...page.files)
				if (page.files.length === 0 || index + PAGE_SIZE >= page.total_count) break
			}
			return files.filter((file) => file.available)
		},
		enabled: computed(() => worldId.value !== null && !!link.value && options.enabled.value),
	})

	const filesById = computed(
		() => new Map((filesQuery.data.value ?? []).map((file) => [String(file.id), file])),
	)

	const versions = computed(() => {
		const current = link.value
		if (!current) return []
		return (filesQuery.data.value ?? []).map((file) => fileToVersion(file, current.project_id))
	})

	return { filesQuery, filesById, versions }
}
