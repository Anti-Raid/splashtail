<script lang="ts">
	import { makeSharedRequest, opGetClusterModules } from "$lib/fetch/ext";
	import { InstanceList } from "$lib/generated/mewld/proc";
	import { CanonicalCommand, CanonicalModule } from "$lib/generated/silverpelt";
	import logger from "$lib/ui/logger";
	import Message from "../Message.svelte";
	import NavButton from "../NavButton.svelte";
	import InputText from "../inputs/InputText.svelte";

    export let instanceList: InstanceList;

    interface State {
        openCluster: number;
        commandSearch: string;
        clusterModuleData: Record<number, Record<string, CanonicalModule>>;
        searchedCommands: LookedUpCommand[];
    }

    let state: State = {
        openCluster: 0,
        clusterModuleData: {},
        commandSearch: "",
        searchedCommands: []
    }

    interface LookedUpCommand {
        command: CanonicalCommand;
        module: CanonicalModule;
    }
    const commandLookup = (): LookedUpCommand[] => {
        if(state?.openCluster == undefined) return [];
        let moduleData = state.clusterModuleData[state.openCluster];
        if(!moduleData) return [];

        let commands: LookedUpCommand[] = [];

        for(let module of Object.values(moduleData)) {
            for(let command of module?.commands) {
                let checkProps = [
                    command?.command?.name,
                    command?.command?.qualified_name,
                    command?.command?.description,
                    ...command?.command?.subcommands?.map(subcommand => subcommand?.name),
                    ...command?.command?.subcommands?.map(subcommand => subcommand?.qualified_name),
                    ...command?.command?.subcommands?.map(subcommand => subcommand?.description)
                ]

                if(checkProps.some(prop => prop?.toLowerCase()?.includes(state.commandSearch?.toLowerCase()))) {
                    commands.push({
                        command,
                        module
                    })
                }
            }
        }

        return commands;
    }

    const fetchCluster = async (_: number | undefined) => {
        logger.info("FetchCluster", "Fetching cluster modules", state?.openCluster)
        let resp = await makeSharedRequest(opGetClusterModules(state?.openCluster))
        // Save resp to state
        if(!state.clusterModuleData[state?.openCluster || 0]) state.clusterModuleData[state?.openCluster || 0] = resp;
    }

    $: if(state?.commandSearch) {
        state.searchedCommands = commandLookup();
    }
</script>

<!--Cluster Menu at the right of the page-->
<article class="command-list-article">
    <small class="text-red-600 word-wrap block mb-1">
        Different clusters may have different available modules due to outages, A/B testing and other reasons.
    </small>
    <section class="command-list flex flex-grow">
        <nav class="cluster-map flex-none border-r border-slate-500 w-28">
            {#each instanceList?.Instances as instance}
                <NavButton 
                    current={state.openCluster == instance?.ClusterID} 
                    title={`Cluster ${instance?.ClusterID}`} 
                    onClick={() => state.openCluster = instance?.ClusterID || 0}
                    extClass="block mb-2"
                />
            {/each}
        </nav>
        <div class="cluster-map-content flex-1 px-2">
            {#if !state.clusterModuleData[state?.openCluster]}
                {#await fetchCluster(state?.openCluster)}
                    <Message type="loading">
                        Loading cluster modules...
                    </Message>
                {:catch}
                    <Message type="error">
                        Failed to load cluster modules
                    </Message>
                {/await}
            {:else}
                <!--Search bar-->
                <InputText 
                    id="command-search-bar"
                    label="Command Lookup"
                    placeholder="Search for a command"
                    minlength={0}
                    showErrors={false}
                    bind:value={state.commandSearch}
                />
                
                <ul>
                    {#each state.searchedCommands as searchedCommand}
                    <li class="cluster-search-command mb-7">
                        <h3 class="text-xl font-bold">{searchedCommand?.command?.command?.name}</h3>
                        {#if searchedCommand?.command?.command?.description}
                            <p class="text-slate-200">{searchedCommand?.command?.command?.description}</p>
                        {/if}
                        <p class="text-slate-200"><strong>Module:</strong> {searchedCommand?.module?.name}</p>
                    </li>
                    {/each}
                </ul>
            {/if}
        </div>
    </section>

    <details>
        <summary class="hover:cursor-pointer">Debug</summary>
        <pre>{JSON.stringify(state, null, 4)}</pre>
    </details>
</article>