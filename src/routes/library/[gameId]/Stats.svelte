<script lang="ts">
    import { formatDuration, formatLocaleDate, formatTime } from "$lib/timeFormatting.js";
    import { Chart } from "@flowbite-svelte-plugins/chart";
    import type { Session } from "$lib/types";
    import type { ApexOptions } from "apexcharts";

    // Sessions prop from gameInfo page.
    let { sessions }: { sessions: Session[] } = $props();

    /** Function for getting the last 7 sessions*/
    function getSessionRange(sessions: Session[]) {
        let tally: Record<string, number> = {};

        for (const session of sessions) {
            let dateStr = formatLocaleDate(session.startTs)
            
            // If more than one session is recorded on a single day, tally increments for the specific day.
            if (tally[dateStr]) {
                tally[dateStr] += 1;
            } else {
                tally[dateStr] = 1;
            }
        }

        // Both arrays are reduced to seven entries.
        let dates = Object.keys(tally).slice(-7);
        let counts = Object.values(tally).slice(-7);

        // Both arrays are returned as an object.
        return { dates, counts };
    }

    /** Function for getting the top 5 longest sessions by game*/
    function getLongestSessions(sessions: Session[]) {

        // Sessions are sorted by duration descending.
        let sortedSessions = [...sessions].sort((a, b) => b.durationSeconds - a.durationSeconds);

        // The top five longest sessions are obtained.
        let topFive = sortedSessions.slice(0, 5);

        // The dates of the longest sessions are obtained.
        let dates = topFive.map(session => formatLocaleDate(session.startTs));

        // The durations of the longest sessions are obtained.
        let durationsSeconds = topFive.map(session => session.durationSeconds);
        
        // The durations are formatted into HH:MM:SS
        let durationsString = durationsSeconds.map(sec => formatDuration(sec));

        // Return all three arrays
        return { dates, durationsSeconds, durationsString };
    }

    /** Function for getting the most played days for a specific game measured by hours played*/
    function getMostPlayedDays(sessions: Session[]) {
        let daysString = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        
        // A record for storing the days of the week and the total hours played on those days.
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

            // Gets the day of the week from the start timestamp.
            let day = new Date(session.startTs).getDay();

            // Adds duration seconds onto the record item corresponding to the day of the week of the timestamp.
            ledger[daysString[day]] += session.durationSeconds;    
        }

        // Extracts the days from the record.
        let days = Object.keys(ledger);

        // Extracts the durations from the record and formats them to hours.
        let hours = Object.values(ledger).map(seconds => Math.round((seconds / 3600) * 10) / 10);

        // Returns days and hours as an object.
        return {days, hours};
    }

    let recentSessions = $derived(getSessionRange(sessions));

    let longestSessions = $derived(getLongestSessions(sessions));

    let mostPlayedDays = $derived(getMostPlayedDays(sessions));

    // The options for the last 7 sessions chart.
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

    // The options for the top 5 longest sessions chart.
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
                data: longestSessions.durationsSeconds,
            }
        ],
        xaxis: {
            categories: longestSessions.dates,
            labels: {
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

    // Options for the most played days chart.
    let mostPlayedDaysOptions = $derived<ApexOptions>({
        chart: {
            type: "pie",
            height: "300px",
        },
        labels: mostPlayedDays.days,

        series: mostPlayedDays.hours,
        
        // Appends 'Hours' onto the hour counts for clarificaton.
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

    <!--Last 7 sessions chart-->
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Last 7 Sessions</h5>
    <Chart options={recentOptions} />

    <!--Longest 5 sessions chart-->
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Longest 5 Sessions</h5>
    <Chart options={longestOptions} />

    <!--Most common days played chart-->
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">Most Common Days Played</h5>
    <Chart options={mostPlayedDaysOptions} />

<!--Displays a message if there are no sessions recorded (sessions.length > 0)-->
{:else}
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">No Sessions Recorded Yet!</h5>
{/if}