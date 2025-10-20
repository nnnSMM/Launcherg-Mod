<script lang="ts">
  export let back: boolean = false;
  export let disabled: boolean = false;
</script>

<button
  class="w-12 h-12 rounded-full bg-white/80 backdrop-blur-sm shadow-md flex items-center justify-center transition-opacity disabled:opacity-30 hover:bg-white"
  on:click|stopPropagation
  {disabled}
>
  <svg
    class="w-6 h-6"
    viewBox="0 0 532 532"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    transform={back ? 'rotate(180)' : ''}
  >
    <path
      fill="currentColor"
      d="M176.34 520.646c-13.793 13.805-36.208 13.805-50.001 0-13.785-13.804-13.785-36.238 0-50.034L330.78 266 126.34 61.391c-13.785-13.805-13.785-36.239 0-50.044 13.793-13.796 36.208-13.796 50.002 0 22.928 22.947 206.395 206.507 229.332 229.454a35.065 35.065 0 0 1 10.326 25.126c0 9.2-3.393 18.26-10.326 25.2-45.865 45.901-206.404 206.564-229.332 229.52Z"
    ></path>
  </svg>
</button>
