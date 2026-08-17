declare type u8 = number
declare type u16 = number
declare type u32 = number
declare type u64 = bigint
declare type u128 = bigint
declare type usize = bigint
declare type i8 = number
declare type i16 = number
declare type i32 = number
declare type i64 = bigint
declare type i128 = bigint
declare type isize = bigint
declare type NonZeroU8 = number
declare type NonZeroU16 = number
declare type NonZeroU32 = number
declare type NonZeroU64 = bigint
declare type NonZeroU128 = bigint
declare type NonZeroUsize = bigint
declare type NonZeroI8 = number
declare type NonZeroI16 = number
declare type NonZeroI32 = number
declare type NonZeroI64 = bigint
declare type NonZeroI128 = bigint
declare type NonZeroIsize = bigint
declare type f32 = number
declare type f64 = number

declare type ArrayLengthMutationKeys = "splice" | "push" | "pop" | "shift" | "unshift"
declare type FixedLengthArray<T, L extends number, TObj = [T, ...Array<T>]> =
    Pick<TObj, Exclude<keyof TObj, ArrayLengthMutationKeys>>
    & {
        readonly length: L
        [ I : number ] : T
        [Symbol.iterator]: () => IterableIterator<T>
    }

export type AppEvent = { tag: "loading", value: LoadingPayload } | { tag: "process", value: ProcessPayload } | { tag: "instance", value: InstancePayload } | { tag: "instance_groups_changed", value: InstanceGroupsChangedPayload } | { tag: "onboarding_checklist", value: OnboardingChecklist } | { tag: "instance_bulk_update_progress", value: InstanceBulkUpdateProgressPayload } | { tag: "install_job", value: InstallJobSnapshot } | { tag: "command", value: CommandPayload } | { tag: "warning", value: WarningPayload } | { tag: "friend", value: FriendPayload } | { tag: "notification", value: string } | { tag: "log", value: LogPayload } | { tag: "ads_consent_required", value: boolean }
export type LoadingBarType = { tag: "legacy_data_migration" } | { tag: "directory_move", value: { old: string, new: string } } | { tag: "java_download", value: { version: u32 } } | { tag: "pack_file_download", value: { instance_id: string, pack_name: string, icon: string | undefined, pack_version: string } } | { tag: "pack_download", value: { instance_id: string, pack_name: string, icon: string | undefined, pack_id: string | undefined, pack_version: string | undefined } } | { tag: "pack_import", value: { pack_name: string } } | { tag: "minecraft_download", value: { instance_id: string, instance_name: string } } | { tag: "instance_update", value: { instance_id: string, instance_name: string } } | { tag: "zip_extract", value: { instance_id: string, instance_name: string } } | { tag: "pack_export", value: { instance_id: string, instance_name: string } } | { tag: "config_change", value: { new_path: string } } | { tag: "copy_instance", value: { import_location: string, instance_name: string } } | { tag: "launcher_update", value: { version: string, current_version: string } }
export type LoadingPayload = { event: LoadingBarType, loader_uuid: string, fraction: f64 | undefined, message: string }
export type WarningPayload = { message: string }
export type InstanceBulkUpdateProgressPayload = { instanceId: string, stage: InstanceBulkUpdateProgressStage, current: u64, total: u64 }
export type InstanceBulkUpdateProgressStage = { tag: "resolving_versions" } | { tag: "downloading" } | { tag: "finishing" }
export type CommandPayload = { tag: "InstallMod", value: { id: string } } | { tag: "InstallVersion", value: { id: string } } | { tag: "InstallModpack", value: { id: string } } | { tag: "InstallServer", value: { id: string } } | { tag: "LaunchInstance", value: { id: string, server: string | undefined, singleplayer_world: string | undefined } } | { tag: "InstallSharedInstanceInvite", value: { invite_id: string } } | { tag: "RunMRPack", value: { path: string } }
export type ProcessPayload = { instance_id: string, uuid: string, event: ProcessPayloadType, message: string }
export type ProcessPayloadType = { tag: "launched" } | { tag: "finished" }
export type InstancePayload = { instance_id: string, event: InstancePayloadType }
export type InstancePayloadType = { tag: "created" } | { tag: "synced" } | { tag: "servers_updated" } | { tag: "world_updated", value: { world: string } } | { tag: "server_joined", value: { host: string, port: u16, timestamp: string } } | { tag: "edited" } | { tag: "content_install_finished", value: { project_ids: string[] } } | { tag: "content_install_failed", value: { project_ids: string[], message: string } } | { tag: "removed" }
export type InstanceGroupsChangedPayload = { instance_ids: string[] }
export type FriendPayload = { tag: "friend_request", value: { from: string } } | { tag: "user_offline", value: { id: string } } | { tag: "status_update", value: { user_status: FriendStatusPayload } } | { tag: "status_sync" }
export type FriendStatusPayload = { user_id: string, profile_name: string | undefined, last_update: string }
export type SharedInstanceUnavailableReason = { tag: "deleted" } | { tag: "access_revoked" } | { tag: "quarantined" }
export type LogEvent = { tag: "log4j", value: Log4jEvent } | { tag: "legacy", value: { message: string } }
export type LogPayload = { instance_id: string, event: LogEvent }
export type OnboardingChecklist = { has_created_instance: boolean, has_logged_into_minecraft: boolean, has_logged_into_modrinth: boolean, show_checklist: boolean }
export type Log4jEvent = { timestamp_millis: i64 | undefined, logger_name: string | undefined, level: string | undefined, thread_name: string | undefined, message: string | undefined, throwable: string | undefined }
export type ModLoader = { tag: "vanilla" } | { tag: "forge" } | { tag: "fabric" } | { tag: "quilt" } | { tag: "neoforge" }
export type InstallJobSnapshot = { job_id: string, instance_id: string | undefined, kind: InstallJobKind, status: InstallJobStatus, target: InstallTarget, phase: InstallPhaseId, progress: InstallProgress | undefined, details: InstallPhaseDetails, display: InstallJobDisplay | undefined, error: InstallErrorView | undefined, rollback_error: InstallErrorView | undefined, created: string, modified: string, finished: string | undefined }
export type InstallJobKind = { tag: "create_instance" } | { tag: "create_modpack_instance" } | { tag: "create_shared_instance" } | { tag: "import_instance" } | { tag: "duplicate_instance" } | { tag: "install_existing_instance" } | { tag: "install_pack_to_existing_instance" } | { tag: "update_shared_instance" }
export type InstallJobStatus = { tag: "queued" } | { tag: "running" } | { tag: "succeeded" } | { tag: "failed" } | { tag: "interrupted" } | { tag: "canceled" }
export type InstallTarget = { tag: "new_instance", value: { instance_id: string | undefined } } | { tag: "existing_instance", value: { instance_id: string } }
export type InstallPhaseId = { tag: "preparing_instance" } | { tag: "resolving_pack" } | { tag: "downloading_pack_file" } | { tag: "reading_pack_manifest" } | { tag: "downloading_content" } | { tag: "extracting_overrides" } | { tag: "resolving_minecraft" } | { tag: "resolving_loader" } | { tag: "preparing_java" } | { tag: "downloading_minecraft" } | { tag: "running_loader_processors" } | { tag: "finalizing" } | { tag: "rolling_back" }
export type InstallProgress = { current: u64, total: u64, secondary: InstallProgressSecondary | undefined }
export type InstallProgressSecondary = { current: u64, total: u64 }
export type InstallPhaseDetails = { tag: "empty" } | { tag: "instance", value: { name: string } } | { tag: "minecraft", value: { game_version: string, loader: ModLoader } } | { tag: "java", value: { major_version: u32, step: InstallJavaStep } } | { tag: "modpack", value: { project_id: string | undefined, version_id: string | undefined, title: string | undefined } } | { tag: "import", value: { launcher_type: ImportLauncherType, instance_folder: string } }
export type InstallJavaStep = { tag: "resolving" } | { tag: "fetching_metadata" } | { tag: "downloading" } | { tag: "extracting" } | { tag: "validating" }
export type InstallJobDisplay = { title: string, icon: string | undefined }
export type InstallErrorView = { code: string, phase: InstallPhaseId | undefined, message: string, reason: SharedInstanceUnavailableReason | undefined, api: InstallApiErrorDetails | undefined, context: InstallErrorContext | undefined }
export type InstallApiErrorDetails = { error: string, status: u16 | undefined, method: string | undefined, url: string | undefined, route: string | undefined }
export type InstallErrorContext = { operation: string, source_path: string | undefined, target_path: string | undefined, file_path: string | undefined, entry_path: string | undefined, urls: string[], expected_hash: string | undefined, expected_size: u64 | undefined, project_id: string | undefined, version_id: string | undefined, minecraft_version: string | undefined, loader: string | undefined, java_version: u32 | undefined, os: string | undefined, arch: string | undefined }
export type ImportLauncherType = { tag: "MultiMC" } | { tag: "PrismLauncher" } | { tag: "ATLauncher" } | { tag: "GDLauncher" } | { tag: "Curseforge" } | { tag: "Unknown" }

export type Type = "AppEvent" | "LoadingBarType" | "LoadingPayload" | "WarningPayload" | "InstanceBulkUpdateProgressPayload" | "InstanceBulkUpdateProgressStage" | "CommandPayload" | "ProcessPayload" | "ProcessPayloadType" | "InstancePayload" | "InstancePayloadType" | "InstanceGroupsChangedPayload" | "OnboardingChecklist" | "FriendPayload" | "FriendStatusPayload" | "LogEvent" | "LogPayload" | "Log4jEvent" | "InstallJobSnapshot" | "InstallJobKind" | "InstallJobStatus" | "InstallTarget" | "InstallPhaseId" | "InstallProgress" | "InstallProgressSecondary" | "InstallPhaseDetails" | "InstallJavaStep" | "InstallJobDisplay" | "InstallErrorView" | "InstallApiErrorDetails" | "InstallErrorContext" | "ImportLauncherType" | "ModLoader" | "SharedInstanceUnavailableReason"
declare type ValueType<T extends Type> = T extends "AppEvent" ? AppEvent : T extends "LoadingBarType" ? LoadingBarType : T extends "LoadingPayload" ? LoadingPayload : T extends "WarningPayload" ? WarningPayload : T extends "InstanceBulkUpdateProgressPayload" ? InstanceBulkUpdateProgressPayload : T extends "InstanceBulkUpdateProgressStage" ? InstanceBulkUpdateProgressStage : T extends "CommandPayload" ? CommandPayload : T extends "ProcessPayload" ? ProcessPayload : T extends "ProcessPayloadType" ? ProcessPayloadType : T extends "InstancePayload" ? InstancePayload : T extends "InstancePayloadType" ? InstancePayloadType : T extends "InstanceGroupsChangedPayload" ? InstanceGroupsChangedPayload : T extends "OnboardingChecklist" ? OnboardingChecklist : T extends "FriendPayload" ? FriendPayload : T extends "FriendStatusPayload" ? FriendStatusPayload : T extends "LogEvent" ? LogEvent : T extends "LogPayload" ? LogPayload : T extends "Log4jEvent" ? Log4jEvent : T extends "InstallJobSnapshot" ? InstallJobSnapshot : T extends "InstallJobKind" ? InstallJobKind : T extends "InstallJobStatus" ? InstallJobStatus : T extends "InstallTarget" ? InstallTarget : T extends "InstallPhaseId" ? InstallPhaseId : T extends "InstallProgress" ? InstallProgress : T extends "InstallProgressSecondary" ? InstallProgressSecondary : T extends "InstallPhaseDetails" ? InstallPhaseDetails : T extends "InstallJavaStep" ? InstallJavaStep : T extends "InstallJobDisplay" ? InstallJobDisplay : T extends "InstallErrorView" ? InstallErrorView : T extends "InstallApiErrorDetails" ? InstallApiErrorDetails : T extends "InstallErrorContext" ? InstallErrorContext : T extends "ImportLauncherType" ? ImportLauncherType : T extends "ModLoader" ? ModLoader : T extends "SharedInstanceUnavailableReason" ? SharedInstanceUnavailableReason : void

export interface Result<T extends Type> {
    value: ValueType<T>;
    bytes: Uint8Array;
}

export function deserialize<T extends Type>(type: T, bytes: Uint8Array): Result<T>
