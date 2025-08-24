<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export const scrollBy = (options: ScrollToOptions) => {
    ref?.scrollBy(options);
  };

  export const scrollTo = (options: ScrollToOptions) => {
    ref?.scrollTo(options);
  };

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();
  const onScroll = (event: Event) => {
    dispatcher("scroll", { event });
  };

  let ref: HTMLElement;
</script>

<div class="overflow-x-auto" on:scroll={onScroll} bind:this={ref}>
  <slot />
</div>
