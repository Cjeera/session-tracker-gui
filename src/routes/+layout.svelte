<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from "$app/state";
    import "../app.css";
    import { Button } from "#lib/components/ui/button/index.ts";
    import { Toaster } from "#lib/components/ui/sonner/index.js";
    import { toast } from 'svelte-sonner';
    import { check } from '@tauri-apps/plugin-updater';
    import { ask, message } from '@tauri-apps/plugin-dialog';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { tracker } from "./sessionTracker.svelte.ts";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { Switch } from "#lib/components/ui/switch/index.ts";
    import { Label } from "#lib/components/ui/label/index.js";
    import { autoSearch } from '#lib/types.svelte.ts';
    import type { Process } from '#lib/types.svelte.ts';
    import {
        isPermissionGranted,
        requestPermission,
        sendNotification,
        } from '@tauri-apps/plugin-notification';

    let autoSearchReady = $state(false);
    let changingAutoSearch = $state(false);

    async function autoSearchTrigger(enabled: boolean) {
        if (changingAutoSearch) return;
        changingAutoSearch = true;
        try {
            await invoke("set_auto_search", { enabled });
            autoSearch.autosearchOn = enabled;
        } catch (error) {
            console.error(error);
            toast.error(String(error));
        } finally {
            changingAutoSearch = false;
        }
    }

    let { children } = $props();
    let activeUrl = $derived(page.url.pathname);

    onMount(() => {
        let disposed = false;
        let unlistenClose = () => {};
        let unlistenAutoSearch = () => {};

        async function setupListeners() {
            try {
                const unlisten = await listen<Process>("auto-search-game-found", async (event) => {
                    if (autoSearch.autosearchOn && !changingAutoSearch) {
                        void tracker.trackSession(event.payload);
                    }
                    let permissionGranted = await isPermissionGranted();

                    if (!permissionGranted) {
                        const permission = await requestPermission();
                        permissionGranted = permission === 'granted';
                    }
                    if (permissionGranted) {
                        sendNotification({ title: 'Game Found', body: `Tracking ${tracker.gameInput}` });
                    }
                });
                
                if (disposed) { unlisten(); return; }

                unlistenAutoSearch = unlisten;

                // Synchronise the backend with the default frontend setting.
                await invoke("set_auto_search", {
                    enabled: autoSearch.autosearchOn
                });

                autoSearchReady = true;
            } catch (error) {
                console.error(error);
                toast.error(String(error));
            }
        }
        void setupListeners();
        return () => {
            disposed = true;
            unlistenAutoSearch();
        };
    });

    onMount(async () => {
        try {
            const update = await check();
            if (update) {
                const wantsToUpdate = await ask(
                    `Version ${update.version} is available!\n\nRelease notes:\n${update.body}\n\nWould you like to install it now?`,
                    {
                        title: 'Update Available!',
                        kind: 'info',
                        okLabel: 'Update Now',
                        cancelLabel: 'Later'
                    }
                );
                if (wantsToUpdate) {
                    await update.downloadAndInstall();
                    await message('Update installed! The app will now restart.', { title: 'Success' });
                    await relaunch();
                }
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
        }
    });
</script>
<Toaster />

<header class="border-b bg-card">
    <nav aria-label="Main navigation" class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-3 p-4">
        <a href="/" class="text-xl font-semibold">Session Tracker GUI</a>
        {#if tracker.stopwatchDisplay.length > 0 && activeUrl.includes("/library")}
            <span class="font-mono text-xl tabular-nums" role="timer">{tracker.stopwatchDisplay}</span>
        {/if}
        <div class="flex gap-2 items-center">
            {#if !tracker.gameFound}
                <Label class="cursor-pointer" for="auto-search">Auto Search</Label>
                <Switch class="cursor-pointer" id="auto-search" checked={autoSearch.autosearchOn} disabled={!autoSearchReady || changingAutoSearch} onCheckedChange={autoSearchTrigger}/>
            {/if}
            <Button href="/" variant={activeUrl === "/" ? "secondary" : "ghost"} aria-current={activeUrl === "/" ? "page" : undefined}>Session Tracker</Button>
            <Button href="/stats" variant={activeUrl.startsWith("/stats") ? "secondary" : "ghost"} aria-current={activeUrl.startsWith("/stats") ? "page" : undefined}>Stats</Button>
            <Button href="/library" variant={activeUrl.startsWith("/library") ? "secondary" : "ghost"} aria-current={activeUrl.startsWith("/library") ? "page" : undefined}>Library</Button>
        </div>
    </nav>
</header>

{@render children()}
