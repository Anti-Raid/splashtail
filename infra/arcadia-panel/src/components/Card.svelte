<!--
    AnimatedBox is a box/card element that flips/animates when it becomes visible

	image => The avatar of the card
	display-name => The display name of the card
	short => The short description of the card
	extra => Anything that should be displayed below the card

	Optionals:
	index => The index of the object
	type => The type of the object
	actions => The actions applicable to the object (CardButton, CardLinkButton)
-->

<div class="rounded-lg p-4 bot_card bg-themable-600/50 shadow-white/50 text-white">
	<div class="rounded-lg p-3 bg-black card_banner">
		<div class="flex justify-center items-center">
			<span class="rounded-md w-10 h-10">
				<slot name="image" />
			</span>
			<div class="items-center pl-4 mt-1 text-2xl">
				<span class="inline-block text-2xl font-semibold text-white line-clamp-1 tracking-tight">
					<slot name="display-name" />
				</span>
			</div>
		</div>
	</div>
	<div class="mt-3 rounded-lg bg-black/80 p-3 min-h-[7rem]">
		<p class="break-all text-white">
			<slot name="short" />
		</p>
	</div>

	<div class="block mt-5 text-md tracking-tight my-2">
		{#if $$slots.index || $$slots.type}
			<div class="mt-3 rounded-lg bg-black/80 p-3 text-white">
				{#if $$slots.index}
					<span class="font-extrabold">
						<slot name="index" />
					</span>
				{/if}
				{#if $$slots.type}
					<slot name="type" />
				{/if}
			</div>
		{/if}

		{#if $$slots.actionA}
			{#if $$slots.actionB}
				<div class="flex">
					<slot name="actionA" />
					<div class="ml-3" />
					<slot name="actionB" />
				</div>
			{:else}
				<div class="flex justify-evenly items-center">
					<slot name="actionA" />
				</div>
			{/if}
		{/if}
	</div>

	<slot name="extra" />
</div>
