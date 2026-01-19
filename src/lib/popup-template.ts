import Handlebars from 'handlebars';

/**
 * Render a markdown template with Handlebars.
 *
 * Only returns a rendered string when `jsonText` is valid JSON.
 */
export function renderPopupTemplate(template: string, jsonText: string): string | null {
  const input = jsonText.trim();
  if (!input) {
    return null;
  }

  let data: unknown;
  try {
    data = JSON.parse(input);
  } catch {
    return null;
  }

  try {
    return Handlebars.compile(template)(data);
  } catch (error) {
    console.error(`Failed to render popup template: ${error}`);
    return null;
  }
}

