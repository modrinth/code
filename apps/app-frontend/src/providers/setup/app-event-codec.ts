import type { AppEvent } from '@/generated/app-events/AppEvent'
import { deserialize } from '@/generated/app-events/postcard'

type WireObject = Record<string, unknown>

interface WireEnum {
	tag: string
	value?: unknown
}

function wireObject(value: unknown): WireObject {
	if (value === null || typeof value !== 'object' || Array.isArray(value)) {
		throw new TypeError('Invalid Postcard app event object')
	}
	return value as WireObject
}

function wireEnum(value: unknown): WireEnum {
	const event = wireObject(value)
	if (typeof event.tag !== 'string') {
		throw new TypeError('Invalid Postcard app event enum')
	}
	return event as unknown as WireEnum
}

function taggedObject(value: unknown, tag: string): WireObject {
	const event = wireEnum(value)
	return {
		[tag]: event.tag,
		...(event.value === undefined ? {} : wireObject(event.value)),
	}
}

function unitVariant(value: unknown): string {
	return wireEnum(value).tag
}

function nullable(value: unknown): unknown {
	return value === undefined ? null : value
}

function number(value: unknown): unknown {
	return typeof value === 'bigint' ? Number(value) : value
}

function normalizeLoadingPayload(value: unknown): WireObject {
	const payload = wireObject(value)
	const event = taggedObject(payload.event, 'type')

	for (const field of ['icon', 'pack_id', 'pack_version']) {
		if (field in event) event[field] = nullable(event[field])
	}

	return {
		...payload,
		event,
		fraction: nullable(payload.fraction),
	}
}

function normalizeProcessPayload(value: unknown): WireObject {
	const payload = wireObject(value)
	return { ...payload, event: unitVariant(payload.event) }
}

function normalizeInstancePayload(value: unknown): WireObject {
	const payload = wireObject(value)
	const event = wireEnum(payload.event)
	return {
		instance_id: payload.instance_id,
		event: event.tag,
		...(event.value === undefined ? {} : wireObject(event.value)),
	}
}

function normalizeBulkUpdatePayload(value: unknown): WireObject {
	const payload = wireObject(value)
	return {
		...payload,
		stage: unitVariant(payload.stage),
		current: number(payload.current),
		total: number(payload.total),
	}
}

function normalizeCommandPayload(value: unknown): WireObject {
	const command = taggedObject(value, 'event')
	if (command.event === 'LaunchInstance') {
		command.server = nullable(command.server)
		command.singleplayer_world = nullable(command.singleplayer_world)
	}
	return command
}

function normalizeFriendPayload(value: unknown): WireObject {
	const event = taggedObject(value, 'event')
	if (event.event === 'status_update') {
		const status = wireObject(event.user_status)
		event.user_status = {
			...status,
			profile_name: nullable(status.profile_name),
		}
	}
	return event
}

function normalizeLogPayload(value: unknown): WireObject {
	const payload = wireObject(value)
	const event = taggedObject(payload.event, 'type')

	if (event.type === 'log4j') {
		for (const field of [
			'timestamp_millis',
			'logger_name',
			'level',
			'thread_name',
			'message',
			'throwable',
		]) {
			event[field] = nullable(event[field])
		}
		event.timestamp_millis = number(event.timestamp_millis)
	}

	return { instance_id: payload.instance_id, ...event }
}

function normalizeInstallProgress(value: unknown): WireObject {
	const progress = wireObject(value)
	const secondary = progress.secondary

	return {
		...progress,
		current: number(progress.current),
		total: number(progress.total),
		secondary:
			secondary === undefined
				? undefined
				: {
						...wireObject(secondary),
						current: number(wireObject(secondary).current),
						total: number(wireObject(secondary).total),
					},
	}
}

function normalizeInstallDetails(value: unknown): WireObject {
	const details = taggedObject(value, 'type')

	if (details.type === 'minecraft') details.loader = unitVariant(details.loader)
	if (details.type === 'java') details.step = unitVariant(details.step)
	if (details.type === 'import') details.launcher_type = unitVariant(details.launcher_type)
	if (details.type === 'modpack') {
		details.project_id = nullable(details.project_id)
		details.version_id = nullable(details.version_id)
		details.title = nullable(details.title)
	}

	return details
}

function normalizeInstallContext(value: unknown): WireObject {
	const context = wireObject(value)
	return {
		...context,
		expected_size: context.expected_size === undefined ? undefined : number(context.expected_size),
	}
}

function normalizeInstallError(value: unknown): WireObject {
	const error = wireObject(value)
	return {
		...error,
		phase: error.phase === undefined ? undefined : unitVariant(error.phase),
		reason: error.reason === undefined ? undefined : unitVariant(error.reason),
		api: error.api === undefined ? undefined : wireObject(error.api),
		context: error.context === undefined ? undefined : normalizeInstallContext(error.context),
	}
}

function normalizeInstallJob(value: unknown): WireObject {
	const job = wireObject(value)
	const display = job.display === undefined ? null : wireObject(job.display)

	if (display !== null) display.icon = nullable(display.icon)

	return {
		...job,
		instance_id: nullable(job.instance_id),
		kind: unitVariant(job.kind),
		status: unitVariant(job.status),
		target: (() => {
			const target = taggedObject(job.target, 'type')
			if ('instance_id' in target) target.instance_id = nullable(target.instance_id)
			return target
		})(),
		phase: unitVariant(job.phase),
		progress: job.progress === undefined ? null : normalizeInstallProgress(job.progress),
		details: normalizeInstallDetails(job.details),
		display,
		error: job.error === undefined ? null : normalizeInstallError(job.error),
		rollback_error:
			job.rollback_error === undefined ? null : normalizeInstallError(job.rollback_error),
		finished: nullable(job.finished),
	}
}

export function decodeAppEvent(payload: ArrayBuffer): AppEvent {
	const result = deserialize('AppEvent', new Uint8Array(payload))
	if (result.bytes.length !== 0) {
		throw new TypeError('Postcard app event contained trailing bytes')
	}
	const event = result.value as WireEnum

	const decoded = (() => {
		switch (event.tag) {
			case 'loading':
				return { type: event.tag, payload: normalizeLoadingPayload(event.value) }
			case 'process':
				return { type: event.tag, payload: normalizeProcessPayload(event.value) }
			case 'instance':
				return { type: event.tag, payload: normalizeInstancePayload(event.value) }
			case 'instance_groups_changed':
				return { type: event.tag, payload: wireObject(event.value) }
			case 'onboarding_checklist':
				return { type: event.tag, payload: wireObject(event.value) }
			case 'instance_bulk_update_progress':
				return { type: event.tag, payload: normalizeBulkUpdatePayload(event.value) }
			case 'install_job':
				return { type: event.tag, payload: normalizeInstallJob(event.value) }
			case 'command':
				return { type: event.tag, payload: normalizeCommandPayload(event.value) }
			case 'warning':
				return { type: event.tag, payload: wireObject(event.value) }
			case 'friend':
				return { type: event.tag, payload: normalizeFriendPayload(event.value) }
			case 'notification':
				return { type: event.tag, payload: JSON.parse(String(event.value)) as unknown }
			case 'log':
				return { type: event.tag, payload: normalizeLogPayload(event.value) }
			case 'ads_consent_required':
				return { type: event.tag, payload: event.value }
			default:
				throw new TypeError(`Unknown Postcard app event: ${event.tag}`)
		}
	})()

	return decoded as AppEvent
}
