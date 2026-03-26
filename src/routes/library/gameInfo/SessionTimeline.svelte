<script>
    import { formatDate } from "./timeFormatting";
    import { Timeline, TimelineItem} from "flowbite-svelte";

    // Sessions prop from gameInfo page.
    let { sessions } = $props();
</script>

{#if sessions.length > 0}
    <Timeline>
        <!--Displays the date and notes of each session for a specific game in a timeline-->
        {#each sessions as session}
            <TimelineItem class="text-gray-300" title="Notes:" date={formatDate(session.startTs)} dateFormat="full-date">{session.notes || "No session notes"}</TimelineItem>
        {/each}
    </Timeline>
{:else}
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">No Sessions Recorded Yet!</h5>
{/if}