<script lang="ts">
    import { onMount, tick } from 'svelte';
    import TechStatus from '$lib/TechStatus.svelte';
    import loadScript from '$lib/loadScript';
    import Licenses from '$lib/LICENSES.md';
    import wasm_script from '../../../pkg/asteroids_3d_lib.js?url';
    import wasm_mod from '../../../pkg/asteroids_3d_lib_bg.wasm?url';
    import bytesTotal from '../../../pkg/asteroids_3d_lib_bg.wasm?size';

    let bytesLoaded = 0;
    let errorMessage = '';

    function prettyBytes(bytes: number) {
        if (bytes < 1024 ** 2) {
            return Math.round(bytes / 1024) + ' KB';
        }
        return Math.round(bytes / 1024 ** 2) + ' MB';
    }

    async function load() {
        await loadScript(wasm_script);

        // https://stackoverflow.com/a/65529994/4471524
        async function load() {
            const response = await fetch(wasm_mod);

            let loaded = 0;

            function progressHandler(loaded: number) {
                bytesLoaded = loaded;
            }

            var res = new Response(
                new ReadableStream(
                    {
                        async start(controller) {
                            var reader = response.body.getReader();
                            for (;;) {
                                var { done, value } = await reader.read();

                                if (done) {
                                    progressHandler(bytesTotal);
                                    break;
                                }

                                loaded += value.byteLength;
                                progressHandler(loaded);
                                controller.enqueue(value);
                            }
                            controller.close();
                        },
                    },
                    {
                        status: response.status,
                        statusText: response.statusText,
                    }
                )
            );

            // Make sure to copy the headers!
            // Wasm is very picky with it's headers and it will fail to compile if they are not
            // specified correctly.
            for (const pair of response.headers.entries()) {
                res.headers.set(pair[0], pair[1]);
            }
            return res;
        }

        (await wasm_bindgen(load())).main();
    }

    let techStatus = {
        WebAssembly: undefined,
        'WebGL 2': undefined,
        PointerEvent: undefined,
    };
    $: techOk = (() => {
        if (Object.values(techStatus).every((v) => v)) return true;
        if (Object.values(techStatus).some((v) => v === undefined)) return undefined;
        return false;
    })();

    onMount(async () => {
        techStatus.WebAssembly =
            typeof window.WebAssembly?.instantiateStreaming === 'function' ||
            typeof window.WebAssembly?.instantiate === 'function';
        techStatus['WebGL 2'] = !!(document.getElementById(
            'can'
        ) as HTMLCanvasElement)?.getContext?.('webgl2');
        techStatus.PointerEvent = typeof PointerEvent === 'function';

        await tick();
        if (!techOk) return;

        load().catch((e) => {
            console.error(e);
            errorMessage = e.toString();
        });
    });
</script>

<svelte:head>
    <title>Home</title>
</svelte:head>

<section class="w-full h-screen overflow-hidden">
    <canvas class="w-0 h-0" id="can" />
    <div class="w-full h-full flex justify-center px-10 flex-col">
        {#if !techOk}
            <div class="container self-center items-center flex flex-col">
                <div class="text-4xl sm:text-6xl space-y-4">
                    <noscript>
                        <TechStatus status={false}>JavaScript</TechStatus>
                    </noscript>
                    {#each Object.entries(techStatus) as [k, v]}
                        <TechStatus status={v}>{k}</TechStatus>
                    {/each}
                </div>
                <noscript>
                    <p class="mt-4 text-2xl">You need to enable JavaScript to use this website.</p>
                </noscript>
                {#if techOk === false}
                    <p class="mt-4 text-2xl">
                        Your browser doesn't support the required technology to use this website.
                        Switch to another browser, such as Chrome, Edge, or Firefox.
                    </p>
                {/if}
            </div>
        {:else}
            <span class="text-2xl">
                {prettyBytes(bytesLoaded)} of {prettyBytes(bytesTotal)}
            </span>
            <div class="w-full mb-2 bg-blue-900 rounded-xl">
                <div
                    style={`width: ${(bytesLoaded / bytesTotal) * 100}%`}
                    class="bg-blue-500 py-5 rounded-xl"
                />
            </div>
            <p class="text-red-300">{errorMessage}</p>
        {/if}
    </div>
</section>

<section class="bg-gray-900">
    <div class="container mx-auto pt-8 pb-8">
        <h1 class="text-3xl">Asteroids_3d</h1>
        <p>
            <a href="https://github.com/lights0123/asteroids_3d">View source on GitHub</a>
        </p>
        <div class="md mt-8">
            <Licenses />
        </div>
    </div>
</section>
