import type { SharedInstances } from '@modrinth/api-client'
import { CopyIcon, DownloadIcon, SearchIcon } from '@modrinth/assets'
import {
	type ComboboxOption,
	type ContentCardTableItem,
	type ContentItem,
	useFormatBytes,
} from '@modrinth/ui'
import { computed, type Ref, ref } from 'vue'

import { useCopyText } from '~/composables/shared-instance-moderation'
import { SHARED_INSTANCE_CONFIG_FILE_TYPES } from '~/helpers/shared-instance-moderation'

type SortMode = 'uploaded' | 'name'
type RowKind = 'uploaded' | 'modrinth'

export type InspectorRow = {
	kind: RowKind
	typeFilter: string
	searchText: string
	sortName: string
	tableItem: ContentCardTableItem
	fileName: string
	fileType: string
}

const TYPE_LABELS: Record<string, string> = {
	mod: 'Mods',
	resourcepack: 'Resource Packs',
	shader: 'Shaders',
	datapack: 'Data Packs',
	plugin: 'Plugins',
	modpack: 'Modpacks',
}

const KIND_ORDER: Record<RowKind, number> = { uploaded: 0, modrinth: 1 }

export const sortOptions: ComboboxOption<SortMode>[] = [
	{ value: 'uploaded', label: 'Uploaded first' },
	{ value: 'name', label: 'Name' },
]

function normalizeType(type: string) {
	return type === 'shaderpack' ? 'shader' : type
}

export function useInspectorContent(
	version: Ref<SharedInstances.Instances.v1.InstanceVersion | undefined>,
	content: Ref<ContentItem[] | undefined>,
) {
	const copyText = useCopyText()
	const formatBytes = useFormatBytes()

	const searchQuery = ref('')
	const sortMode = ref<SortMode>('uploaded')
	const selectedFilters = ref<string[]>([])

	const externalFilesByKey = computed(() => {
		const files = new Map<string, SharedInstances.Instances.v1.ExternalFile>()
		for (const file of version.value?.external_files ?? []) {
			files.set(`${file.file_type}:${file.file_name}`, file)
		}
		return files
	})

	const overrideFiles = computed(() =>
		(version.value?.external_files ?? []).filter((file) =>
			SHARED_INSTANCE_CONFIG_FILE_TYPES.has(file.file_type),
		),
	)

	function externalRow(item: ContentItem): InspectorRow {
		const fileType = item.project_type
		const file = externalFilesByKey.value.get(`${fileType}:${item.file_name}`)
		const sha512 = file?.sha512
		const downloadUrl = file?.url ?? item.external_url ?? null

		return {
			kind: 'uploaded',
			typeFilter: normalizeType(fileType),
			searchText: item.file_name.toLowerCase(),
			sortName: item.file_name.toLowerCase(),
			fileName: item.file_name,
			fileType,
			tableItem: {
				id: item.id,
				projectType: fileType,
				project: { id: item.id, slug: item.file_name, title: item.file_name, icon_url: undefined },
				version: {
					id: item.id,
					version_number: item.size != null ? formatBytes(item.size) : 'Unknown size',
					file_name: sha512 ? `sha512 ${sha512.slice(0, 4)}…${sha512.slice(-4)}` : '',
				},
				external: true,
				externalFile: true,
				overflowOptions: [
					...(downloadUrl
						? [
								{
									id: 'download',
									type: 'link' as const,
									label: 'Download',
									icon: DownloadIcon,
									href: downloadUrl,
									target: '_blank',
									rel: 'noopener noreferrer',
								},
							]
						: []),
					...(sha512
						? [
								{
									id: 'copy-sha512',
									label: 'Copy SHA-512',
									icon: CopyIcon,
									action: () => copyText(sha512, 'SHA-512 hash copied'),
								},
							]
						: []),
					...(downloadUrl && item.file_name.endsWith('.jar')
						? [
								{
									id: 'slicer',
									type: 'link' as const,
									label: 'View in Slicer',
									icon: SearchIcon,
									href: `https://slicer.run/?url=${encodeURIComponent(downloadUrl)}`,
									target: '_blank',
									rel: 'noopener noreferrer',
								},
							]
						: []),
				],
			},
		}
	}

	function modrinthRow(item: ContentItem): InspectorRow {
		const title = item.project?.title ?? item.file_name
		const versionId = item.version?.id

		return {
			kind: 'modrinth',
			typeFilter: normalizeType(item.project_type),
			searchText: `${title} ${item.file_name}`.toLowerCase(),
			sortName: title.toLowerCase(),
			fileName: item.file_name,
			fileType: item.project_type,
			tableItem: {
				id: item.id,
				projectType: item.project_type,
				project: item.project ?? {
					id: item.id,
					slug: item.file_name,
					title: item.file_name,
					icon_url: undefined,
				},
				projectLink: item.project?.id ? `/project/${item.project.id}` : undefined,
				version: item.version,
				owner: item.owner
					? {
							...item.owner,
							link: `/${item.owner.type}/${encodeURIComponent(item.owner.id)}`,
						}
					: undefined,
				versionLink:
					item.project?.id && versionId
						? `/project/${item.project.id}/version/${versionId}`
						: undefined,
				source: item.source
					? {
							...item.source,
							label: 'Provided by',
							link: `/project/${item.source.project.slug ?? item.source.project.id}`,
						}
					: undefined,
				hideDelete: true,
				overflowOptions: versionId
					? [
							{
								id: 'copy-version-id',
								label: 'Copy version ID',
								icon: CopyIcon,
								action: () => copyText(versionId, 'Version ID copied'),
							},
						]
					: [],
			},
		}
	}

	const rows = computed<InspectorRow[]>(() =>
		(content.value ?? [])
			.filter(
				(item) => !(item.external && SHARED_INSTANCE_CONFIG_FILE_TYPES.has(item.project_type)),
			)
			.map((item) => (item.external ? externalRow(item) : modrinthRow(item))),
	)

	const filterOptions = computed(() => {
		const types = new Set(rows.value.map((row) => row.typeFilter))
		return [
			...(rows.value.some((row) => row.kind === 'uploaded')
				? [{ id: 'uploaded', label: 'Uploaded' }]
				: []),
			...Object.entries(TYPE_LABELS)
				.filter(([type]) => types.has(type))
				.map(([type, label]) => ({ id: type, label })),
		]
	})

	const visibleRows = computed(() => {
		const query = searchQuery.value.trim().toLowerCase()
		const filters = selectedFilters.value
		return rows.value
			.filter((row) => !query || row.searchText.includes(query))
			.filter(
				(row) =>
					filters.length === 0 ||
					filters.includes(row.typeFilter) ||
					(filters.includes('uploaded') && row.kind === 'uploaded'),
			)
			.sort(
				(first, second) =>
					(sortMode.value === 'uploaded' ? KIND_ORDER[first.kind] - KIND_ORDER[second.kind] : 0) ||
					first.sortName.localeCompare(second.sortName),
			)
	})

	const tableItems = computed(() => visibleRows.value.map((row) => row.tableItem))

	function resetContentFilters() {
		searchQuery.value = ''
		selectedFilters.value = []
	}

	return {
		searchQuery,
		sortMode,
		selectedFilters,
		rows,
		visibleRows,
		tableItems,
		filterOptions,
		overrideFiles,
		resetContentFilters,
	}
}
