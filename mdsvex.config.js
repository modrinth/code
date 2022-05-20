import { defineMDSveXConfig as defineConfig } from 'mdsvex';
import examples from 'mdsvexamples';

const config = defineConfig({
    extensions: ['.svelte.md', '.md', '.svx'],

    smartypants: {
        dashes: 'oldschool',
    },

    remarkPlugins: [
        [
            examples,
            {
                defaults: {
                    Wrapper: '$routes/_internal/components/Example.svelte',
                },
            },
        ],
    ],
    rehypePlugins: [],

    layout: {
        _: './src/routes/_internal/layout/page.svelte',
    },
});

export default config;
