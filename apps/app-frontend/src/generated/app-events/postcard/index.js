const BITS_PER_BYTE = 8, BITS_PER_VARINT_BYTE = 7, U8_BYTES = 1, U16_BYTES = 2, U32_BYTES = 4, U64_BYTES = 8, U128_BYTES = 16

const de_zig_zag_signed = (n) => (n >> 1n) ^ (-(n & 0b1n))
const zig_zag = (n_bytes, n) => (n << 1n) ^ (n >> BigInt(n_bytes * BITS_PER_BYTE - 1))
const varint_max = (n_bytes) => Math.floor((n_bytes * BITS_PER_BYTE + (BITS_PER_BYTE - 1)) / BITS_PER_VARINT_BYTE)
const max_of_last_byte = (n_bytes) => (1 << (n_bytes * BITS_PER_BYTE) % 7) - 1
const to_number_if_safe = (n) => Number.MAX_SAFE_INTEGER < ((n < 0n) ? -n : n) ? n : Number(n)
const varint = (n_bytes, n) => { let value = BigInt(n), out = []; for (let i = 0; i < varint_max(n_bytes); i++) { out.push(Number(value & 0xFFn)); if (value < 128n) { return out } out[i] |= 0x80; value >>= 7n } }

class Deserializer {
    constructor(bytes_in) { this.bytes = Array.from(bytes_in); }
    pop_next = () => { const next = this.bytes.shift(); if (next === undefined) { throw "input buffer too small" } return next }
    pop_n = (n) => { const bytes = Array(); for (let i = 0; i < n; i++) { bytes.push(this.bytes.shift()) } return bytes }
    get_int8 = (signed) => signed ? new Int8Array([this.pop_next()])[0] : this.pop_next();
    try_take = (n_bytes) => { let out = 0n, v_max = varint_max(n_bytes); for (let i = 0; i < v_max; i++) { const val = this.pop_next(), carry = BigInt(val & 0x7F); out |= carry << BigInt(7 * i); if ((val & 0x80) === 0) { if (i === v_max - 1 && val > max_of_last_byte(n_bytes)) { throw "Bad Variant" } else return out } } throw "Bad Variant"; }
    deserialize_bool = () => { const byte = this.pop_next(); return byte === undefined ? undefined : byte > 0 ? true : false }
    deserialize_number = (n_bytes, signed) => { if (n_bytes === U8_BYTES) { return this.get_int8(signed) } else if (n_bytes === U16_BYTES || n_bytes === U32_BYTES || n_bytes === U64_BYTES || n_bytes === U128_BYTES) { const val = this.try_take(n_bytes); return to_number_if_safe(signed ? de_zig_zag_signed(val) : val) } else { throw "byte count not supported" } }
    deserialize_number_float = (n_bytes) => { const b_buffer = new ArrayBuffer(n_bytes), b_view = new DataView(b_buffer); this.pop_n(n_bytes).forEach((b, i) => b_view.setUint8(i, b)); if (n_bytes === U32_BYTES) { return b_view.getFloat32(0, true) } else if (n_bytes === U64_BYTES) { return b_view.getFloat64(0, true) } else { throw "byte count not supported" } }
    deserialize_string = () => new TextDecoder().decode(new Uint8Array(this.pop_n(Number(this.try_take(U32_BYTES)))))
    deserialize_array = (des, len) => Array.from({length: len === undefined ? Number(this.try_take(U32_BYTES)) : len}, (v, i) => des(this))
    deserialize_string_key_map = (des) => { return [...Array(Number(this.try_take(U32_BYTES)))].reduce((prev) => { prev[this.deserialize_string()] = des(this); return prev }, {}) }
    deserialize_map = (des) => { return [...Array(Number(this.try_take(U32_BYTES)))].reduce((prev) => { const d = des(this); prev.set(d[0], d[1]); return prev }, new Map()) }
    release_bytes = () => { return new Uint8Array(this.bytes); }
}

function deserialize_APP_EVENT(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "loading",
            value: deserialize_LOADING_PAYLOAD(d)
        };
    case 1:
        return {
            tag: "process",
            value: deserialize_PROCESS_PAYLOAD(d)
        };
    case 2:
        return {
            tag: "instance",
            value: deserialize_INSTANCE_PAYLOAD(d)
        };
    case 3:
        return {
            tag: "instance_groups_changed",
            value: deserialize_INSTANCE_GROUPS_CHANGED_PAYLOAD(d)
        };
    case 4:
        return {
            tag: "onboarding_checklist",
            value: deserialize_ONBOARDING_CHECKLIST(d)
        };
    case 5:
        return {
            tag: "instance_bulk_update_progress",
            value: deserialize_INSTANCE_BULK_UPDATE_PROGRESS_PAYLOAD(d)
        };
    case 6:
        return {
            tag: "install_job",
            value: deserialize_INSTALL_JOB_SNAPSHOT(d)
        };
    case 7:
        return {
            tag: "command",
            value: deserialize_COMMAND_PAYLOAD(d)
        };
    case 8:
        return {
            tag: "warning",
            value: deserialize_WARNING_PAYLOAD(d)
        };
    case 9:
        return {
            tag: "friend",
            value: deserialize_FRIEND_PAYLOAD(d)
        };
    case 10:
        return {
            tag: "notification",
            value: d.deserialize_string()
        };
    case 11:
        return {
            tag: "log",
            value: deserialize_LOG_PAYLOAD(d)
        };
    case 12:
        return {
            tag: "ads_consent_required",
            value: d.deserialize_bool()
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_LOADING_BAR_TYPE(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "legacy_data_migration"
        };
    case 1:
        return {
            tag: "directory_move",
            value: {
                old: d.deserialize_string(),
                new: d.deserialize_string()
            }
        };
    case 2:
        return {
            tag: "java_download",
            value: {
                version: d.deserialize_number(U32_BYTES, false)
            }
        };
    case 3:
        return {
            tag: "pack_file_download",
            value: {
                instance_id: d.deserialize_string(),
                pack_name: d.deserialize_string(),
                icon: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                pack_version: d.deserialize_string()
            }
        };
    case 4:
        return {
            tag: "pack_download",
            value: {
                instance_id: d.deserialize_string(),
                pack_name: d.deserialize_string(),
                icon: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                pack_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                pack_version: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
            }
        };
    case 5:
        return {
            tag: "pack_import",
            value: {
                pack_name: d.deserialize_string()
            }
        };
    case 6:
        return {
            tag: "minecraft_download",
            value: {
                instance_id: d.deserialize_string(),
                instance_name: d.deserialize_string()
            }
        };
    case 7:
        return {
            tag: "instance_update",
            value: {
                instance_id: d.deserialize_string(),
                instance_name: d.deserialize_string()
            }
        };
    case 8:
        return {
            tag: "zip_extract",
            value: {
                instance_id: d.deserialize_string(),
                instance_name: d.deserialize_string()
            }
        };
    case 9:
        return {
            tag: "pack_export",
            value: {
                instance_id: d.deserialize_string(),
                instance_name: d.deserialize_string()
            }
        };
    case 10:
        return {
            tag: "config_change",
            value: {
                new_path: d.deserialize_string()
            }
        };
    case 11:
        return {
            tag: "copy_instance",
            value: {
                import_location: d.deserialize_string(),
                instance_name: d.deserialize_string()
            }
        };
    case 12:
        return {
            tag: "launcher_update",
            value: {
                version: d.deserialize_string(),
                current_version: d.deserialize_string()
            }
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_LOADING_PAYLOAD(d) {
    return {
        event: deserialize_LOADING_BAR_TYPE(d),
        loader_uuid: d.deserialize_string(),
        fraction: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_number_float(U64_BYTES),
        message: d.deserialize_string()
    };
}

function deserialize_WARNING_PAYLOAD(d) {
    return {
        message: d.deserialize_string()
    };
}

function deserialize_INSTANCE_BULK_UPDATE_PROGRESS_PAYLOAD(d) {
    return {
        instanceId: d.deserialize_string(),
        stage: deserialize_INSTANCE_BULK_UPDATE_PROGRESS_STAGE(d),
        current: d.deserialize_number(U64_BYTES, false),
        total: d.deserialize_number(U64_BYTES, false)
    };
}

function deserialize_INSTANCE_BULK_UPDATE_PROGRESS_STAGE(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "resolving_versions"
        };
    case 1:
        return {
            tag: "downloading"
        };
    case 2:
        return {
            tag: "finishing"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_COMMAND_PAYLOAD(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "InstallMod",
            value: {
                id: d.deserialize_string()
            }
        };
    case 1:
        return {
            tag: "InstallVersion",
            value: {
                id: d.deserialize_string()
            }
        };
    case 2:
        return {
            tag: "InstallModpack",
            value: {
                id: d.deserialize_string()
            }
        };
    case 3:
        return {
            tag: "InstallServer",
            value: {
                id: d.deserialize_string()
            }
        };
    case 4:
        return {
            tag: "LaunchInstance",
            value: {
                id: d.deserialize_string(),
                server: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                singleplayer_world: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
            }
        };
    case 5:
        return {
            tag: "InstallSharedInstanceInvite",
            value: {
                invite_id: d.deserialize_string()
            }
        };
    case 6:
        return {
            tag: "RunMRPack",
            value: {
                path: d.deserialize_string()
            }
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_PROCESS_PAYLOAD(d) {
    return {
        instance_id: d.deserialize_string(),
        uuid: d.deserialize_string(),
        event: deserialize_PROCESS_PAYLOAD_TYPE(d),
        message: d.deserialize_string()
    };
}

function deserialize_PROCESS_PAYLOAD_TYPE(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "launched"
        };
    case 1:
        return {
            tag: "finished"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTANCE_PAYLOAD(d) {
    return {
        instance_id: d.deserialize_string(),
        event: deserialize_INSTANCE_PAYLOAD_TYPE(d)
    };
}

function deserialize_INSTANCE_PAYLOAD_TYPE(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "created"
        };
    case 1:
        return {
            tag: "synced"
        };
    case 2:
        return {
            tag: "servers_updated"
        };
    case 3:
        return {
            tag: "world_updated",
            value: {
                world: d.deserialize_string()
            }
        };
    case 4:
        return {
            tag: "server_joined",
            value: {
                host: d.deserialize_string(),
                port: d.deserialize_number(U16_BYTES, false),
                timestamp: d.deserialize_string()
            }
        };
    case 5:
        return {
            tag: "edited"
        };
    case 6:
        return {
            tag: "content_install_finished",
            value: {
                project_ids: d.deserialize_array(() => d.deserialize_string())
            }
        };
    case 7:
        return {
            tag: "content_install_failed",
            value: {
                project_ids: d.deserialize_array(() => d.deserialize_string()),
                message: d.deserialize_string()
            }
        };
    case 8:
        return {
            tag: "removed"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTANCE_GROUPS_CHANGED_PAYLOAD(d) {
    return {
        instance_ids: d.deserialize_array(() => d.deserialize_string())
    };
}

function deserialize_ONBOARDING_CHECKLIST(d) {
    return {
        has_created_instance: d.deserialize_bool(),
        has_logged_into_minecraft: d.deserialize_bool(),
        has_logged_into_modrinth: d.deserialize_bool(),
        show_checklist: d.deserialize_bool()
    };
}

function deserialize_FRIEND_PAYLOAD(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "friend_request",
            value: {
                from: d.deserialize_string()
            }
        };
    case 1:
        return {
            tag: "user_offline",
            value: {
                id: d.deserialize_string()
            }
        };
    case 2:
        return {
            tag: "status_update",
            value: {
                user_status: deserialize_FRIEND_STATUS_PAYLOAD(d)
            }
        };
    case 3:
        return {
            tag: "status_sync"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_FRIEND_STATUS_PAYLOAD(d) {
    return {
        user_id: d.deserialize_string(),
        profile_name: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        last_update: d.deserialize_string()
    };
}

function deserialize_LOG_EVENT(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "log4j",
            value: deserialize_LOG4J_EVENT(d)
        };
    case 1:
        return {
            tag: "legacy",
            value: {
                message: d.deserialize_string()
            }
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_LOG_PAYLOAD(d) {
    return {
        instance_id: d.deserialize_string(),
        event: deserialize_LOG_EVENT(d)
    };
}

function deserialize_LOG4J_EVENT(d) {
    return {
        timestamp_millis: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_number(U64_BYTES, true),
        logger_name: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        level: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        thread_name: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        message: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        throwable: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
    };
}

function deserialize_INSTALL_JOB_SNAPSHOT(d) {
    return {
        job_id: d.deserialize_string(),
        instance_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        kind: deserialize_INSTALL_JOB_KIND(d),
        status: deserialize_INSTALL_JOB_STATUS(d),
        target: deserialize_INSTALL_TARGET(d),
        phase: deserialize_INSTALL_PHASE_ID(d),
        progress: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_PROGRESS(d),
        details: deserialize_INSTALL_PHASE_DETAILS(d),
        display: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_JOB_DISPLAY(d),
        error: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_ERROR_VIEW(d),
        rollback_error: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_ERROR_VIEW(d),
        created: d.deserialize_string(),
        modified: d.deserialize_string(),
        finished: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
    };
}

function deserialize_INSTALL_JOB_KIND(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "create_instance"
        };
    case 1:
        return {
            tag: "create_modpack_instance"
        };
    case 2:
        return {
            tag: "create_shared_instance"
        };
    case 3:
        return {
            tag: "import_instance"
        };
    case 4:
        return {
            tag: "duplicate_instance"
        };
    case 5:
        return {
            tag: "install_existing_instance"
        };
    case 6:
        return {
            tag: "install_pack_to_existing_instance"
        };
    case 7:
        return {
            tag: "update_shared_instance"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_JOB_STATUS(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "queued"
        };
    case 1:
        return {
            tag: "running"
        };
    case 2:
        return {
            tag: "succeeded"
        };
    case 3:
        return {
            tag: "failed"
        };
    case 4:
        return {
            tag: "interrupted"
        };
    case 5:
        return {
            tag: "canceled"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_TARGET(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "new_instance",
            value: {
                instance_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
            }
        };
    case 1:
        return {
            tag: "existing_instance",
            value: {
                instance_id: d.deserialize_string()
            }
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_PHASE_ID(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "preparing_instance"
        };
    case 1:
        return {
            tag: "resolving_pack"
        };
    case 2:
        return {
            tag: "downloading_pack_file"
        };
    case 3:
        return {
            tag: "reading_pack_manifest"
        };
    case 4:
        return {
            tag: "downloading_content"
        };
    case 5:
        return {
            tag: "extracting_overrides"
        };
    case 6:
        return {
            tag: "resolving_minecraft"
        };
    case 7:
        return {
            tag: "resolving_loader"
        };
    case 8:
        return {
            tag: "preparing_java"
        };
    case 9:
        return {
            tag: "downloading_minecraft"
        };
    case 10:
        return {
            tag: "running_loader_processors"
        };
    case 11:
        return {
            tag: "finalizing"
        };
    case 12:
        return {
            tag: "rolling_back"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_PROGRESS(d) {
    return {
        current: d.deserialize_number(U64_BYTES, false),
        total: d.deserialize_number(U64_BYTES, false),
        secondary: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_PROGRESS_SECONDARY(d)
    };
}

function deserialize_INSTALL_PROGRESS_SECONDARY(d) {
    return {
        current: d.deserialize_number(U64_BYTES, false),
        total: d.deserialize_number(U64_BYTES, false)
    };
}

function deserialize_INSTALL_PHASE_DETAILS(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "empty"
        };
    case 1:
        return {
            tag: "instance",
            value: {
                name: d.deserialize_string()
            }
        };
    case 2:
        return {
            tag: "minecraft",
            value: {
                game_version: d.deserialize_string(),
                loader: deserialize_MOD_LOADER(d)
            }
        };
    case 3:
        return {
            tag: "java",
            value: {
                major_version: d.deserialize_number(U32_BYTES, false),
                step: deserialize_INSTALL_JAVA_STEP(d)
            }
        };
    case 4:
        return {
            tag: "modpack",
            value: {
                project_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                version_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
                title: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
            }
        };
    case 5:
        return {
            tag: "import",
            value: {
                launcher_type: deserialize_IMPORT_LAUNCHER_TYPE(d),
                instance_folder: d.deserialize_string()
            }
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_JAVA_STEP(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "resolving"
        };
    case 1:
        return {
            tag: "fetching_metadata"
        };
    case 2:
        return {
            tag: "downloading"
        };
    case 3:
        return {
            tag: "extracting"
        };
    case 4:
        return {
            tag: "validating"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_INSTALL_JOB_DISPLAY(d) {
    return {
        title: d.deserialize_string(),
        icon: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
    };
}

function deserialize_INSTALL_ERROR_VIEW(d) {
    return {
        code: d.deserialize_string(),
        phase: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_PHASE_ID(d),
        message: d.deserialize_string(),
        reason: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_SHARED_INSTANCE_UNAVAILABLE_REASON(d),
        api: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_API_ERROR_DETAILS(d),
        context: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : deserialize_INSTALL_ERROR_CONTEXT(d)
    };
}

function deserialize_INSTALL_API_ERROR_DETAILS(d) {
    return {
        error: d.deserialize_string(),
        status: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_number(U16_BYTES, false),
        method: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        url: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        route: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
    };
}

function deserialize_INSTALL_ERROR_CONTEXT(d) {
    return {
        operation: d.deserialize_string(),
        source_path: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        target_path: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        file_path: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        entry_path: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        urls: d.deserialize_array(() => d.deserialize_string()),
        expected_hash: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        expected_size: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_number(U64_BYTES, false),
        project_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        version_id: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        minecraft_version: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        loader: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        java_version: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_number(U32_BYTES, false),
        os: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string(),
        arch: (d.deserialize_number(U32_BYTES, false) === 0) ? undefined : d.deserialize_string()
    };
}

function deserialize_IMPORT_LAUNCHER_TYPE(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "MultiMC"
        };
    case 1:
        return {
            tag: "PrismLauncher"
        };
    case 2:
        return {
            tag: "ATLauncher"
        };
    case 3:
        return {
            tag: "GDLauncher"
        };
    case 4:
        return {
            tag: "Curseforge"
        };
    case 5:
        return {
            tag: "Unknown"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_MOD_LOADER(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "vanilla"
        };
    case 1:
        return {
            tag: "forge"
        };
    case 2:
        return {
            tag: "fabric"
        };
    case 3:
        return {
            tag: "quilt"
        };
    case 4:
        return {
            tag: "neoforge"
        };
    default:
        throw "variant not implemented"
    }
}

function deserialize_SHARED_INSTANCE_UNAVAILABLE_REASON(d) {
    switch (d.deserialize_number(U32_BYTES, false)) {
    case 0:
        return {
            tag: "deleted"
        };
    case 1:
        return {
            tag: "access_revoked"
        };
    case 2:
        return {
            tag: "quarantined"
        };
    default:
        throw "variant not implemented"
    }
}

/**
 * Deserialize a value from an array of bytes.
 * @param {string} type - The type of the value to deserialize.
 * @param {Uint8Array} bytes - The byte array to deserialize from.
 * @return {Object} The deserialized value and remaining bytes.
 */
function deserialize(type, bytes) {
    if (!(typeof type === "string")) {
        throw "type must be a string";
    }
    const d = new Deserializer(bytes);
    var return_value = undefined;
    switch (type) {
    case "AppEvent":
        return_value = deserialize_APP_EVENT(d);
        break;
    case "LoadingBarType":
        return_value = deserialize_LOADING_BAR_TYPE(d);
        break;
    case "LoadingPayload":
        return_value = deserialize_LOADING_PAYLOAD(d);
        break;
    case "WarningPayload":
        return_value = deserialize_WARNING_PAYLOAD(d);
        break;
    case "InstanceBulkUpdateProgressPayload":
        return_value = deserialize_INSTANCE_BULK_UPDATE_PROGRESS_PAYLOAD(d);
        break;
    case "InstanceBulkUpdateProgressStage":
        return_value = deserialize_INSTANCE_BULK_UPDATE_PROGRESS_STAGE(d);
        break;
    case "CommandPayload":
        return_value = deserialize_COMMAND_PAYLOAD(d);
        break;
    case "ProcessPayload":
        return_value = deserialize_PROCESS_PAYLOAD(d);
        break;
    case "ProcessPayloadType":
        return_value = deserialize_PROCESS_PAYLOAD_TYPE(d);
        break;
    case "InstancePayload":
        return_value = deserialize_INSTANCE_PAYLOAD(d);
        break;
    case "InstancePayloadType":
        return_value = deserialize_INSTANCE_PAYLOAD_TYPE(d);
        break;
    case "InstanceGroupsChangedPayload":
        return_value = deserialize_INSTANCE_GROUPS_CHANGED_PAYLOAD(d);
        break;
    case "OnboardingChecklist":
        return_value = deserialize_ONBOARDING_CHECKLIST(d);
        break;
    case "FriendPayload":
        return_value = deserialize_FRIEND_PAYLOAD(d);
        break;
    case "FriendStatusPayload":
        return_value = deserialize_FRIEND_STATUS_PAYLOAD(d);
        break;
    case "LogEvent":
        return_value = deserialize_LOG_EVENT(d);
        break;
    case "LogPayload":
        return_value = deserialize_LOG_PAYLOAD(d);
        break;
    case "Log4jEvent":
        return_value = deserialize_LOG4J_EVENT(d);
        break;
    case "InstallJobSnapshot":
        return_value = deserialize_INSTALL_JOB_SNAPSHOT(d);
        break;
    case "InstallJobKind":
        return_value = deserialize_INSTALL_JOB_KIND(d);
        break;
    case "InstallJobStatus":
        return_value = deserialize_INSTALL_JOB_STATUS(d);
        break;
    case "InstallTarget":
        return_value = deserialize_INSTALL_TARGET(d);
        break;
    case "InstallPhaseId":
        return_value = deserialize_INSTALL_PHASE_ID(d);
        break;
    case "InstallProgress":
        return_value = deserialize_INSTALL_PROGRESS(d);
        break;
    case "InstallProgressSecondary":
        return_value = deserialize_INSTALL_PROGRESS_SECONDARY(d);
        break;
    case "InstallPhaseDetails":
        return_value = deserialize_INSTALL_PHASE_DETAILS(d);
        break;
    case "InstallJavaStep":
        return_value = deserialize_INSTALL_JAVA_STEP(d);
        break;
    case "InstallJobDisplay":
        return_value = deserialize_INSTALL_JOB_DISPLAY(d);
        break;
    case "InstallErrorView":
        return_value = deserialize_INSTALL_ERROR_VIEW(d);
        break;
    case "InstallApiErrorDetails":
        return_value = deserialize_INSTALL_API_ERROR_DETAILS(d);
        break;
    case "InstallErrorContext":
        return_value = deserialize_INSTALL_ERROR_CONTEXT(d);
        break;
    case "ImportLauncherType":
        return_value = deserialize_IMPORT_LAUNCHER_TYPE(d);
        break;
    case "ModLoader":
        return_value = deserialize_MOD_LOADER(d);
        break;
    case "SharedInstanceUnavailableReason":
        return_value = deserialize_SHARED_INSTANCE_UNAVAILABLE_REASON(d);
        break;
    default:
        throw "type not implemented";
    }
    return { value: return_value, bytes: d.release_bytes() };
}

export {
    deserialize
};
