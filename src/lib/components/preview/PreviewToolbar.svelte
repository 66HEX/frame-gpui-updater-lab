<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import {
		IconCrop as CropIcon,
		IconFlipHorizontal as FlipHorizontalIcon,
		IconFlipVertical as FlipVerticalIcon,
		IconRotateCw
	} from '$lib/icons';
	import type { CropRect } from '$lib/utils/crop';

	let {
		controlsDisabled,
		flipHorizontal,
		flipVertical,
		cropMode,
		appliedCrop,
		hasCropDimensions,
		onRotate,
		onToggleFlip,
		onToggleCrop
	}: {
		controlsDisabled: boolean;
		flipHorizontal: boolean;
		flipVertical: boolean;
		cropMode: boolean;
		appliedCrop: CropRect | null;
		hasCropDimensions: boolean;
		onRotate: () => void;
		onToggleFlip: (axis: 'horizontal' | 'vertical') => void;
		onToggleCrop: () => void;
	} = $props();
</script>

<div
	class="button-highlight pointer-events-auto absolute! top-1/2 left-4 z-40 flex -translate-y-1/2 flex-col gap-2 rounded-md bg-background p-1 shadow-xl"
>
	<Button
		size="icon"
		variant="ghost"
		onclick={(event) => {
			event.stopPropagation();
			onRotate();
		}}
		onmousedown={(event) => event.stopPropagation()}
		disabled={controlsDisabled}
	>
		<IconRotateCw size={16} />
	</Button>
	<Button
		size="icon"
		variant={flipHorizontal ? 'default' : 'ghost'}
		onclick={(event) => {
			event.stopPropagation();
			onToggleFlip('horizontal');
		}}
		onmousedown={(event) => event.stopPropagation()}
		disabled={controlsDisabled}
	>
		<FlipHorizontalIcon size={16} />
	</Button>
	<Button
		size="icon"
		variant={flipVertical ? 'default' : 'ghost'}
		onclick={(event) => {
			event.stopPropagation();
			onToggleFlip('vertical');
		}}
		onmousedown={(event) => event.stopPropagation()}
		disabled={controlsDisabled}
	>
		<FlipVerticalIcon size={16} />
	</Button>
	<Button
		size="icon"
		variant={cropMode ? 'default' : appliedCrop ? 'default' : 'ghost'}
		onclick={(event) => {
			event.stopPropagation();
			onToggleCrop();
		}}
		onmousedown={(event) => event.stopPropagation()}
		disabled={controlsDisabled || !hasCropDimensions}
	>
		<CropIcon size={16} />
	</Button>
</div>
