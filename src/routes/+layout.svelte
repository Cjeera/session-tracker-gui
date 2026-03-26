<script>
    import { onMount } from 'svelte';
    import { page } from "$app/state";
    import { Navbar, NavBrand, NavLi, NavUl, NavHamburger } from "flowbite-svelte";
    
    // Import the Tauri plugins
    import { check } from '@tauri-apps/plugin-updater';
    import { ask, message } from '@tauri-apps/plugin-dialog';
    import { relaunch } from '@tauri-apps/plugin-process';

    let { children } = $props();
    let activeUrl = $derived(page.url.pathname);

    // Run the update check once when the app starts
    onMount(async () => {
        try {
            const update = await check();
            
            if (update) {
                console.log(`Found update ${update.version}`);
                
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
                    // Downloads and installs the update silently in the background
                    await update.downloadAndInstall();
                    
                    // Notify the user and restart
                    await message('Update installed! The app will now restart.', { title: 'Success' });
                    await relaunch();
                }
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
        }
    });
</script>

<Navbar fluid={true}>
    <NavBrand href="/">
        <span class="self-center whitespace-nowrap text-xl font-semibold text-white">Session Tracker GUI</span>
    </NavBrand>
    <NavHamburger/>
    <NavUl {activeUrl}>
        <NavLi href="/">Session Tracker</NavLi>
        <NavLi href="/library">Library</NavLi>
    </NavUl>
</Navbar>

{@render children()}