/// <reference types="@sveltejs/kit" />
/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module '*?size' {
    const size: number;
    export default size;
}

declare function wasm_bindgen(data: any): void;