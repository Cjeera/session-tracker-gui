<script>
    import { formatDate } from "./timeFormatting";
    import { Timeline, TimelineItem} from "flowbite-svelte";

    // Sessions prop from gameInfo page.
    let { sessions } = $props();
</script>

<Timeline>

    <!--Displays the date and notes of each session for a specific game in a timeline-->
    {#each sessions as session}
        <TimelineItem class="text-gray-300" title="Notes:" date={formatDate(session.startTs)} dateFormat="full-date">{session.notes || "No session notes"}</TimelineItem>
    {/each}

</Timeline>

