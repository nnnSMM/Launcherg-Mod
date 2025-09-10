<script lang="ts">
  import { createEventDispatcher, onMount, tick } from "svelte";

  export let label = "";
  export let value: string;
  export let placeholder: string = "";
  export let autofocus = false;
  export let isShortcut = false;

  let isRecording = false;

  const dispatcher = createEventDispatcher<{ update: { value: string } }>();

  let input: HTMLInputElement | null = null;

  onMount(async () => {
    if (!autofocus) {
      return;
    }
    await tick();
    input?.focus();
  });

  function handleFocus() {
    if (isShortcut) {
      isRecording = true;
      value = "記録中...";
    }
  }

  function handleBlur() {
    if (isShortcut) {
      isRecording = false;
      if (value === "記録中...") {
        value = "";
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isShortcut || !isRecording) {
      return;
    }

    event.preventDefault();

    let parts: string[] = [];
    if (event.ctrlKey) {
      parts.push("Control");
    }
    if (event.altKey) {
      parts.push("Alt");
    }
    if (event.shiftKey) {
      parts.push("Shift");
    }
    if (event.metaKey) {
      parts.push("Command");
    }

    const key = event.key.toUpperCase();
    if (!["CONTROL", "ALT", "SHIFT", "COMMAND"].includes(key)) {
      parts.push(key);
    }

    value = parts.join("+");
    dispatcher("update", { value });
    isRecording = false;
    input?.blur();
  }
</script>

<label>
  {#if label}
    <div class="text-(text-primary body) font-medium mb-1">{label}</div>
  {/if}
  <div
    class="w-full border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all"
  >
    <input
      bind:this={input}
      bind:value
      type="text"
      on:input={(e) => {
        if (!isShortcut) {
          dispatcher("update", { value: e.currentTarget.value });
        }
      }}
      on:focus={handleFocus}
      on:blur={handleBlur}
      on:keydown={handleKeydown}
      readonly={isShortcut}
      {placeholder}
      class="w-full border border-(border-primary solid) rounded bg-bg-primary p-(x-3 y-1) text-(input text-primary) transition-all focus:border-transparent placeholder-ui-tertiary"
    />
  </div>
</label>
