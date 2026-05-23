<script lang="ts">
  import APopover from "./APopover.svelte";
</script>

<APopover>
  <button slot="button" id="trigger-btn">Open Popover</button>
  <div id="popover-content">Content</div>
</APopover>
