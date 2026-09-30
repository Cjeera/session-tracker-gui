<script lang="ts">
    import { BarChart, PieChart } from "layerchart/svg";
    import { scaleBand } from "d3-scale";
    import * as Chart from "$lib/components/ui/chart";
    import * as Card from "$lib/components/ui/card";
    import { formatDuration } from "$lib/timeFormatting";
    import { getRecentActivity, getLongestSessions, getWeekdayPlaytime } from "$lib/sessionStats";
    import type { Session } from "$lib/types";
    import { fade } from "svelte/transition";

    let { sessions }: { sessions: Session[] } = $props();

    const recentConfig = {
        sessions: { label: "Sessions", color: "var(--chart-1)" },
    } satisfies Chart.ChartConfig;
    const longestConfig = {
        durationSeconds: { label: "Playtime", color: "var(--chart-2)" },
    } satisfies Chart.ChartConfig;
    const weekdayConfig = {
        sunday: { label: "Sunday", color: "var(--chart-1)" },
        monday: { label: "Monday", color: "var(--chart-2)" },
        tuesday: { label: "Tuesday", color: "var(--chart-3)" },
        wednesday: { label: "Wednesday", color: "var(--chart-4)" },
        thursday: { label: "Thursday", color: "var(--chart-5)" },
        friday: { label: "Friday", color: "var(--chart-6)" },
        saturday: { label: "Saturday", color: "var(--chart-7)" },
    } satisfies Chart.ChartConfig;

    let recentSessions = $derived(getRecentActivity(sessions));
    let longestSessions = $derived(getLongestSessions(sessions));
    let weekdayPlaytime = $derived(getWeekdayPlaytime(sessions));
    let hasPlaytime = $derived(weekdayPlaytime.some((day) => day.seconds > 0));
</script>

{#snippet durationValue({ value, name }: { value: unknown; name: string })}
    <span class="text-muted-foreground">{name}</span>
    <span class="ml-auto font-mono font-medium tabular-nums">{formatDuration(Math.round(Number(value)))}</span>
{/snippet}

<div class="space-y-6" in:fade={{ duration: 75 }}>
    {#if sessions.length === 0}
        <p class="py-8 text-muted-foreground">No sessions recorded yet.</p>
    {:else}
        <Card.Root>
            <Card.Header>
                <Card.Title>Recent Session Frequency</Card.Title>
                <Card.Description>Session counts across the last seven dates with activity.</Card.Description>
            </Card.Header>
            <Card.Content>
                <Chart.Container config={recentConfig} class="aspect-auto h-[300px] w-full" aria-label="Session counts on the seven most recent active days">
                    <BarChart
                        data={recentSessions}
                        x="day"
                        y="sessions"
                        xScale={scaleBand().padding(0.25)}
                        xDomain={recentSessions.map((day) => day.day)}
                        yDomain={[0, null]}
                        yNice
                        series={[{ key: "sessions", label: recentConfig.sessions.label, color: "var(--color-sessions)" }]}
                        props={{
                            xAxis: { format: (day) => recentSessions.find((item) => item.day === String(day))?.date ?? String(day) },
                            yAxis: { format: (count) => Number.isInteger(Number(count)) ? String(count) : "" },
                        }}
                    >
                        {#snippet tooltip({ context })}
                            <Chart.Tooltip label={context.tooltip.data?.date} />
                        {/snippet}
                    </BarChart>
                </Chart.Container>
                <ul class="sr-only">
                    {#each recentSessions as day}<li>{day.date}: {day.sessions} sessions</li>{/each}
                </ul>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header><Card.Title>Longest 5 Sessions</Card.Title></Card.Header>
            <Card.Content>
                <Chart.Container config={longestConfig} class="aspect-auto h-[300px] w-full" aria-label="Five longest sessions by playtime">
                    <BarChart
                        data={longestSessions}
                        orientation="horizontal"
                        x="durationSeconds"
                        y="key"
                        yScale={scaleBand().padding(0.25)}
                        yDomain={longestSessions.map((session) => session.key)}
                        xDomain={[0, null]}
                        axis="y"
                        labels={{ placement: "outside", value: "durationSeconds", format: (value) => formatDuration(Math.round(Number(value))) }}
                        padding={{ left: 85, right: 70 }}
                        series={[{ key: "durationSeconds", label: longestConfig.durationSeconds.label, color: "var(--color-durationSeconds)" }]}
                        props={{
                            yAxis: { format: (key) => longestSessions.find((session) => session.key === String(key))?.date ?? String(key) },
                        }}
                    >
                        {#snippet tooltip({ context })}
                            <Chart.Tooltip label={context.tooltip.data?.date} formatter={durationValue} />
                        {/snippet}
                    </BarChart>
                </Chart.Container>
                <ul class="sr-only">
                    {#each longestSessions as session}<li>{session.date}: {formatDuration(session.durationSeconds)}</li>{/each}
                </ul>
            </Card.Content>
        </Card.Root>

        <Card.Root>
            <Card.Header>
                <Card.Title>Most Common Days Played</Card.Title>
                <Card.Description>Playtime grouped by the day each session started.</Card.Description>
            </Card.Header>
            <Card.Content>
                {#if hasPlaytime}
                    <Chart.Container config={weekdayConfig} class="aspect-auto h-[350px] w-full" aria-label="Playtime by weekday">
                        <PieChart
                            data={weekdayPlaytime}
                            key="key"
                            label="label"
                            value="seconds"
                            legend={{ placement: "bottom", classes: { root: "w-full", items: "justify-center gap-x-4 gap-y-2" } }}
                            padding={{ bottom: 80 }}
                            cRange={weekdayPlaytime.map((day) => weekdayConfig[day.key as keyof typeof weekdayConfig].color)}
                        >
                            {#snippet tooltip({ context })}
                                <Chart.Tooltip label={context.tooltip.data?.label} formatter={durationValue} />
                            {/snippet}
                        </PieChart>
                    </Chart.Container>
                {:else}
                    <p class="py-8 text-muted-foreground">No playtime recorded yet.</p>
                {/if}
                <ul class="sr-only" aria-label="Weekday playtime">
                    {#each weekdayPlaytime as day}<li>{day.label}: {formatDuration(day.seconds)}</li>{/each}
                </ul>
            </Card.Content>
        </Card.Root>
    {/if}
</div>
