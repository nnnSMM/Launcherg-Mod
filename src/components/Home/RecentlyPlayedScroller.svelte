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
  export const reInit = () => {
    if (!emblaApi) return;
    emblaApi.reInit();
    onUpdate(emblaApi);
  };

  // Update button states when the carousel state changes
  const onUpdate = (api: EmblaCarouselType) => {
    if (!api) return;
    canScrollPrev = api.canScrollPrev();
    canScrollNext = api.canScrollNext();
  }

  const onInit = (event: CustomEvent<EmblaCarouselType>) => {
    emblaApi = event.detail;
    onUpdate(emblaApi);
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
  on:emblaInit={onInit}
  on:emblaSelect={() => onUpdate(emblaApi)}
  on:emblaReInit={() => onUpdate(emblaApi)}
>
  <div class="embla__container">
    <slot />
  </div>
</div>
