<script lang="ts">
	import { makeSharedRequest, opGetClusterModules } from "$lib/fetch/ext";
	import { InstanceList } from "$lib/generated/mewld/proc";
	import { CanonicalCommand, CanonicalCommandData, CanonicalModule, CanonicalCommandExtendedData } from "$lib/generated/silverpelt";
	import logger from "$lib/ui/logger";
	import Message from "../Message.svelte";
	import Modal from "../Modal.svelte";
	import NavButton from "../inputs/button/NavButton.svelte";
    import ButtonReact from "../inputs/button/ButtonReact.svelte";
	import InputText from "../inputs/InputText.svelte";
	import GuildClusterLookup from "./GuildClusterLookup.svelte";
	import { Color } from "../inputs/button/colors";
	import { DataHandler, Datatable, Th, ThFilter } from "@vincjo/datatables";
	import { Readable } from "svelte/store";
	import BoolInput from "../inputs/BoolInput.svelte";

    export let instanceList: InstanceList;

    interface State {
        openCluster: number;
        openModule: string;
        commandSearch: string;
        clusterModuleData: Record<number, Record<string, CanonicalModule>>;
        searchedCommands: LookedUpCommand[];
        clusterFinderOpen: boolean;
        clusterFinderByGuildIdExpectedData: {
            cluster: number;
            shard: number;
        } | null;
    }

    let state: State = {
        openCluster: 0,
        openModule: "core",
        clusterModuleData: {},
        commandSearch: "",
        searchedCommands: [],
        clusterFinderOpen: false,
        clusterFinderByGuildIdExpectedData: null
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
            if(module?.web_hidden) continue; // Skip web_hidden modules, they are internal and are not publicly usable anyways

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
    } else {
        state.searchedCommands = [];
    }

    interface ParsedCanonicalCommandData extends CanonicalCommandData {
        subcommand_depth: number;
        parent_command?: CanonicalCommandData;
        extended_data: CanonicalCommandExtendedData;
        search_permissions: string;
    }

    let cmdDataTable: Readable<ParsedCanonicalCommandData[]>;
    const createCmdDataTable = async (_: string) => {
        let module = state.clusterModuleData[state.openCluster][state.openModule];

        let commands: ParsedCanonicalCommandData[] = [];

        // Recursively parse commands
        const parseCommand = (
            command: CanonicalCommandData, 
            extended_data: Record<string, CanonicalCommandExtendedData>,
            depth: number = 0, 
            parent: CanonicalCommandData | undefined,
        ) => {
            let extData = extended_data[depth == 0 ? "" : command?.name] || extended_data[""]
            logger.info("ParseCommand", "Parsing command", command?.name, depth, parent, extData)
            commands.push({
                ...command,
                subcommand_depth: depth,
                parent_command: parent,
                extended_data: extData,
                search_permissions: extData?.default_perms?.checks?.map(check => check?.kittycat_perms)?.join(", ")
            })

            if(command?.subcommands) {
                for(let subcommand of command?.subcommands) {
                    parseCommand(subcommand, extended_data, depth + 1, command)
                }
            }
        }

        for(let command of module?.commands) {
            let extData: Record<string, CanonicalCommandExtendedData> = {};

            for(let commandExtData of command?.extended_data) {
                extData[commandExtData?.id] = commandExtData;
            }

            logger.info("ParseCommand.ExtData", "Got extended data", extData)

            parseCommand(command?.command, extData, 0, undefined)
        }

        const handler = new DataHandler(commands, { rowsPerPage: 20 })

        cmdDataTable = handler.getRows()

        return {
            handler,
            rows: cmdDataTable
        }
    }
</script>

<!--Cluster Menu at the right of the page-->
<article class="command-list-article overflow-x-auto overflow-y-hidden h-full">
    <small class="text-red-600 word-wrap block mb-1">
        Different clusters may have different available modules due to outages, A/B testing and other reasons.
    </small>
    <section class="command-list flex flex-grow">
        <nav class="cluster-map flex-none border-r border-slate-500 w-28">
            {#each instanceList?.Instances as instance}
                <NavButton 
                    current={state.openCluster == instance?.ClusterID} 
                    title={`Cluster ${instance?.ClusterID}`} 
                    onClick={() => {
                        state.openCluster = instance?.ClusterID || 0
                    }}
                    extClass="block mb-2 w-full"
                />
            {/each}
            <NavButton 
                current={false} 
                title={`⚠️ Help`} 
                onClick={() => {
                    state.clusterFinderOpen = true
                    state.clusterFinderByGuildIdExpectedData = null;
                }}
                extClass="block mb-2 w-full"
            />
        </nav>
        <div class="cluster-map-content flex-1 flex-grow px-2">
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

                <!--Module list-->
                <section class="cluster-module-list flex flex-grow">
                    <!--Bar-->
                    <nav class="cluster-map flex-none border-r border-slate-500 w-40">
                        {#each Object.entries(state.clusterModuleData[state?.openCluster]) as [_, module]}
                            {#if !module?.web_hidden}
                                <NavButton 
                                    current={state.openModule == module?.id}
                                    title={module?.name} 
                                    onClick={() => {
                                        state.openModule = module?.id || state.clusterModuleData[state?.openCluster]["core"].id
                                    }}
                                    extClass="block mb-2 w-full"
                                />
                            {/if}
                        {/each}
                    </nav>
                    <!--Content-->
                    <div class="cluster-module-list-content flex-1 flex-grow px-2 mb-auto">
                        {#if state.openModule}
                            <h1 class="text-2xl font-semibold">{state.clusterModuleData[state?.openCluster][state?.openModule]?.name}</h1>
                            <p class="text-slate-200">{state.clusterModuleData[state?.openCluster][state?.openModule]?.description}</p>

                            {#if state.clusterModuleData[state?.openCluster][state?.openModule].configurable}
                                <p class="text-green-500 mt-2">
                                    <strong>This module is CONFIGURABLE</strong>
                                </p>
                            {:else}
                                <p class="text-red-500 mt-2">
                                    <strong>This module is NOT CONFIGURABLE</strong>
                                </p>
                            {/if}

                            {#if state.clusterModuleData[state?.openCluster][state?.openModule].commands_configurable}
                                <p class="text-green-500 mt-2">
                                    <strong>Commands in this module are individually CONFIGURABLE</strong>
                                </p>
                            {:else}
                                <p class="text-red-500 mt-2">
                                    <strong>Commands in this module are NOT individually CONFIGURABLE</strong>
                                </p>
                            {/if}

                            {#if state.clusterModuleData[state?.openCluster][state?.openModule].web_hidden}
                                <p class="text-red-500 mt-2">
                                    <strong>This module is HIDDEN on the website and dashboard</strong>
                                </p>
                            {/if}

                            <BoolInput 
                                id="enabled-by-default"
                                label="Enabled by default"
                                description="Whether this module is enabled by default"
                                disabled={true}
                                value={state.clusterModuleData[state?.openCluster][state?.openModule].is_default_enabled}
                                onChange={() => {}}
                            />

                            {#await createCmdDataTable(state?.openModule)}
                                <Message type="loading">
                                    Loading commands...
                                </Message>
                            {:then data}
                                <Datatable handler={data.handler} search={false}>
                                    <table class="overflow-x-auto">
                                        <thead>
                                            <tr>
                                                <Th handler={data.handler} orderBy={"qualified_name"}>Name</Th>
                                                <Th handler={data.handler} orderBy={"description"}>Description</Th>
                                                <Th handler={data.handler} orderBy={"arguments"}>Arguments</Th>
                                                <Th handler={data.handler} orderBy={"search_permissions"}>Permissions</Th>
                                            </tr>
                                            <tr>
                                                <ThFilter handler={data.handler} filterBy={"qualified_name"} />
                                                <ThFilter handler={data.handler} filterBy={"description"} />
                                                <ThFilter handler={data.handler} filterBy={"arguments"} />
                                                <ThFilter handler={data.handler} filterBy={"search_permissions"} />
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {#each $cmdDataTable as row}
                                                <tr>
                                                    <td>
                                                        {#if row.subcommand_depth == 0}
                                                            <span class="font-semibold">
                                                                {row.name}
                                                            </span>
                                                        {:else}
                                                            <span class="whitespace-nowrap">
                                                                <span class="font-semibold">{row?.parent_command?.name}</span>{" "}<em>{row.name}</em>
                                                            </span>
                                                        {/if}

                                                        <!--NSFW command, TODO: Make tooltip-->
                                                        {#if row.nsfw}
                                                            <div class="command-note">
                                                                <span class="text-red-400 font-semibold">NSFW</span>
                                                            </div>
                                                        {/if}

                                                        <!--Base command of a slash command, TODO: Make tooltip-->
                                                        {#if row.subcommand_required || row.subcommands.length}
                                                            <div class="command-note">
                                                                <span class="text-blue-400 font-semibold">BASE</span>
                                                            </div>
                                                        {/if}
                                                    </td>
                                                    <td>
                                                        {#if row.description}
                                                            {row.description}
                                                        {:else}
                                                            Mystery Box?
                                                        {/if}
                                                    </td>
                                                    <td>
                                                        <ul class="list-disc list-outside">
                                                            {#each row.arguments as arg, i}
                                                                <li class={(i+1) < row.arguments.length ? "mb-2" : ""}>
                                                                    <span class="command-argument">
                                                                        <span class="font-semibold">{arg.name}</span>{#if arg.required}<span class="text-red-400 font-semibold text-lg">*<span class="sr-only">Required parameter)</span></span>{/if}{#if arg.description}: <em>{arg.description}</em>{/if}
                                                                    </span>
                                                                </li>
                                                            {/each}
                                                        </ul>
                                                    </td>
                                                    <td>
                                                        <ul class="list-disc list-outside">
                                                            {#each (row.extended_data?.default_perms?.checks || []) as check}
                                                                <li class="mr-2">
                                                                    <pre class="command-parameter">{check.kittycat_perms}</pre>
                                                                </li>
                                                            {/each}
                                                        </ul>
                                                    </td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>            
                                </Datatable>
                            {:catch}
                                <Message type="error">
                                    Failed to load commands
                                </Message>
                            {/await}
                        {/if}
                    </div>
                </section>
            {/if}
        </div>
    </section>

    <details>
        <summary class="hover:cursor-pointer">Debug</summary>
        <pre>{JSON.stringify(state, null, 4)}</pre>
    </details>

    {#if state.clusterFinderOpen}
        <Modal bind:showModal={state.clusterFinderOpen}>
            <h1 slot="header" class="font-semibold text-2xl">Help</h1>
            <h2 class="text-xl">
                Server Lookup
            </h2>
            <p>
                If you're planning to add AntiRaid to a specific server, please enter the Server's ID below. 

                You can find your Server's ID from either the <em>AntiRaid Dashboard</em> or by <em><a class="text-blue-400 hover:text-blue-600" href="https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID#:~:text=Obtaining%20Server%20IDs%20%2D%20Mobile%20App,name%20and%20select%20Copy%20ID.">following the steps outlined here!</a></em>
            </p>

            <GuildClusterLookup 
                instanceList={instanceList} 
                bind:expectedInfo={state.clusterFinderByGuildIdExpectedData}
            />  

            {#if state.clusterFinderByGuildIdExpectedData}
                <ButtonReact 
                    color={Color.Themable}
                    icon="mdi:forward"
                    text="Take Me There!"
                    onClick={async () => {
                        if(!state.clusterFinderByGuildIdExpectedData) return false;
                        state.openCluster = state.clusterFinderByGuildIdExpectedData.cluster;
                        state.clusterFinderOpen = false;
                        return true
                    }}
                    states={
                        {
                            loading: "Loading...",
                            error: "Failed to find cluster",
                            success: "Found cluster!"
                        }
                    }
                />
            {/if}
        </Modal>
    {/if}
</article>

<style>
    table {
            color: white;
            margin: 0 !important;
    }
    tbody td {
            border: 1px solid #f5f5f5;
            padding: 4px 20px;
    }
    tbody tr {
            transition: all, 0.2s;
    }
    tbody tr:hover {
            background: #252323;
    }
</style>
