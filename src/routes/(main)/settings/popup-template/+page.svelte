<script lang="ts">
  import { Button, List, PopupTemplate as PopupTemplateModal, Setting } from '$lib/components';
  import { m } from '$lib/paraglide/messages';
  import { popupTemplates } from '$lib/stores.svelte';
  import { AppWindow, PencilSimpleLine, Sparkle } from 'phosphor-svelte';

  let creator: PopupTemplateModal;
  let updater: PopupTemplateModal;
</script>

<Setting icon={AppWindow} title={m.popup_template()} class="min-h-(--app-h)">
  <List
    icon={Sparkle}
    title={m.popup_template_count({ count: popupTemplates.current.length })}
    name={m.popup_template()}
    hint={m.popup_template_hint()}
    bind:data={popupTemplates.current}
    oncreate={() => creator.showModal()}
  >
    {#snippet row(item)}
      {@const firstLine = (item.template || '').split(/\r?\n/)[0]?.trim()}
      <div class="list-col-grow flex items-center gap-4 truncate" title={item.id}>
        <span class="min-w-8 truncate text-base font-light">{item.id}</span>
        {#if firstLine}
          <span class="truncate text-xs opacity-50">{firstLine}</span>
        {/if}
      </div>
      <Button
        icon={PencilSimpleLine}
        onclick={(event) => {
          event.stopPropagation();
          updater.showModal(item.id);
        }}
      />
    {/snippet}
  </List>
</Setting>

<PopupTemplateModal bind:this={creator} templates={popupTemplates.current} />
<PopupTemplateModal bind:this={updater} templates={popupTemplates.current} />
