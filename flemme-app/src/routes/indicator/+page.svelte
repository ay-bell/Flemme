<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import AudioMotionAnalyzer from 'audiomotion-analyzer';

  let container: HTMLDivElement;
  let audioMotion: AudioMotionAnalyzer | null = null;
  let audioContext: AudioContext | null = null;
  let microphone: MediaStreamAudioSourceNode | null = null;
  let isRecording = $state(false);
  let unlisten: UnlistenFn | null = null;
  let unlistenStop: UnlistenFn | null = null;

  onMount(() => {
    console.log('Indicator window mounted');

    // Create AudioMotion analyzer
    try {
      audioMotion = new AudioMotionAnalyzer(container, {
        mode: 3, // 1/6th octave bands
        gradient: 'prism',
        showBgColor: true,
        bgAlpha: 0.85,
        overlay: true,
        showPeaks: true,
        showScaleX: false,
        showScaleY: false,
        smoothing: 0.7,
        barSpace: 0.3,
        reflexRatio: 0.3,
        reflexAlpha: 0.2,
        ledBars: false,
        lumiBars: false,
        radial: false,
        fillAlpha: 1,
        lineWidth: 0,
        maxFreq: 16000,
        minFreq: 30,
        height: 100,
        width: 500,
      });
      console.log('AudioMotion analyzer created successfully');
    } catch (error) {
      console.error('Failed to create AudioMotion analyzer:', error);
    }

    // Listen for recording start events
    listen('recording-started', async () => {
      console.log('Indicator: Recording started event received');
      isRecording = true;
      await startMicrophone();
    }).then((fn) => {
      unlisten = fn;
      console.log('Recording-started listener registered');
    });

    // Listen for recording stop events
    listen('recording-stopped', () => {
      console.log('Indicator: Recording stopped event received');
      isRecording = false;
      stopMicrophone();
    }).then((fn) => {
      unlistenStop = fn;
      console.log('Recording-stopped listener registered');
    });

    // Cleanup
    return () => {
      console.log('Indicator window cleanup');
      stopMicrophone();
      if (audioMotion) {
        audioMotion.disconnectInput();
      }
      if (unlisten) unlisten();
      if (unlistenStop) unlistenStop();
    };
  });

  async function startMicrophone() {
    try {
      // Request microphone access
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false
        }
      });

      console.log('Microphone stream obtained');

      // Use audioMotion's audioContext
      if (audioMotion && audioMotion.audioCtx) {
        // Create microphone source using audioMotion's context
        microphone = audioMotion.audioCtx.createMediaStreamSource(stream);

        // Connect to audioMotion
        audioMotion.connectInput(microphone);

        // Set volume to 0 to prevent audio feedback
        audioMotion.volume = 0;

        console.log('Microphone connected to analyzer (volume muted)');
      }
    } catch (error) {
      console.error('Failed to access microphone:', error);
    }
  }

  function stopMicrophone() {
    if (microphone) {
      // Disconnect microphone
      if (audioMotion) {
        audioMotion.disconnectInput(microphone);
      }

      // Stop all tracks
      const stream = (microphone.mediaStream as MediaStream);
      if (stream) {
        stream.getTracks().forEach(track => track.stop());
      }

      microphone = null;
      console.log('Microphone disconnected');
    }
  }
</script>

<div class="indicator-container">
  <div class="analyzer-wrapper" class:visible={isRecording}>
    <div bind:this={container} class="audio-motion-container"></div>

    <!-- Recording indicator dot -->
    {#if isRecording}
      <div class="recording-dot"></div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  .indicator-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
  }

  .analyzer-wrapper {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.3s ease, transform 0.3s ease;
  }

  .analyzer-wrapper.visible {
    opacity: 1;
    transform: translateY(0);
  }

  .audio-motion-container {
    width: 500px;
    height: 100px;
  }

  .recording-dot {
    position: absolute;
    top: 15px;
    right: 15px;
    width: 12px;
    height: 12px;
    background: #ef4444;
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.5);
  }

  @keyframes pulse {
    0%, 100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.2);
      opacity: 0.7;
    }
  }

  :global(.audio-motion-container canvas) {
    display: block;
  }
</style>