<script lang="ts">
    import X from 'feather-icons/dist/icons/x-circle.svg';
    import Check from 'feather-icons/dist/icons/check-circle.svg';
    import Question from 'feather-icons/dist/icons/help-circle.svg';
    export let status: boolean | undefined;
</script>

<div class="flex items-center">
    <div class="mr-4">
        {#if status === true}<Check width="1em" height="1em" class="text-green-500" />
        {:else if status === false}<X width="1em" height="1em" class="text-red-500" />
        {:else}<Question width="1em" height="1em" class="text-gray-300" />
        {/if}
    </div>
    <slot />
</div>
