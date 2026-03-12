<script>
    import { page } from "$app/state";
    import {Navbar, NavBrand, NavLi, NavUl, NavHamburger} from "flowbite-svelte";
    let { children } = $props();
    let activeUrl = $derived(page.url.pathname);
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