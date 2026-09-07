/** Loads the skin-rendering helpers when a renderer is needed. */
export async function loadSkinRendering() {
	const [rendering, ears] = await Promise.all([
		import('./skin-rendering'),
		import('../../composables/skin-rendering/use-ears-mod-features'),
	])
	return { ...rendering, applyEarsMod: ears.applyEarsMod, removeEarsMod: ears.removeEarsMod }
}
