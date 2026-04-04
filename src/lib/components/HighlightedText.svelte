<script lang="ts">
  interface Props {
    text: string;
    query: string;
  }

  let { text, query }: Props = $props();

  function getHighlightedParts(text: string, query: string) {
    if (!query) return [{ text, highlight: false }];

    const parts: { text: string; highlight: boolean }[] = [];
    let lastIndex = 0;
    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();

    let index = lowerText.indexOf(lowerQuery);
    while (index !== -1) {
      if (index > lastIndex) {
        parts.push({ text: text.slice(lastIndex, index), highlight: false });
      }
      parts.push({ text: text.slice(index, index + query.length), highlight: true });
      lastIndex = index + query.length;
      index = lowerText.indexOf(lowerQuery, lastIndex);
    }

    if (lastIndex < text.length) {
      parts.push({ text: text.slice(lastIndex), highlight: false });
    }

    return parts;
  }

  let parts = $derived(getHighlightedParts(text, query));
</script>

<span class="highlighted-text">
  {#each parts as part}
    {#if part.highlight}
      <mark class="highlight">{part.text}</mark>
    {:else}
      {part.text}
    {/if}
  {/each}
</span>

<style>
  .highlighted-text {
    word-break: break-all;
  }

  .highlight {
    background: var(--accent-color);
    color: white;
    border-radius: 2px;
    padding: 0 2px;
  }
</style>
