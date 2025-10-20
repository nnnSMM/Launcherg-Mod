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
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    transform={back ? 'rotate(180)' : ''}
  >
    <path
      d="M9.70492 6.29498L15.4149 12L9.70492 17.705L8.29492 16.295L12.5849 12L8.29492 7.70498L9.70492 6.29498Z"
      fill="currentColor"
    />
  </svg>
</button>
