<template>
	<div
		id="rule-6"
		class="rounded-2xl border border-solid border-surface-4 bg-surface-3 markdown-body p-4"
	>
		<nuxt-link to="/legal/rules#generative-ai" target="_blank">
			<h2 class="!mt-0">6. Usage of Generative "AI"</h2>
		</nuxt-link>

		<p>
			Projects must be forthright and honest about the usage of generative AI in their production,
			publication, and within the project itself. Projects cannot be entirely or primarily comprised
			of content created or derived from generative AI output.
		</p>

		<h3 id="disclosure-of-ai-generated-content">6.1. Disclosure of AI generated content</h3>

		<p>You must apply the appropriate “Contains AI-generated content” content disclosure when:</p>
		<ol type="a">
			<li>a substantial portion of the project's code is a product of AI output.</li>
			<li>
				the project includes any assets that are primarily or entirely a product of AI output.
			</li>
			<li>the project's design or functionality relies on the use of generative AI.</li>
			<li>
				any element of the project's page such as description or publishing relies on generative AI.
			</li>
		</ol>

		<h3 id="prohibited-usage-of-ai">6.2. Prohibited AI generated content</h3>

		<ol type="a">
			<li>
				No images uploaded to a gallery, icon, description, or any other part of a project page may
				be created or derived from generative AI output. Any such images may be removed.
			</li>
			<li>
				Projects may not be published publicly if the contents are primarily or entirely a product
				of AI output.
			</li>
		</ol>
	</div>
</template>
<style scoped>
.markdown-body h2 {
	@apply mb-3 mt-5 pb-1 pt-0 text-xl font-semibold text-contrast;
}
.markdown-body h3 {
	@apply mb-1 mt-5 p-0 text-lg font-semibold text-contrast;
}

.markdown-body a {
	@apply !font-semibold !text-brand !no-underline hover:!text-brand hover:!underline;
}

.markdown-body p {
	@apply leading-normal;
}

.markdown-body ol {
	@apply mb-3;
}

.markdown-body ol ol {
	@apply mb-2;
}

.markdown-body li {
	@apply mb-1;
}
</style>
