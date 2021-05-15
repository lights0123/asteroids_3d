<script lang="ts">
    import { page } from '$app/stores';
    import Matomo, { matomo } from '$lib/Matomo.svelte';
    import '../app.css';

    const url = import.meta.env.VITE_MATOMO_HOST.toString();
    const siteId = import.meta.env.VITE_MATOMO_SITE_ID.toString();

    $: if ($page) matomo.trackPageView();

    matomo.enableLinkTracking();
</script>

<main>
    <slot />
</main>
{#if url && siteId}
    <Matomo debug {url} {siteId} />
{/if}
