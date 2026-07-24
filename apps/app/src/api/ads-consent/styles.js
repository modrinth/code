const THEME_STYLE = `
	:root {
		--modrinth-usp-bg: #27292e;
		--modrinth-usp-surface: #34363c;
		--modrinth-usp-divider: #34363c;
		--modrinth-usp-text: #b0bac5;
		--modrinth-usp-contrast: #ffffff;
		--modrinth-usp-brand: #1bd96a;
		--modrinth-usp-link: #4f9cff;
		--modrinth-usp-accent-contrast: #000000;
		--modrinth-usp-shadow: rgba(0, 0, 0, 0.1) 0 4px 6px -1px,
			rgba(0, 0, 0, 0.06) 0 2px 4px -1px;
		color-scheme: dark;
	}

	#qc-cmp2-usp {
		background: var(--modrinth-usp-bg) !important;
		border: 1px solid var(--modrinth-usp-divider) !important;
		border-radius: 1rem !important;
		box-shadow: var(--modrinth-usp-shadow) !important;
		color: var(--modrinth-usp-text) !important;
		font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Oxygen, Ubuntu, Roboto,
			Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif !important;
		max-width: 660px;
	}

	#qc-cmp2-usp .qc-usp-ui-content,
	#qc-cmp2-usp .qc-usp-ui-form-content,
	#qc-cmp2-usp .qc-usp-container {
		background: transparent !important;
	}

	#qc-cmp2-usp .qc-usp-container {
		margin-bottom: 12px;
	}

	#qc-cmp2-usp p,
	#qc-cmp2-usp label,
	#qc-cmp2-usp .qc-usp-action-description {
		color: var(--modrinth-usp-text) !important;
		font-family: inherit !important;
	}

	#qc-cmp2-usp .qc-usp-title,
	#qc-cmp2-usp .qc-cmp2-list-item-title {
		color: var(--modrinth-usp-contrast) !important;
		font-family: inherit !important;
		font-weight: 700 !important;
	}

	#qc-cmp2-usp .qc-usp-title {
		font-size: 1.25rem !important;
	}

	#qc-cmp2-usp a,
	#qc-cmp2-usp .qc-usp-alt-action {
		color: var(--modrinth-usp-link) !important;
	}

	#qc-cmp2-usp .qc-cmp2-list-item {
		background: var(--modrinth-usp-surface) !important;
		border: 1px solid var(--modrinth-usp-divider) !important;
		border-radius: 0.75rem !important;
	}

	#qc-cmp2-usp .qc-cmp2-list-item-header {
		background: transparent !important;
		border: 0 !important;
		color: var(--modrinth-usp-contrast) !important;
	}

	#qc-cmp2-usp .qc-cmp2-list-item-header svg {
		color: var(--modrinth-usp-text) !important;
	}

	#qc-cmp2-usp .qc-cmp2-toggle {
		background: var(--modrinth-usp-bg) !important;
		border-color: var(--modrinth-usp-divider) !important;
	}

	#qc-cmp2-usp .qc-cmp2-toggle .toggle {
		background: var(--modrinth-usp-contrast) !important;
	}

	#qc-cmp2-usp .qc-cmp2-toggle .text {
		color: var(--modrinth-usp-contrast) !important;
	}

	#qc-cmp2-usp .qc-cmp2-toggle[aria-checked='true'] {
		background: var(--modrinth-usp-brand) !important;
		border-color: var(--modrinth-usp-brand) !important;
	}

	#qc-cmp2-usp .qc-cmp2-toggle[aria-checked='true'] .text {
		color: var(--modrinth-usp-accent-contrast) !important;
	}

	#qc-cmp2-usp button[mode='primary'] {
		background: var(--modrinth-usp-brand) !important;
		border: 0 !important;
		border-radius: 0.75rem !important;
		color: var(--modrinth-usp-accent-contrast) !important;
		font-family: inherit !important;
		font-weight: 700 !important;
	}

	#qc-cmp2-usp .qc-usp-close-icon {
		border: 0 !important;
		background: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M18 6 6 18'/%3E%3Cpath d='m6 6 12 12'/%3E%3C/svg%3E")
			center / 1.5rem 1.5rem no-repeat;
	}

	#qc-cmp2-usp a:focus-visible,
	#qc-cmp2-usp button:focus-visible {
		outline: 2px solid var(--modrinth-usp-brand) !important;
		outline-offset: 2px !important;
	}

	#qc-cmp2-usp .qc-usp-ui-content {
		max-width: 100% !important;
	}

	#qc-cmp2-usp .qc-usp-ui-content .qc-usp-ui-form-content {
		border: 1px solid transparent !important;
		padding: 0 !important;
	}
`

const RAIL_STYLE = `
	html.modrinth-ads-consent-preferences #modrinth-rail-1 {
		display: none !important;
	}
`

const OVERLAY_STYLE = `
	html.modrinth-ads-consent-overlay:not(.modrinth-ads-consent-fallback):not(.modrinth-ads-consent-preferences) #qc-cmp2-main,
	html.modrinth-ads-consent-preferences:not(.modrinth-ads-consent-preferences-visible) #qc-cmp2-main {
		display: none !important;
	}

	#qc-cmp2-usp .qc-usp-close-icon {
		display: none !important;
	}
`

function installStyle(id, css) {
	if (document.getElementById(id)) return

	const style = document.createElement('style')
	style.id = id
	style.textContent = css
	document.documentElement.appendChild(style)
}

function installConsentStyles() {
	installStyle('modrinth-ads-consent-theme-style', THEME_STYLE)
	installStyle('modrinth-ads-rail-style', RAIL_STYLE)
	installStyle('modrinth-ads-consent-overlay-style', OVERLAY_STYLE)
}
