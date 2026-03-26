<script lang="ts">
    import { formatLocaleDate } from "./timeFormatting.js";
    import { Chart } from "@flowbite-svelte-plugins/chart";
    import type { Session } from "./types.js";
    import type { ApexOptions } from "apexcharts";

    // Sessions prop from gameInfo page.
    let { sessions }: { sessions: Session[] } = $props();

    function getSessionRange(sessions: Session[]) {
        let tally: Record<string, number> = {};

        for (const session of sessions) {
            let dateStr = formatLocaleDate(session.startTs)

            if (tally[dateStr]) {
                tally[dateStr] += 1;
            } else {
                tally[dateStr] = 1;
            }
        }

        let dates = Object.keys(tally).slice(-7);
   
        let counts = Object.values(tally).slice(-7);

        return { dates, counts };

    }

    let chartData: {dates: string[], counts: number[]} = $derived(getSessionRange(sessions));

    let options = $derived<ApexOptions>({
        chart: {
            type: "bar",
            height: "300px",
        },
        series: [
            {
                name: "Sessions",
                data: chartData.counts,
            }
        ],
        xaxis: {
            categories: chartData.dates
        },
    });
</script>

<Chart {options} />
