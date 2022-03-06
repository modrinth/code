/// <reference types="@sveltejs/kit" />
/// <reference types="unplugin-icons/types/svelte" />

declare module '$assets/images/*' {
    export { SvelteComponentDev as default } from 'svelte/internal';
}
declare module '$locales/*';

declare module '*.svg' {
    import { SvelteComponent } from 'svelte';
    const content: SvelteComponent;
    export default content;
}