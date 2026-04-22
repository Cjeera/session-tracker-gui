<script lang="ts">
    import { formatDate } from "$lib/timeFormatting";
    import { Timeline, TimelineItem } from "flowbite-svelte";
    import { fade } from "svelte/transition"; // <-- 1. Import fade

    let { sessions } = $props();
</script>

<div in:fade={{ duration: 75 }}> 
    <Timeline>
        {#each sessions as session}
            <TimelineItem
                class="text-gray-300"
                title="Notes:"
                date={formatDate(session.startTs)}
                dateFormat="full-date"
                >{session.notes || "No session notes"}
            </TimelineItem>
        {/each}
    </Timeline>
</div>