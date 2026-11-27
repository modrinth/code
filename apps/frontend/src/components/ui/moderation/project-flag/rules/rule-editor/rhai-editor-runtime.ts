export async function loadRhaiEditor() {
	await import('@modrinth/ui/src/utils/ace-theme')
	const [{ VAceEditor }, { RHAI_EDITOR_OPTIONS }] = await Promise.all([
		import('vue3-ace-editor'),
		import('../ace-rhai'),
		import('ace-builds/src-noconflict/ext-language_tools'),
	])
	return { component: VAceEditor, options: RHAI_EDITOR_OPTIONS }
}

export type RhaiEditorRuntime = Awaited<ReturnType<typeof loadRhaiEditor>>
