<script lang="ts">
  import emblaCarouselSvelte from "embla-carousel-svelte";
  import type { EmblaCarouselType, EmblaOptionsType } from "embla-carousel-svelte";

  export let options: EmblaOptionsType = {
    align: 'start',
    containScroll: 'trimSnaps',
    dragFree: true, // This provides the inertial effect
  };

  let emblaApi: EmblaCarouselType;

  // State for arrow buttons, to be bound by the parent
  export let canScrollPrev = false;
  export let canScrollNext = true;

  // Functions to control the carousel, to be called by the parent
  export const scrollPrev = () => emblaApi?.scrollPrev();
  export const scrollNext = () => emblaApi?.scrollNext();

  // Update button states when the carousel state changes
  function onSelect(api: EmblaCarouselType) {
    canScrollPrev = api.canScrollPrev();
    canScrollNext = api.canScrollNext();
  }
</script>

<style>
  .embla {
    overflow: hidden;
    cursor: grab;
  }
  .embla:active {
    cursor: grabbing;
  }
  .embla__container {
    display: flex;
  }
</style>

<div
  class="embla"
  use:emblaCarouselSvelte={{ options }}
  bind:emblaApi
  on:emblaSelect={() => onSelect(emblaApi)}
  on:emblaInit={() => onSelect(emblaApi)}
  on:emblaReInit={() => onSelect(emblaApi)}
>
  <div class="embla__container">
    <slot />
  </div>
</div>
