/// <reference types="@sveltejs/kit" />
/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module '*?size' {
    const size: number;
    export default size;
}
declare module '*.md' {
    import { SvelteComponentTyped } from 'svelte';
    export default class Md extends SvelteComponentTyped<{}, {}, {}> {}
}

declare function wasm_bindgen(data: any): void;
