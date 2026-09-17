/**
 * Subtle UI feedback via Web Audio (no bundled sound files).
 * Respects prefs + prefers-reduced-motion (treat as mute for motion-linked cues).
 */

const STORAGE_KEY = 'owyx.uiSounds'

export function getOwyxUiSoundsEnabled(): boolean {
	try {
		const v = localStorage.getItem(STORAGE_KEY)
		if (v === null) return true
		return v === '1' || v === 'true'
	} catch {
		return true
	}
}

export function setOwyxUiSoundsEnabled(on: boolean) {
	try {
		localStorage.setItem(STORAGE_KEY, on ? '1' : '0')
	} catch {
		/* ignore */
	}
}

let ctx: AudioContext | null = null

function audio(): AudioContext | null {
	if (typeof window === 'undefined') return null
	try {
		if (!ctx) {
			const AC =
				window.AudioContext ||
				(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
			if (!AC) return null
			ctx = new AC()
		}
		return ctx
	} catch {
		return null
	}
}

export type OwyxUiSoundKind = 'click' | 'toggle' | 'success' | 'soft'

/** Soft synthetic ticks — CC0 equivalent (generated, not sampled from elsewhere). */
export function playOwyxUiSound(kind: OwyxUiSoundKind = 'click') {
	if (!getOwyxUiSoundsEnabled()) return
	if (
		typeof window !== 'undefined' &&
		window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
	) {
		return
	}
	const ac = audio()
	if (!ac) return
	void ac.resume().catch(() => undefined)

	const now = ac.currentTime
	const osc = ac.createOscillator()
	const gain = ac.createGain()
	osc.connect(gain)
	gain.connect(ac.destination)

	const profiles: Record<
		OwyxUiSoundKind,
		{ f: number; d: number; g: number; type: OscillatorType }
	> = {
		click: { f: 880, d: 0.04, g: 0.035, type: 'sine' },
		toggle: { f: 660, d: 0.05, g: 0.03, type: 'triangle' },
		success: { f: 988, d: 0.08, g: 0.04, type: 'sine' },
		soft: { f: 520, d: 0.03, g: 0.02, type: 'sine' },
	}
	const p = profiles[kind]
	osc.type = p.type
	osc.frequency.setValueAtTime(p.f, now)
	gain.gain.setValueAtTime(0.0001, now)
	gain.gain.exponentialRampToValueAtTime(p.g, now + 0.01)
	gain.gain.exponentialRampToValueAtTime(0.0001, now + p.d)
	osc.start(now)
	osc.stop(now + p.d + 0.02)
}
