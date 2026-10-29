import { useEventListener } from '@vueuse/core'
import { computed, type Ref, ref } from 'vue'

/** Play, pause and mute state for a preview video, with a remaining-time label. */
export function useVideoPlayer(
	video: Ref<HTMLVideoElement | null>,
	onPlaybackError: (error: unknown) => void,
	defaultDurationSeconds = 16,
) {
	const started = ref(false)
	const playing = ref(false)
	const loading = ref(false)
	const muted = ref(false)
	const duration = ref(defaultDurationSeconds)
	const currentTime = ref(0)
	let playbackAttempt = 0

	const remainingTime = computed(() => {
		const seconds = started.value
			? Math.max(1, Math.ceil(duration.value - currentTime.value))
			: Math.round(duration.value)
		return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
	})

	function hasEnded(player: HTMLVideoElement) {
		return (
			player.ended ||
			(Number.isFinite(player.duration) &&
				player.duration > 0 &&
				player.currentTime >= player.duration)
		)
	}

	function sync() {
		const player = video.value
		if (!player) return
		if (Number.isFinite(player.duration)) duration.value = player.duration
		currentTime.value = player.currentTime
		if (hasEnded(player)) {
			playbackAttempt++
			loading.value = false
			playing.value = false
			started.value = false
			return
		}
		playing.value = !player.paused && player.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA
		if (playing.value) started.value = true
	}

	async function play() {
		const player = video.value
		if (!player || loading.value) return
		sync()
		if (hasEnded(player)) player.currentTime = 0
		const attempt = ++playbackAttempt
		loading.value = true
		try {
			await player.play()
			if (attempt === playbackAttempt) sync()
		} catch (error) {
			if (attempt === playbackAttempt) onPlaybackError(error)
		} finally {
			if (attempt === playbackAttempt) loading.value = false
		}
	}

	function stop() {
		playbackAttempt++
		loading.value = false
		video.value?.pause()
		sync()
	}

	function toggleMute() {
		const player = video.value
		if (!player) return
		player.muted = !player.muted
		muted.value = player.muted
	}

	function reset() {
		stop()
		started.value = false
		playing.value = false
		currentTime.value = 0
		if (video.value) video.value.currentTime = 0
	}

	useEventListener(window, 'focus', sync)
	useEventListener(document, 'visibilitychange', () => {
		if (document.visibilityState === 'visible') sync()
	})

	return { started, playing, loading, muted, remainingTime, play, stop, toggleMute, reset, sync }
}
