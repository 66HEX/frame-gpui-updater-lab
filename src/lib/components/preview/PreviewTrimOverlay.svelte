<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { PreviewPlaybackController } from '$lib/features/preview';

	let {
		playback,
		disabled,
		cropMode
	}: {
		playback: PreviewPlaybackController;
		disabled: boolean;
		cropMode: boolean;
	} = $props();

	let sliderRef = $state<HTMLDivElement | undefined>();
	let isHovered = $state(false);
	let hoverX = $state(0);

	$effect(() => {
		playback.setSliderElement(sliderRef);
	});

	function updateHoverPosition(event: MouseEvent) {
		if (!sliderRef || disabled || playback.duration <= 0) return;

		const rect = sliderRef.getBoundingClientRect();
		if (rect.width <= 0) return;

		const clampedX = Math.max(0, Math.min(event.clientX - rect.left, rect.width));
		hoverX = clampedX;
	}
</script>

<div
	class={cn(
		'pointer-events-none absolute left-1/2 z-40 w-[min(36rem,calc(100%-5rem))] -translate-x-1/2 transition-[bottom,opacity]',
		cropMode ? 'hidden' : 'bottom-2'
	)}
>
	<div
		class={cn(
			'relative h-8 select-none',
			disabled ? 'pointer-events-none opacity-50' : 'pointer-events-auto cursor-pointer'
		)}
		bind:this={sliderRef}
		role="presentation"
		onclick={(event) => event.stopPropagation()}
		onmouseenter={() => (isHovered = true)}
		onmouseleave={() => (isHovered = false)}
		onmousemove={updateHoverPosition}
		onmousedown={(event) => {
			event.stopPropagation();
			if (!disabled && event.target === sliderRef) {
				playback.seekTo(event);
			}
		}}
	>
		<div
			class="pointer-events-none absolute top-1/2 left-0 h-4 w-full -translate-y-1/2 rounded-[3px] bg-frame-gray-200 shadow-sm backdrop-blur-md"
		></div>

		<div
			class="pointer-events-none absolute top-1/2 left-0 h-4 -translate-y-1/2 overflow-hidden rounded-[3px] bg-frame-gray-100 backdrop-blur-md"
			style={`right: ${100 - playback.toTimelinePercent(playback.endValue)}%; left: ${playback.toTimelinePercent(playback.startValue)}%;`}
		>
			<div class="absolute inset-0 rounded-[3px] bg-frame-gray-600"></div>
		</div>

		<div
			class="pointer-events-none absolute top-1 bottom-1 z-10 w-px -translate-x-1/2 bg-foreground"
			style={`left: ${playback.toTimelinePercent(playback.currentTime)}%`}
		></div>

		<div
			class="absolute top-1/2 z-20 flex h-8 w-6 -translate-x-1/2 -translate-y-1/2 items-center justify-start"
			class:cursor-ew-resize={!disabled}
			style={`left: ${playback.toTimelinePercent(playback.startValue)}%`}
			role="presentation"
			onmousedown={(event) => {
				event.stopPropagation();
				if (!disabled) playback.beginHandleDrag(event, 'start');
			}}
		></div>

		<div
			class="absolute top-1/2 z-20 flex h-8 w-6 -translate-x-1/2 -translate-y-1/2 items-center justify-end"
			class:cursor-ew-resize={!disabled}
			style={`left: ${playback.toTimelinePercent(playback.endValue)}%`}
			role="presentation"
			onmousedown={(event) => {
				event.stopPropagation();
				if (!disabled) playback.beginHandleDrag(event, 'end');
			}}
		></div>

		{#if !disabled && isHovered}
			<div
				class="pointer-events-none absolute top-2 bottom-2 z-10 w-px -translate-x-1/2 bg-frame-gray-600"
				style={`left: ${hoverX}px;`}
			></div>
		{/if}
	</div>
</div>
