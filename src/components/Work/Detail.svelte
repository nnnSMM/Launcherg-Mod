<script lang="ts">
  import LinkText from "@/components/UI/LinkText.svelte";
  import {
    VoiceActorImportance,
    type VoiceActor,
    type Work,
  } from "@/lib/types";

  export let work: Work;

  const getCreatorUrl = (id: number) =>
    `https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/creater.php?creater=${id}`;
  const getVoiceActorClass = (importance: VoiceActor["importance"]) => {
    switch (importance) {
      case VoiceActorImportance.Main:
        return "text-text-primary font-bold";
      case VoiceActorImportance.Sub:
        return "text-text-secondary";
      case VoiceActorImportance.Mob:
        return "text-text-tertiary";
      default:
        const _: never = importance;
        break;
    }
  };
</script>

<div class="flex flex-col gap-6">
  <!-- シナリオ -->
  {#if work.creators.writers.length}
    <div class="min-w-0">
      <div class="text-[11px] tracking-widest uppercase text-text-tertiary mb-2.5 flex items-center gap-1.5">
        <div class="i-material-symbols-edit-document-outline-rounded w-4 h-4 opacity-80" />
        シナリオ
      </div>
      <div class="flex flex-wrap gap-2.5">
        {#each work.creators.writers as v (v.id)}
          <div class="inline-flex items-center bg-black/5 dark:bg-white/5 backdrop-blur-sm border border-black/10 dark:border-white/10 rounded px-1.5 py-0.5 transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/10 hover:shadow-md">
            <LinkText href={getCreatorUrl(v.id)}>
              <span class="text-[11px] font-medium drop-shadow-sm">{v.name}</span>
            </LinkText>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 原画 -->
  {#if work.creators.illustrators.length}
    <div class="min-w-0">
      <div class="text-[11px] tracking-widest uppercase text-text-tertiary mb-2.5 flex items-center gap-1.5">
        <div class="i-material-symbols-brush-outline-rounded w-4 h-4 opacity-80" />
        原画
      </div>
      <div class="flex flex-wrap gap-2.5">
        {#each work.creators.illustrators as v (v.id)}
          <div class="inline-flex items-center bg-black/5 dark:bg-white/5 backdrop-blur-sm border border-black/10 dark:border-white/10 rounded px-1.5 py-0.5 transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/10 hover:shadow-md">
            <LinkText href={getCreatorUrl(v.id)}>
              <span class="text-[11px] font-medium drop-shadow-sm">{v.name}</span>
            </LinkText>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 声優 -->
  {#if work.creators.voiceActors.length}
    <div class="min-w-0">
      <div class="text-[11px] tracking-widest uppercase text-text-tertiary mb-2.5 flex items-center gap-1.5">
        <div class="i-material-symbols-mic-outline-rounded w-4 h-4 opacity-80" />
        声優
      </div>
      <div class="flex flex-wrap gap-2.5">
        {#each work.creators.voiceActors as v (v.id)}
          <div class="inline-flex items-center gap-1.5 bg-black/5 dark:bg-white/5 backdrop-blur-sm border border-black/10 dark:border-white/10 rounded px-1.5 py-0.5 transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/10 hover:shadow-md">
            <LinkText href={getCreatorUrl(v.id)}>
              <span class="text-[11px] font-medium drop-shadow-sm">{v.name}</span>
            </LinkText>
            <span class="text-[10px] tracking-wide {getVoiceActorClass(v.importance)} opacity-60 uppercase">{v.role}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 楽曲 -->
  {#if work.musics.length}
    <div class="min-w-0">
      <div class="text-[11px] tracking-widest uppercase text-text-tertiary mb-2.5 flex items-center gap-1.5">
        <div class="i-material-symbols-music-note-rounded w-4 h-4 opacity-80" />
        楽曲
      </div>
      <div class="flex flex-wrap gap-2.5">
        {#each work.musics as title, i (`${title}-${i}`)}
          <div class="inline-flex items-center gap-1.5 bg-black/5 dark:bg-white/5 backdrop-blur-sm border border-black/10 dark:border-white/10 rounded px-1.5 py-0.5 transition-all hover:-translate-y-0.5 hover:bg-black/10 dark:hover:bg-white/10 hover:shadow-md max-w-full">
            <div class="i-iconoir-youtube w-3 h-3 color-#cc0000 shrink-0 drop-shadow-sm" />
            <div class="truncate">
              <LinkText
                href={encodeURI(
                  `https://www.youtube.com/results?search_query=${work.name}+${title}`,
                )}
              >
                <span class="text-[11px] font-medium drop-shadow-sm">{title}</span>
              </LinkText>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
