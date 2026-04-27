<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import PreviewViewport from './PreviewViewport.svelte';
	import PreviewTimeline from './PreviewTimeline.svelte';
	import type { ConversionConfig, CropSettings } from '$lib/types';
	import { createPreviewCrop, createPreviewPlayback, createPreviewRenderer } from '$lib/features/preview';

	let {
		filePath,
		mediaKind = 'video',
		initialStartTime,
		initialEndTime,
		rotation = '0',
		flipHorizontal = false,
		flipVertical = false,
		onSave,
		onUpdateConfig,
		controlsDisabled = false,
		initialCrop = null,
		sourceWidth,
		sourceHeight
	}: {
		filePath: string;
		mediaKind?: 'video' | 'audio' | 'image';
		initialStartTime?: string;
		initialEndTime?: string;
		rotation?: ConversionConfig['rotation'];
		flipHorizontal?: boolean;
		flipVertical?: boolean;
		onSave: (start?: string, end?: string) => void;
		onUpdateConfig?: (config: Partial<ConversionConfig>) => void;
		controlsDisabled?: boolean;
		initialCrop?: CropSettings | null;
		sourceWidth?: number;
		sourceHeight?: number;
	} = $props();

	const isImage = $derived(mediaKind === 'image');
	const trimDisabled = $derived(controlsDisabled || isImage);

	const playback = createPreviewPlayback({
		isImage: () => mediaKind === 'image',
		onSave: (start, end) => onSave(start, end)
	});

	const crop = createPreviewCrop({
		getRotation: () => rotation,
		getFlipHorizontal: () => flipHorizontal,
		getFlipVertical: () => flipVertical,
		getSourceWidth: () => sourceWidth,
		getSourceHeight: () => sourceHeight,
		getControlsDisabled: () => controlsDisabled,
		onUpdateConfig: (config) => onUpdateConfig?.(config)
	});

	const renderer = createPreviewRenderer();

	$effect(() => {
		untrack(() => playback.syncInitialValues(initialStartTime, initialEndTime));
	});

	$effect(() => {
		void initialCrop;
		void rotation;
		void flipHorizontal;
		void flipVertical;
		untrack(() => crop.syncInitialCrop(initialCrop));
	});

	$effect(() => {
		untrack(() => crop.setNaturalDimensions(renderer.naturalWidth, renderer.naturalHeight));
	});

	$effect(() => {
		void filePath;
		void mediaKind;
		untrack(() => {
			void renderer.setSource(filePath, mediaKind);
		});
	});

	$effect(() => {
		const nextMediaKind = mediaKind;
		const nextRotation = rotation;
		const nextFlipHorizontal = flipHorizontal;
		const nextFlipVertical = flipVertical;
		const cropMode = crop.cropMode;
		const appliedCrop = crop.appliedCrop;
		const nextSourceWidth = sourceWidth;
		const nextSourceHeight = sourceHeight;
		untrack(() =>
			renderer.setPresentationState({
				mediaKind: nextMediaKind,
				rotation: nextRotation,
				flipHorizontal: nextFlipHorizontal,
				flipVertical: nextFlipVertical,
				cropMode,
				appliedCrop,
				sourceWidth: nextSourceWidth,
				sourceHeight: nextSourceHeight
			})
		);
	});

	onMount(() => {
		return () => {
			playback.destroy();
			crop.destroy();
			renderer.destroy();
		};
	});
</script>

<div class="card-highlight flex h-full flex-col rounded-lg bg-frame-gray-100 p-4 shadow-md">
	<PreviewViewport
		{filePath}
		{mediaKind}
		{renderer}
		{crop}
		{playback}
		{controlsDisabled}
		{flipHorizontal}
		{flipVertical}
	/>
	<PreviewTimeline {playback} {trimDisabled} {isImage} />
</div>
