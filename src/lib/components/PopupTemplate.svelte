<script lang="ts" module>
  import { buildFormSchema } from '$lib/constraint';
  import { m } from '$lib/paraglide/messages';
  import type { PopupTemplate } from '$lib/types';
  import { markdown } from '@codemirror/lang-markdown';

  // form schema
  const schema = buildFormSchema(({ text }) => ({
    name: text().maxlength(64)
  }));

  const TEMPLATE_PLACEHOLDER = `
${m.popup_template_tip()}

{{user.name}}
`.trimStart();
</script>

<script lang="ts">
  import { enhance } from '$app/forms';
  import { CodeMirror, Label, Modal, alert } from '$lib/components';
  import { updatePopupTemplateId } from '$lib/shortcut';
  import { Loading } from '$lib/states.svelte';

  const { templates }: { templates: PopupTemplate[] } = $props();
  const loading = new Loading();

  let templateId: string = $state('');
  let templateName: string = $state('');
  let templateText: string = $state('');

  // fill form fields
  const fillForm = (template: PopupTemplate) => {
    templateName = template.id;
    templateText = template.template;
  };

  // show modal dialog
  let modal: Modal;
  export const showModal = (id?: string) => {
    if (id) {
      const template = templates.find((t) => t.id === id);
      if (!template) {
        return;
      }
      templateId = id;
      fillForm(template);
    }
    modal.show();
  };

  /**
   * Save popup template to persistent storage.
   *
   * @param form - form element
   */
  function save(form: HTMLFormElement) {
    // validate inputs
    templateName = templateName.trim();
    let template = templates.find((t) => t.id === templateName);
    if (template && template.id !== templateId) {
      alert({ level: 'error', message: m.name_already_used() });
      const nameInput = form.querySelector('input[name="name"]');
      (nameInput as HTMLInputElement | null)?.focus();
      return;
    }
    if (!templateText || templateText.trim().length === 0) {
      alert({ level: 'error', message: m.popup_template_content_empty() });
      return;
    }

    // start saving
    loading.start();
    template = templates.find((t) => t.id === templateId);
    if (template) {
      // update template
      if (template.id !== templateName) {
        template.id = templateName;
        updatePopupTemplateId(templateId, templateName);
      }
      template.template = templateText;
      alert(m.popup_template_updated_success());
    } else {
      // add new template
      templates.push({
        id: templateName,
        template: templateText
      });
      // reset form
      templateName = '';
      templateText = '';
      alert(m.popup_template_added_success());
    }
    modal.close();
    loading.end();
  }
</script>

<Modal title="{templateId ? m.update() : m.add()}{m.popup_template()}" bind:this={modal}>
  <form
    method="post"
    use:enhance={({ formElement, cancel }) => {
      cancel();
      save(formElement);
    }}
  >
    <fieldset class="fieldset">
      <Label required>{m.type_name()}</Label>
      <input class="autofocus input input-sm w-full" {...schema.name} bind:value={templateName} />

      <Label required tip={m.popup_template_tip()}>{m.popup_template()}</Label>
      <CodeMirror title={m.popup_template()} language={markdown()} placeholder={TEMPLATE_PLACEHOLDER} bind:document={templateText} />
    </fieldset>
    <div class="modal-action">
      <button type="button" class="btn" onclick={() => modal.close()}>{m.cancel()}</button>
      <button type="submit" class="btn btn-submit" disabled={loading.started}>
        {m.confirm()}
        {#if loading.delayed}
          <span class="loading loading-xs loading-dots"></span>
        {/if}
      </button>
    </div>
  </form>
</Modal>

