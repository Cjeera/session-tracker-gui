<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import Input from "#lib/components/ui/input/input.svelte";
    import Button, { buttonVariants } from "#lib/components/ui/button/button.svelte";
    import Label from "#lib/components/ui/label/label.svelte";

    let { gameId }: { gameId: number } = $props();
    
    let loading = $state(false);
    let error = $state("");
    let processName = $state<string>("");

    let processNameEdited = $derived(processName);
    let editing = $state(false);

    async function getProccesNames() {
        try {
            loading = true;
            processName = await invoke<string>("get_processes", { gameId: gameId });
            loading = false;
        } catch (error) {
            loading = false;
            error = error;
            console.error(error);
        }
    }

    async function insertProcess() {
        try {
            await invoke("insert_process_name", { gameId: gameId, processName: processNameEdited });
            getProccesNames();
            editing = false;
        } catch (error) {
            error = error;
            console.error(error);
        }
    }

    onMount(() => {
        getProccesNames();
    })
</script>

{#if loading}
    <div class="justify-items-center">
        <p>Loading settings...</p>
    </div>
{:else if !loading && !error}
    <Label class="text-lg border-b-2">Process Name</Label>
    <div class="flex items-center max-w-lg p-2 gap-2">
        {#if processName.length > 0}
            {#if editing}
                <Input required type="text" bind:value={processNameEdited} placeholder="Process name..." />
                <Button class="cursor-pointer" variant="outline" onclick={async () => await insertProcess()}>Submit</Button>
                <Button class="cursor-pointer" variant="outline" onclick={() => editing = false}>Cancel</Button>
            {:else}
                <p class="text-balance">{processName[0]}</p>
                <Button class="cursor-pointer" variant="outline" onclick={() => editing = true}>Edit</Button>
            {/if}
        {:else}
            <Input required type="text" class="max-w-lg" bind:value={processNameEdited} placeholder="Process name..." />
            <Button class="cursor-pointer" variant="outline" >Submit</Button>
        {/if}
    </div>
{:else}
    <div class="justify-items-center">
        <p>{error}</p>
    </div>
{/if}
