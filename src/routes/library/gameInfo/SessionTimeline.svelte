<script>
    import { formatDate, formatTime, formatDuration} from "./timeFormatting.js";
    import { Timeline, TimelineItem} from "flowbite-svelte";

    let { sessions } = $props();
</script>

<Timeline>

    {#each sessions as session}
        <TimelineItem class="text-gray-300" title="Notes:" date={formatDate(session.startTs)}>{session.notes || "No session notes"}</TimelineItem>
    {/each}

</Timeline>

