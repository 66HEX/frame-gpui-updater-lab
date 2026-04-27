<script lang="ts">
	import { untrack } from 'svelte';
	import { fade } from 'svelte/transition';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { IconPlay, IconPause2 } from '$lib/icons';
	import CropOverlay from './CropOverlay.svelte';
	import CropAspectBar from './CropAspectBar.svelte';
	import PreviewToolbar from './PreviewToolbar.svelte';
	import type {
		PreviewCropController,
		PreviewPlaybackController,
		PreviewRendererController
	} from '$lib/features/preview';

	let {
		filePath,
		mediaKind,
		renderer,
		crop,
		playback,
		controlsDisabled,
		flipHorizontal,
		flipVertical
	}: {
		filePath: string;
		mediaKind: 'video' | 'audio' | 'image';
		renderer: PreviewRendererController;
		crop: PreviewCropController;
		playback: PreviewPlaybackController;
		controlsDisabled: boolean;
		flipHorizontal: boolean;
		flipVertical: boolean;
	} = $props();

	let containerRef = $state<HTMLDivElement | undefined>();
	let wrapperRef = $state<HTMLDivElement | undefined>();
	let canvasRef = $state<HTMLCanvasElement | undefined>();
	let audioRef = $state<HTMLAudioElement | undefined>();
	let isHovering = $state(false);

	const isImage = $derived(mediaKind === 'image');
	const isAudio = $derived(mediaKind === 'audio');
	const audioSrc = $derived(convertFileSrc(filePath));

	$effect(() => {
		untrack(() => {
			renderer.setCanvasElement(canvasRef);
			renderer.setWrapperElement(wrapperRef);
		});
	});

	$effect(() => {
		if (!isAudio) {
			const mediaElement = renderer.mediaElement;
			untrack(() => playback.setMediaElement(mediaElement));
		} else {
			untrack(() => playback.setMediaElement(audioRef));
		}
	});

	$effect(() => {
		if (!containerRef) return;

		const observer = new ResizeObserver((entries) => {
			for (const entry of entries) {
				crop.setContainerSize(entry.contentRect.width, entry.contentRect.height);
			}
		});

		observer.observe(containerRef);
		return () => observer.disconnect();
	});

	$effect(() => {
		if (!wrapperRef) return;

		const updateBounds = () => {
			if (!wrapperRef) return;
			const rect = wrapperRef.getBoundingClientRect();
			crop.setVideoBounds(rect.width, rect.height);
		};

		const observer = new ResizeObserver(updateBounds);
		observer.observe(wrapperRef);
		updateBounds();
		window.addEventListener('resize', updateBounds);

		return () => {
			observer.disconnect();
			window.removeEventListener('resize', updateBounds);
		};
	});
</script>

<div
	class="input-highlight relative flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-md bg-black"
	bind:this={containerRef}
	onclick={() => !isImage && !crop.cropMode && playback.togglePlay()}
	onmouseenter={() => (isHovering = true)}
	onmouseleave={() => (isHovering = false)}
	role="presentation"
>
	{#if isAudio}
		<audio bind:this={audioRef} src={audioSrc} class="hidden"></audio>
	{:else}
		<div
			class="relative inline-flex max-h-full max-w-full overflow-hidden"
			bind:this={wrapperRef}
			style={crop.videoStyle}
		>
			<canvas bind:this={canvasRef} class="block h-full w-full bg-black"></canvas>

			{#if crop.cropMode && crop.draftCrop}
				<CropOverlay
					draftCrop={crop.draftCrop}
					isSideRotation={crop.isSideRotation}
					onBeginCropDrag={crop.beginCropDrag}
				/>
			{/if}
		</div>
	{/if}

	{#if crop.cropMode && crop.draftCrop}
		<CropAspectBar
			cropAspect={crop.cropAspect}
			hasCropDimensions={crop.hasCropDimensions}
			onSelectAspect={crop.selectAspect}
			onReset={crop.resetCropSelection}
			onApply={crop.applyCrop}
		/>
	{/if}

	<PreviewToolbar
		{controlsDisabled}
		{flipHorizontal}
		{flipVertical}
		cropMode={crop.cropMode}
		appliedCrop={crop.appliedCrop}
		hasCropDimensions={crop.hasCropDimensions}
		onRotate={crop.handleRotateToggle}
		onToggleFlip={crop.toggleFlip}
		onToggleCrop={crop.toggleCropMode}
	/>

	{#if !isImage && !crop.cropMode && (!playback.isPlaying || isHovering)}
		<div
			class="absolute inset-0 z-10 flex items-center justify-center"
			onclick={(event) => {
				event.stopPropagation();
				playback.togglePlay();
			}}
			role="presentation"
		>
			<div class="absolute inset-0 bg-background/40" transition:fade={{ duration: 100 }}></div>
			<div
				class="relative flex size-16 items-center justify-center rounded-full bg-frame-gray-200 text-foreground shadow-sm backdrop-blur-md"
				style="transform-origin: center; will-change: opacity; transform: translateZ(0);"
				transition:fade={{ duration: 100 }}
			>
				{#if playback.isPlaying}
					<IconPause2 size={24} />
				{:else}
					<IconPlay size={24} />
				{/if}
			</div>
		</div>
	{/if}
</div>
