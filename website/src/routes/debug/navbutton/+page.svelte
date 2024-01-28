<script lang="ts">
	import SideButton from "../../../components/inputs/button/NavButton.svelte";

    let selected = 0;

    $: console.log(selected)
</script>
<SideButton current={selected == 0} title="Dashboard" onClick={() => selected = 0}>
</SideButton>
<SideButton current={selected == 1} title="Cluster 1" onClick={() => selected = 1}>
</SideButton>
<SideButton current={selected == 2} title="Cluster 2" onClick={() => selected = 2}>
</SideButton>