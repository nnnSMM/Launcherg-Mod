<script lang="ts">
  import emblaCarouselSvelte from "embla-carousel-svelte";
  import type { EmblaCarouselType, EmblaOptionsType } from "embla-carousel";

  export let options: EmblaOptionsType = {
    align: 'start',
    containScroll: 'trimSnaps',
    dragFree: true, // This provides the inertial effect
    loop: true,
  };

  let emblaApi: EmblaCarouselType;

  // Functions to control the carousel, to be called by the parent
  export const scrollPrev = () => emblaApi?.scrollPrev();
  export const scrollNext = () => emblaApi?.scrollNext();
  export const reInit = () => {
    if (!emblaApi) return;
    emblaApi.reInit();
  };

  const onInit = (event: CustomEvent<EmblaCarouselType>) => {
    emblaApi = event.detail;
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
  use:emblaCarouselSvelte={{ options, plugins: [] }}
  on:emblaInit={onInit}
>
  <div class="embla__container">
    <slot />
  </div>
</div>
