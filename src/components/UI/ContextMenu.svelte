<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";

  export let x: number;
  export let y: number;
  export let options: { label: string; onSelect: () => void }[];
  export let disableHover = false;

  const dispatch = createEventDispatcher();

  const closeMenu = () => dispatch("close");

  onMount(() => {
    window.addEventListener("click", closeMenu);
    window.addEventListener("contextmenu", closeMenu, { capture: true });
  });

  onDestroy(() => {
    window.removeEventListener("click", closeMenu);
    window.removeEventListener("contextmenu", closeMenu, { capture: true });
  });
</script>

<div
  role="menu"
  tabindex="-1"
  class="glass-menu-surface fixed z-50 rounded-md py-1"
  style="left: {x}px; top: {y}px;"
  on:click|stopPropagation
  on:contextmenu|stopPropagation
  on:keydown={(e) => {
    if (e.key === 'Escape') {
      closeMenu();
    }
  }}
>
  <ul>
    {#each options as option}
      <li>
        <button
          class="w-full text-left px-4 py-1.5 text-sm text-text-primary bg-transparent transition-colors {disableHover
            ? ''
            : 'hover:bg-accent-primary/16'}"
          on:click={() => {
            option.onSelect();
            closeMenu();
          }}
        >
          {option.label}
        </button>
      </li>
    {/each}
  </ul>
</div>
