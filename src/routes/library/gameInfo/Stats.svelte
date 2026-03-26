<script lang="ts">
    import { formatDuration, formatLocaleDate, formatTime } from "./timeFormatting.js";
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

    function getLongestSessions(sessions: Session[]) {
        let sortedSessions = [...sessions].sort((a, b) => b.durationSeconds - a.durationSeconds);
        let topFive = sortedSessions.slice(0, 5);

        let dates = topFive.map(session => formatLocaleDate(session.startTs));
        let durationsSeconds = topFive.map(session => session.durationSeconds);
        
        // This was a great idea! Let's map it cleanly and return it.
        let durationsString = durationsSeconds.map(sec => formatDuration(sec));

        // Return all three arrays
        return { dates, durationsSeconds, durationsString };
    }

    function getMostPlayedDays(sessions: Session[]) {

        let daysString = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        
        let ledger: Record<string, number> = {
            "Sunday": 0,
            "Monday": 0,
            "Tuesday": 0,
            "Wednesday": 0,
            "Thursday": 0,
            "Friday": 0,
            "Saturday": 0,
        };

        for (const session of sessions) {
            let day = new Date(session.startTs).getDay();
            ledger[daysString[day]] += session.durationSeconds;    
        }

        let days = Object.keys(ledger);
        let hours = Object.values(ledger).map(seconds => Math.round((seconds / 3600) * 10) / 10);

        return {days, hours};
    }

    let recentSessions = $derived(getSessionRange(sessions));

    let longestSessions = $derived(getLongestSessions(sessions));

    let mostPlayedDays = $derived(getMostPlayedDays(sessions));

    let recentOptions = $derived<ApexOptions>({
        chart: {
            type: "bar",
            height: "300px",
        },
        series: [
            {
                name: "Sessions",
                data: recentSessions.counts,
            }
        ],
        xaxis: {
            categories: recentSessions.dates
        },
    });

    let longestOptions = $derived<ApexOptions>({
        chart: {
            type: "bar",
            height: "300px",
        },
        plotOptions: {
            bar: {
                horizontal: true,
            }
        },
        series: [
            {
                name: "Playtime",
                data: longestSessions.durationsSeconds, // Numbers go to the data
            }
        ],
        xaxis: {
            categories: longestSessions.dates, // Strings go to the categories
            labels: {
                // Let's hide the raw numbers on the bottom axis to keep it clean
                show: false 
            }
        },
        // Replace the raw numbers with the HH:MM:SS strings on hover/labels
        dataLabels: {
            enabled: true,
            formatter: function (_value, { dataPointIndex }) {
                return longestSessions.durationsString[dataPointIndex];
            }
        },
        tooltip: {
            y: {
                formatter: function (_value, { dataPointIndex }) {
                    return longestSessions.durationsString[dataPointIndex];
                }
            }
        }
    });

    let mostPlayedDaysOptions = $derived<ApexOptions>({
        chart: {
            type: "pie",
            height: "300px",
        },
        labels: mostPlayedDays.days,

        series: mostPlayedDays.hours,
        
        yaxis: {
            labels: {
                formatter: function (value) {
                    return value + " Hours";
                }
            }
        }
    });
</script>

{#if sessions.length > 0}
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Last 7 Sessions</h5>
    <Chart options={recentOptions} />
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Longest 5 Sessions</h5>
    <Chart options={longestOptions} />
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Most Common Days Played</h5>
    <Chart options={mostPlayedDaysOptions} />
{:else}
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">No Sessions Recorded Yet!</h5>
{/if}