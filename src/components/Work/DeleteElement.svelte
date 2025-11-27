<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandDeleteCollectionElement,
    commandDeleteCollectionElementLogical,
  } from "@/lib/command";
  import type { CollectionElement } from "@/lib/types";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { deleteTab, tabs, selected } from "@/store/tabs";

  export let isOpen: boolean;
  export let element: CollectionElement;

  const physicalDelete = async () => {
    await commandDeleteCollectionElement(element.id);
    await sidebarCollectionElements.refetch();
    deleteTab($tabs[$selected].id);
    isOpen = false;
  };

  const logicalDelete = async () => {
    await commandDeleteCollectionElementLogical(element.id);
    await sidebarCollectionElements.refetch();
    // Do not close tab, just refresh? Or maybe close tab if it's confusing.
    // If we keep tab, we should see "Set Path" button.
    // Let's keep the tab open so user sees the change.
    // But we need to make sure the UI updates.
    // sidebarCollectionElements.refetch() updates the store, which updates `element` in Actions.svelte.
    isOpen = false;
  };

  let isConfirmingPhysicalDelete = false;

  const handlePhysicalDeleteClick = () => {
    isConfirmingPhysicalDelete = true;
  };
</script>

<Modal
  {isOpen}
  on:close={() => {
    isOpen = false;
    isConfirmingPhysicalDelete = false;
  }}
  on:cancel={() => {
    isOpen = false;
    isConfirmingPhysicalDelete = false;
  }}
  title={isConfirmingPhysicalDelete ? "本当に削除しますか？" : "Delete game"}
  withContentPadding={false}
  autofocusCloseButton
  headerClass="border-b-(border-warning opacity-40) "
>
  <div
    class="bg-bg-warning border-(b-1px solid border-warning opacity-40) flex gap-2 p-(x-4 y-5)"
  >
    <div
      class="w-6 h-6 i-material-symbols-warning-outline-rounded color-accent-warning"
    />
    <div class="space-y-1">
      {#if isConfirmingPhysicalDelete}
        <div class="text-(body text-primary) font-medium">
          この操作は取り消せません
        </div>
        <div class="text-(body2 text-primary)">
          プレイ履歴、スクリーンショット、設定など、このゲームに関する全てのデータが完全に削除されます。本当によろしいですか？
        </div>
      {:else}
        <div class="text-(body text-primary) font-medium">
          このゲームの削除方法を選択してください
        </div>
        <div class="text-(body2 text-primary)">
          <ul class="list-disc pl-4 space-y-1">
            <li>
              <b>未インストール状態にする</b>:
              起動パスの設定をクリアします。次回プレイするには再度パスの設定が必要です。記録やゲームファイルはそのまま残ります。
            </li>
            <li>
              <b>完全削除</b>:
              ライブラリから完全に削除されます。プレイ履歴も削除されます。
            </li>
          </ul>
        </div>
      {/if}
    </div>
  </div>
  <div class="p-4 max-w-full flex flex-col gap-3" slot="footer">
    {#if isConfirmingPhysicalDelete}
      <Button
        text="はい、完全に削除します"
        variant="error"
        wrappable
        appendClass="w-full justify-center"
        on:click={physicalDelete}
      />
      <Button
        text="キャンセル"
        variant="normal"
        wrappable
        appendClass="w-full justify-center"
        on:click={() => (isConfirmingPhysicalDelete = false)}
      />
    {:else}
      <Button
        text="未インストール状態にする"
        variant="warning"
        wrappable
        appendClass="w-full justify-center"
        on:click={logicalDelete}
      />
      <Button
        text="完全削除"
        variant="error"
        wrappable
        appendClass="w-full justify-center"
        on:click={handlePhysicalDeleteClick}
      />
    {/if}
  </div>
</Modal>
