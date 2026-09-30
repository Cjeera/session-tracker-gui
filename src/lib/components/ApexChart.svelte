<script lang="ts">
    import { onMount } from "svelte";
    import type ApexCharts from "apexcharts";
    import type { ApexOptions } from "apexcharts";

    let { options }: { options: ApexOptions } = $props();
    let container: HTMLDivElement;
    let chart = $state.raw<ApexCharts | null>(null);
    let error = $state("");

    onMount(() => {
        let disposed = false;
        let instance: ApexCharts | null = null;
        async function mountChart() {
            try {
                const { default: ApexCharts } = await import("apexcharts");
                if (disposed) return;
                instance = new ApexCharts(container, {
                    ...options,
                    theme: { mode: "dark", ...options.theme },
                    chart: { background: "transparent", foreColor: "#9ca3af", ...options.chart },
                });
                await instance.render();
                if (disposed) instance.destroy();
                else chart = instance;
            } catch (cause) {
                if (!disposed) {
                    error = "Unable to display chart.";
                    console.error(cause);
                }
            }
        }
        mountChart();
        return () => {
            disposed = true;
            if (chart) chart.destroy();
            chart = null;
        };
    });

    $effect(() => {
        if (chart) {
            chart.updateOptions({
                ...options,
                theme: { mode: "dark", ...options.theme },
                chart: { background: "transparent", foreColor: "#9ca3af", ...options.chart },
            }).catch((cause) => console.error("Unable to update chart:", cause));
        }
    });
</script>

<div bind:this={container} class="min-h-[300px]"></div>
{#if error}<p class="text-destructive" role="alert">{error}</p>{/if}
