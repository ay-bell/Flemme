<script lang="ts">
  import { onMount } from 'svelte';

  // Props
  let {
    isRecording = $bindable(false),
    audioData = $bindable<number[]>([])
  }: {
    isRecording?: boolean;
    audioData?: number[];
  } = $props();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let animationId: number;

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      canvas.width = 300;
      canvas.height = 80;
      animate();
    }

    return () => {
      if (animationId) {
        cancelAnimationFrame(animationId);
      }
    };
  });

  function animate() {
    if (!ctx || !canvas) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Background
    ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    if (isRecording) {
      // Draw waveform
      const barWidth = 3;
      const barGap = 2;
      const numBars = Math.floor(canvas.width / (barWidth + barGap));
      const centerY = canvas.height / 2;

      ctx.fillStyle = '#3b82f6'; // Blue color

      // If we have real audio data, use it, otherwise use random data for demo
      const data = audioData.length > 0 ? audioData : generateDemoData(numBars);

      for (let i = 0; i < numBars; i++) {
        const dataIndex = Math.floor((i / numBars) * data.length);
        const amplitude = data[dataIndex] || 0;

        // Scale amplitude to fit canvas height (max 70% of height)
        const barHeight = Math.max(2, amplitude * canvas.height * 0.7);

        const x = i * (barWidth + barGap);
        const y = centerY - barHeight / 2;

        ctx.fillRect(x, y, barWidth, barHeight);
      }

      // Recording indicator dot
      const dotRadius = 6;
      const dotX = canvas.width - 20;
      const dotY = 20;

      // Pulsing effect
      const pulseScale = 0.8 + Math.sin(Date.now() / 300) * 0.2;
      ctx.beginPath();
      ctx.arc(dotX, dotY, dotRadius * pulseScale, 0, Math.PI * 2);
      ctx.fillStyle = '#ef4444'; // Red color
      ctx.fill();
    }

    animationId = requestAnimationFrame(animate);
  }

  function generateDemoData(numBars: number): number[] {
    // Generate smooth random wave for demo purposes
    const data: number[] = [];
    const time = Date.now() / 1000;

    for (let i = 0; i < numBars; i++) {
      // Multiple sine waves for more organic look
      const wave1 = Math.sin(time * 2 + i * 0.5) * 0.3;
      const wave2 = Math.sin(time * 3 + i * 0.3) * 0.2;
      const wave3 = Math.sin(time * 5 + i * 0.1) * 0.1;
      const amplitude = Math.abs(wave1 + wave2 + wave3) + 0.1;

      data.push(amplitude);
    }

    return data;
  }
</script>

{#if isRecording}
  <div class="recording-indicator">
    <canvas bind:this={canvas}></canvas>
  </div>
{/if}

<style>
  .recording-indicator {
    position: fixed;
    bottom: 80px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(10px);
  }

  canvas {
    display: block;
  }
</style>