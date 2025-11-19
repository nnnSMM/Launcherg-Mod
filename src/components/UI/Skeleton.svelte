<script lang="ts">
    export let width: string = "100%";
    export let height: string = "1rem";
    export let className: string = "";
</script>

<div
    class="bg-bg-secondary rounded animate-pulse {className}"
    style="width: {width}; height: {height};"
/>

<style>
    /* Optional: Add a more complex shimmer effect if animate-pulse isn't enough */
</style>
