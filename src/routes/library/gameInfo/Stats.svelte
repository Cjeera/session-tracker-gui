<script>
    import {
        formatDate,
        formatLocaleDate,
        formatTime,
        formatDuration,
    } from "./timeFormatting.js";
    import { Chart } from "@flowbite-svelte-plugins/chart";

    let { sessions } = $props();

    function getSessionRange(sessions) {
        let tally = {};

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

        return {dates, counts};

    }

    let chartData = $derived(getSessionRange(sessions));

    /** @type {import('apexcharts').ApexOptions} */
    let options = $derived({
        chart: {
            type: "bar",
            height: "300px",
        },
        series: [
            {
                name: "Sessions",
                data: chartData.counts,
            },
        ],
        xaxis: {
            categories: chartData.dates,
        },
    });
</script>

<Chart {options} />