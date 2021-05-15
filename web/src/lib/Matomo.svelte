<!-- from https://gitlab.com/dexterlabs/dexlib/-/blob/master/packages/svelte-matomo/src/index.svelte -->
<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    const delayedCalls = writable([]);

    export const matomo: Record<string, (...args: any) => void> = new Proxy(
        {},
        {
            get: (_, method) => (...args: any) =>
                delayedCalls.update((calls) => [...calls, [method, args]]),
        }
    );
</script>

<script lang="ts">
    import { onMount } from 'svelte';

    export let url: string;
    export let siteId: number | string;
    export let debug = false;

    export let cookies = true;
    export let consentRequired = false;
    export let doNotTrack = false;
    export let heartBeat = 2000;

    let _matomo;
    let loaded = false;

    onMount(() => (loaded = true));

    $: scriptUrl = `${url}/matomo.js`;
    $: trackUrl = `${url}/matomo.php`;
    $: tracker = _matomo && _matomo.getTracker(trackUrl, siteId);

    $: if (tracker && !cookies) tracker.disableCookies();
    $: if (tracker && consentRequired) tracker.requireConsent();
    $: if (tracker && doNotTrack) tracker.setDoNotTrack();
    $: if (tracker && heartBeat) tracker.enableHeartBeatTimer(heartBeat);

    $: while (tracker && $delayedCalls.length) {
        const [fnName, args] = $delayedCalls.shift();
        if (tracker[fnName] instanceof Function) {
            if (debug) console.log('Calling', fnName, args);
            tracker[fnName](...args);
        } else {
            console.error('Trying to call nonexistent function', fnName);
        }
    }
</script>

<svelte:head>
    {#if loaded}
        <script defer async src={scriptUrl} on:load={() => (_matomo = window.Matomo)} />
    {/if}
</svelte:head>
