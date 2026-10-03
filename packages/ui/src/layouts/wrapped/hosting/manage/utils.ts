import type {LocationQuery, RouteLocationNormalizedLoadedGeneric} from "vue-router";

import type {FileInfo} from "#ui/layouts/shared/files-tab/providers/file-manager.ts";
import {infoFrom} from "#ui/layouts/shared/files-tab/utils.ts";

export type FileTypeToQueryStart = {
	'file': 'file-content',
	'directory': 'files'
};

export type QueryableFileTypes = keyof FileTypeToQueryStart;

type FileQueryKey<T extends QueryableFileTypes> =
	readonly [FileTypeToQueryStart[T], serverId: string, path: string]
	| readonly [FileTypeToQueryStart[T], serverId: string, worldId: string, path: string];

type FileQueryFilter<T extends QueryableFileTypes> = FileQueryKey<T> |
	(readonly [FileTypeToQueryStart[T], serverId: string]
		| readonly [FileTypeToQueryStart[T], serverId: string, worldId: string]);

export function queryFilterFor<T extends QueryableFileTypes>(serverId: string, worldId: string | undefined | null, type: T, path?: string): FileQueryFilter<T> {
	const rootQueryPart = (type == "file" ? 'file-content' : 'files') as FileTypeToQueryStart[T];
	return [rootQueryPart, serverId, ...(worldId ? [worldId] as const: [] as const), ...(path ? [path] as const: [] as const)] as FileQueryKey<T>
}

export function queryKeyFor<T extends QueryableFileTypes>(serverId: string, worldId: string | undefined | null, info: FileInfo<T>): FileQueryKey<T> {
	const rootQueryPart = (info.type == "file" ? 'file-content' : 'files') as FileTypeToQueryStart[T];
	return [rootQueryPart, serverId, ...(worldId ? [worldId] as const: [] as const), info.path] as FileQueryKey<T>
}

export function infoFromQuery(value: LocationQuery | RouteLocationNormalizedLoadedGeneric): FileInfo {
	const query = (value as Partial<RouteLocationNormalizedLoadedGeneric>).name != null ? value.query as LocationQuery : value as LocationQuery;
	return infoFrom(query.path === 'string' ? query.path : '/', query.editing === 'true')
}
