import { convertFileSrc } from '@tauri-apps/api/core';
import { Application, Assets, Container, Sprite, Texture } from 'pixi.js';
import type { CropRect } from '$lib/utils/crop';

type MediaKind = 'video' | 'audio' | 'image';
const MAX_PREVIEW_DPR = 2;

interface PreviewPresentationState {
	mediaKind: MediaKind;
	rotation: '0' | '90' | '180' | '270';
	flipHorizontal: boolean;
	flipVertical: boolean;
	cropMode: boolean;
	appliedCrop: CropRect | null;
	sourceWidth?: number;
	sourceHeight?: number;
}

function getPreviewResolution(): number {
	if (typeof window === 'undefined') return 1;
	return Math.max(1, Math.min(window.devicePixelRatio || 1, MAX_PREVIEW_DPR));
}

export function createPreviewRenderer() {
	let canvasElement = $state<HTMLCanvasElement | undefined>();
	let wrapperElement = $state<HTMLDivElement | undefined>();
	let app = $state<Application | null>(null);
	let appInitPromise: Promise<void> | null = null;
	let spriteContainer = $state<Container | null>(null);
	let rotationContainer = $state<Container | null>(null);
	let flipContainer = $state<Container | null>(null);
	let sprite = $state<Sprite | null>(null);
	let resizeObserver = $state<ResizeObserver | null>(null);
	let texture = $state<Texture | null>(null);
	let mediaElement = $state<HTMLVideoElement | undefined>();
	let frameLoopCleanup: (() => void) | null = null;
	let currentAssetUrl = $state<string | null>(null);
	let sourceRequestId = 0;
	let naturalWidth = $state(0);
	let naturalHeight = $state(0);
	let wrapperWidth = $state(0);
	let wrapperHeight = $state(0);
	let presentation = $state<PreviewPresentationState>({
		mediaKind: 'video',
		rotation: '0',
		flipHorizontal: false,
		flipVertical: false,
		cropMode: false,
		appliedCrop: null
	});

	async function ensureApp() {
		if (!canvasElement || app) return;
		if (appInitPromise) {
			await appInitPromise;
			return;
		}

		appInitPromise = (async () => {
			const nextApp = new Application();
			await nextApp.init({
				canvas: canvasElement,
				width: Math.max(1, wrapperWidth || 1),
				height: Math.max(1, wrapperHeight || 1),
				resolution: getPreviewResolution(),
				autoDensity: true,
				backgroundAlpha: 0,
				antialias: true,
				autoStart: false,
				preference: 'webgpu'
			});

			const nextContainer = new Container();
			const nextRotationContainer = new Container();
			const nextFlipContainer = new Container();
			const nextSprite = new Sprite();
			nextSprite.anchor.set(0.5);
			nextSprite.visible = false;
			nextFlipContainer.addChild(nextSprite);
			nextRotationContainer.addChild(nextFlipContainer);
			nextContainer.addChild(nextRotationContainer);
			nextApp.stage.addChild(nextContainer);

			app = nextApp;
			spriteContainer = nextContainer;
			rotationContainer = nextRotationContainer;
			flipContainer = nextFlipContainer;
			sprite = nextSprite;
			updateScene();
		})();

		try {
			await appInitPromise;
		} finally {
			appInitPromise = null;
		}
	}

	async function clearTexture() {
		frameLoopCleanup?.();
		frameLoopCleanup = null;

		if (mediaElement) {
			mediaElement.pause();
		}

		if (currentAssetUrl) {
			try {
				await Assets.unload(currentAssetUrl);
			} catch {
				// Ignore cache unload issues for local media assets.
			}
		}

		texture?.destroy(false);
		texture = null;
		mediaElement = undefined;
		currentAssetUrl = null;
		naturalWidth = 0;
		naturalHeight = 0;

		if (sprite) {
			sprite.texture = Texture.EMPTY;
			sprite.visible = false;
		}
	}

	function syncNaturalDimensions() {
		if (!texture) return;
		const width = presentation.sourceWidth ?? texture.source.width;
		const height = presentation.sourceHeight ?? texture.source.height;
		naturalWidth = width || 0;
		naturalHeight = height || 0;
		updateScene();
	}

	function bindTexture(nextTexture: Texture) {
		texture = nextTexture;
		if (nextTexture.source.resource instanceof HTMLVideoElement) {
			mediaElement = nextTexture.source.resource;
			mediaElement.muted = false;
			mediaElement.defaultMuted = false;
			mediaElement.volume = 1;
			mediaElement.loop = false;
			mediaElement.autoplay = false;
			mediaElement.playsInline = true;
			attachFrameLoop(mediaElement);
		} else {
			mediaElement = undefined;
		}

		if (sprite) {
			sprite.texture = nextTexture;
			sprite.visible = true;
		}

		syncNaturalDimensions();
	}

	function attachFrameLoop(video: HTMLVideoElement) {
		frameLoopCleanup?.();
		frameLoopCleanup = null;

		if (typeof video.requestVideoFrameCallback === 'function') {
			let callbackId = 0;

			const onFrame = () => {
				if (!app || mediaElement !== video) return;
				app.render();
				callbackId = video.requestVideoFrameCallback(onFrame);
			};

			const start = () => {
				if (callbackId) return;
				callbackId = video.requestVideoFrameCallback(onFrame);
			};

			const stop = () => {
				if (!callbackId) return;
				video.cancelVideoFrameCallback(callbackId);
				callbackId = 0;
			};
			const renderCurrentFrame = () => app?.render();

			video.addEventListener('play', start);
			video.addEventListener('pause', stop);
			video.addEventListener('ended', stop);
			video.addEventListener('seeking', renderCurrentFrame);
			video.addEventListener('seeked', renderCurrentFrame);
			video.addEventListener('timeupdate', renderCurrentFrame);
			video.addEventListener('loadeddata', renderCurrentFrame);

			if (!video.paused) {
				start();
			}

			frameLoopCleanup = () => {
				stop();
				video.removeEventListener('play', start);
				video.removeEventListener('pause', stop);
				video.removeEventListener('ended', stop);
				video.removeEventListener('seeking', renderCurrentFrame);
				video.removeEventListener('seeked', renderCurrentFrame);
				video.removeEventListener('timeupdate', renderCurrentFrame);
				video.removeEventListener('loadeddata', renderCurrentFrame);
			};
			return;
		}

		let rafId = 0;
		const tick = () => {
			if (!app || mediaElement !== video) return;
			app.render();
			if (!video.paused && !video.ended) {
				rafId = window.requestAnimationFrame(tick);
			}
		};
		const start = () => {
			if (rafId) return;
			rafId = window.requestAnimationFrame(tick);
		};
		const stop = () => {
			if (!rafId) return;
			window.cancelAnimationFrame(rafId);
			rafId = 0;
		};
		const renderCurrentFrame = () => app?.render();

		video.addEventListener('play', start);
		video.addEventListener('pause', stop);
		video.addEventListener('ended', stop);
		video.addEventListener('seeking', renderCurrentFrame);
		video.addEventListener('seeked', renderCurrentFrame);
		video.addEventListener('timeupdate', renderCurrentFrame);
		video.addEventListener('loadeddata', renderCurrentFrame);

		if (!video.paused) {
			start();
		}

		frameLoopCleanup = () => {
			stop();
			video.removeEventListener('play', start);
			video.removeEventListener('pause', stop);
			video.removeEventListener('ended', stop);
			video.removeEventListener('seeking', renderCurrentFrame);
			video.removeEventListener('seeked', renderCurrentFrame);
			video.removeEventListener('timeupdate', renderCurrentFrame);
			video.removeEventListener('loadeddata', renderCurrentFrame);
		};
	}

	async function setSource(filePath: string, mediaKind: MediaKind) {
		const requestId = ++sourceRequestId;
		presentation = { ...presentation, mediaKind };

		if (mediaKind === 'audio') {
			await clearTexture();
			return;
		}

		await ensureApp();
		if (requestId !== sourceRequestId) return;

		const assetUrl = convertFileSrc(filePath);
		if (currentAssetUrl === assetUrl && texture) return;

		await clearTexture();
		if (requestId !== sourceRequestId) return;

		const loaded = await Assets.load({
			src: assetUrl,
			data: {
				autoPlay: false,
				muted: false,
				loop: false,
				playsinline: true,
				preload: true
			}
		});
		if (requestId !== sourceRequestId) return;

		if (!(loaded instanceof Texture)) {
			throw new Error('Pixi Assets.load did not return a Texture for preview media');
		}

		currentAssetUrl = assetUrl;
		bindTexture(loaded);
	}

	function setPresentationState(nextPresentation: PreviewPresentationState) {
		presentation = nextPresentation;
		syncNaturalDimensions();
		updateScene();
	}

	function setCanvasElement(element?: HTMLCanvasElement) {
		canvasElement = element;
		void ensureApp();
	}

	function setWrapperElement(element?: HTMLDivElement) {
		if (wrapperElement === element) return;
		resizeObserver?.disconnect();
		wrapperElement = element;

		if (!wrapperElement) return;

		const updateWrapperSize = () => {
			if (!wrapperElement) return;
			const rect = wrapperElement.getBoundingClientRect();
			wrapperWidth = rect.width;
			wrapperHeight = rect.height;
			if (app) {
				app.renderer.resize(Math.max(1, rect.width), Math.max(1, rect.height), getPreviewResolution());
			}
			updateScene();
		};

		resizeObserver = new ResizeObserver(updateWrapperSize);
		resizeObserver.observe(wrapperElement);
		updateWrapperSize();
		void ensureApp();
	}

	function updateScene() {
		if (!app || !spriteContainer || !rotationContainer || !flipContainer || !sprite || !texture)
			return;

		const baseWidth = presentation.sourceWidth ?? naturalWidth;
		const baseHeight = presentation.sourceHeight ?? naturalHeight;
		if (!baseWidth || !baseHeight || !wrapperWidth || !wrapperHeight) return;

		const cropRect =
			!presentation.cropMode && presentation.appliedCrop
				? presentation.appliedCrop
				: { x: 0, y: 0, width: 1, height: 1 };

		const contentWidth = baseWidth * cropRect.width;
		const contentHeight = baseHeight * cropRect.height;
		const sideRotation = presentation.rotation === '90' || presentation.rotation === '270';
		const visualWidth = sideRotation ? contentHeight : contentWidth;
		const visualHeight = sideRotation ? contentWidth : contentHeight;
		const scale = Math.min(wrapperWidth / visualWidth, wrapperHeight / visualHeight);
		const cropCenterX = cropRect.x + cropRect.width / 2 - 0.5;
		const cropCenterY = cropRect.y + cropRect.height / 2 - 0.5;

		sprite.texture = texture;
		sprite.width = baseWidth;
		sprite.height = baseHeight;
		sprite.position.set(-(cropCenterX * baseWidth), -(cropCenterY * baseHeight));
		sprite.visible = true;

		spriteContainer.position.set(wrapperWidth / 2, wrapperHeight / 2);
		spriteContainer.scale.set(scale, scale);
		rotationContainer.rotation = (Number(presentation.rotation) * Math.PI) / 180;
		flipContainer.scale.set(
			presentation.flipHorizontal ? -1 : 1,
			presentation.flipVertical ? -1 : 1
		);

		app.render();
	}

	function destroy() {
		sourceRequestId += 1;
		resizeObserver?.disconnect();
		void clearTexture();
		app?.destroy(true, { children: true, texture: true });
		app = null;
		spriteContainer = null;
		rotationContainer = null;
		flipContainer = null;
		sprite = null;
	}

	return {
		get mediaElement() {
			return mediaElement;
		},
		get naturalWidth() {
			return naturalWidth;
		},
		get naturalHeight() {
			return naturalHeight;
		},
		setCanvasElement,
		setWrapperElement,
		setSource,
		setPresentationState,
		destroy
	};
}

export type PreviewRendererController = ReturnType<typeof createPreviewRenderer>;
