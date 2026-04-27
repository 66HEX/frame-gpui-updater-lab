<script lang="ts">
	import { getHandleCursor, type CropRect, type DragHandle } from '$lib/utils/crop';

	let {
		draftCrop,
		isSideRotation,
		onBeginCropDrag
	}: {
		draftCrop: CropRect;
		isSideRotation: boolean;
		onBeginCropDrag: (handle: DragHandle, event: MouseEvent) => void;
	} = $props();
</script>

<div class="pointer-events-none absolute inset-0 z-20">
	<div
		class="absolute top-0 left-0 w-full bg-black/55 backdrop-blur-[1px]"
		style={`height: ${draftCrop.y * 100}%;`}
	></div>
	<div
		class="absolute left-0 bg-black/55 backdrop-blur-[1px]"
		style={`top: ${draftCrop.y * 100}%; height: ${draftCrop.height * 100}%; width: ${
			draftCrop.x * 100
		}%;`}
	></div>
	<div
		class="absolute right-0 bg-black/55 backdrop-blur-[1px]"
		style={`top: ${draftCrop.y * 100}%; height: ${draftCrop.height * 100}%; width: ${
			(1 - draftCrop.x - draftCrop.width) * 100
		}%;`}
	></div>
	<div
		class="absolute bottom-0 left-0 w-full bg-black/55 backdrop-blur-[1px]"
		style={`height: ${(1 - draftCrop.y - draftCrop.height) * 100}%;`}
	></div>
</div>

<div
	class="absolute z-30 rounded-md border border-white/90 shadow-[0_0_0_1px_rgba(0,0,0,0.45),0_14px_30px_rgba(0,0,0,0.35)] ring-1 ring-white/20"
	style={`left: ${draftCrop.x * 100}%; top: ${draftCrop.y * 100}%; width: ${
		draftCrop.width * 100
	}%; height: ${draftCrop.height * 100}%;`}
	role="presentation"
	onmousedown={(event) => onBeginCropDrag('move', event)}
>
	{#each [1, 2] as index (index)}
		<div
			class="pointer-events-none absolute left-0 h-px w-full bg-white/70"
			style={`top: ${(index / 3) * 100}%`}
		></div>
		<div
			class="pointer-events-none absolute top-0 h-full w-px bg-white/70"
			style={`left: ${(index / 3) * 100}%`}
		></div>
	{/each}

	{#each [{ id: 'nw', top: 0, left: 0 }, { id: 'n', top: 0, left: 50 }, { id: 'ne', top: 0, left: 100 }, { id: 'e', top: 50, left: 100 }, { id: 'se', top: 100, left: 100 }, { id: 's', top: 100, left: 50 }, { id: 'sw', top: 100, left: 0 }, { id: 'w', top: 50, left: 0 }] as handle (handle.id)}
		<span
			onmousedown={(event) => onBeginCropDrag(handle.id as DragHandle, event)}
			class="absolute block h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full border border-black/45 bg-white shadow-[0_2px_8px_rgba(0,0,0,0.75)]"
			style={`cursor: ${getHandleCursor(handle.id, isSideRotation)}; top: ${handle.top}%; left: ${handle.left}%;`}
			role="presentation"
		></span>
	{/each}
</div>
