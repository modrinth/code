import type { PostHog } from 'posthog-js'

interface InstanceProperties {
	loader: string
	game_version: string
}

interface ProjectProperties extends InstanceProperties {
	id: string
	project_type: string
}

type AnalyticsEventMap = {
	Launched: { version: string; dev: boolean }
	PageView: { path: string; fromPath: string; failed: unknown }
	InstanceCreate: { source: string }
	InstanceCreateStart: { source: string }
	InstanceStart: InstanceProperties & { source: string }
	InstanceStop: Partial<InstanceProperties> & { source?: string }
	InstanceDuplicate: InstanceProperties
	InstanceRepair: InstanceProperties
	InstanceSetIcon: Record<string, never>
	InstanceRemoveIcon: Record<string, never>
	InstanceUpdateAll: InstanceProperties & { count: number; selected: boolean }
	InstanceProjectUpdate: InstanceProperties & { id: string; name: string; project_type: string }
	InstanceProjectDisable: InstanceProperties & {
		id: string
		name: string
		project_type: string
		disabled: boolean
	}
	InstanceProjectRemove: InstanceProperties & { id: string; name: string; project_type: string }
	ProjectInstall: ProjectProperties & { version_id: string; title: string; source: string }
	ProjectInstallStart: { source: string }
	PackInstall: { id: string; version_id: string; title: string; source: string }
	PackInstallStart: Record<string, never>
	AccountLogIn: { source?: string }
	AccountLogOut: Record<string, never>
	JavaTest: { path: string; success: boolean }
	JavaManualSelect: { version: string }
	JavaAutoDetect: { path: string; version: string }
}

export type AnalyticsEvent = keyof AnalyticsEventMap

let analytics: PostHog | undefined
let pending: Promise<void> | undefined
let enabled = false
let activated = false
let debug = false
let explicitlyOptedIn = false
const events: Array<{ name: AnalyticsEvent; properties: Record<string, unknown> | undefined }> = []
const allowed = import.meta.env.PROD || import.meta.env.VITE_ENABLE_ANALYTICS === 'true'

function removeActivationListeners() {
	window.removeEventListener('pointerdown', activate)
	window.removeEventListener('keydown', activate)
}

function activate() {
	activated = true
	removeActivationListeners()
	if (!enabled || pending || analytics) return
	pending = import('posthog-js').then(({ posthog }) => {
		if (!enabled) return
		posthog.init('phc_9Iqi6lFs9sr5BSqh9RRNRSJ0mATS9PSgirDiX3iOYJ', {
			persistence: 'localStorage',
			api_host: 'https://posthog.modrinth.com',
		})
		analytics = posthog
		if (explicitlyOptedIn) posthog.opt_in_capturing()
		if (debug) posthog.debug()
		for (const event of events.splice(0)) posthog.capture(event.name, event.properties)
	}).catch(() => {
		events.length = 0
	}).finally(() => {
		pending = undefined
	})
}

export const initAnalytics = () => {
	if (!allowed || enabled) return
	enabled = true
	if (activated) activate()
	else {
		window.addEventListener('pointerdown', activate, { once: true, passive: true })
		window.addEventListener('keydown', activate, { once: true })
	}
}

export const debugAnalytics = () => {
	debug = true
	analytics?.debug()
}

export const optOutAnalytics = () => {
	explicitlyOptedIn = false
	enabled = false
	events.length = 0
	removeActivationListeners()
	analytics?.opt_out_capturing()
}

export const optInAnalytics = () => {
	explicitlyOptedIn = true
	initAnalytics()
	analytics?.opt_in_capturing()
}

type OptionalArgs<T> = Record<string, never> extends T ? [properties?: T] : [properties: T]

export const trackEvent = <E extends AnalyticsEvent>(
	eventName: E,
	...args: OptionalArgs<AnalyticsEventMap[E]>
) => {
	if (!enabled) return
	if (analytics) analytics.capture(eventName, args[0])
	else {
		if (events.length >= 100) events.shift()
		events.push({ name: eventName, properties: args[0] })
	}
}
