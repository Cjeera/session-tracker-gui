<script lang="ts">
    import { formatDate } from "#lib/timeFormatting.js";
    import type { Session } from "#lib/types.js";
    import { fade } from "svelte/transition";
    let { sessions }: { sessions: Session[] } = $props();
</script>

<div in:fade={{ duration: 75 }}>
    {#if sessions.length === 0}
        <p class="py-8 text-muted-foreground">No sessions recorded yet.</p>
    {:else}
        <ol class="ml-2 space-y-6 border-l pl-6">
            {#each sessions as session}
                <li class="relative">
                    <span class="absolute -left-[1.9rem] top-1.5 size-3 rounded-full border bg-primary" aria-hidden="true"></span>
                    <time datetime={session.startTs} class="text-sm text-muted-foreground">{formatDate(session.startTs)}</time>
                    <h3 class="mt-1 font-semibold">Notes</h3>
                    <p class="mt-2 whitespace-pre-wrap text-muted-foreground">{session.notes || "No session notes"}</p>
                </li>
            {/each}
        </ol>
    {/if}
</div>
