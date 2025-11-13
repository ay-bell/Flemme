<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import AudioMotionAnalyzer from 'audiomotion-analyzer';
  import { invoke } from '@tauri-apps/api/core';

  let container: HTMLDivElement;
  let audioMotion: AudioMotionAnalyzer | null = null;
  let microphone: MediaStreamAudioSourceNode | null = null;
  let isRecording = $state(false);
  let isTranscribing = $state(false);
  let activeModeName = $state('Standard');
  let activeModelName = $state('Chargement...');
  let unlisten: UnlistenFn | null = null;
  let unlistenStop: UnlistenFn | null = null;
  let unlistenTranscriptionStart: UnlistenFn | null = null;
  let unlistenTranscriptionComplete: UnlistenFn | null = null;

  onMount(async () => {
    console.log('Indicator window mounted');

    // Load mode and model information
    try {
      const [modeName, modelName] = await invoke<[string, string]>('get_indicator_info');
      activeModeName = modeName;
      activeModelName = modelName;
      console.log('Indicator info loaded:', { modeName, modelName });
    } catch (error) {
      console.error('Failed to load indicator info:', error);
      activeModeName = 'Standard';
      activeModelName = 'Erreur';
    }

    // Create AudioMotion analyzer
    try {
      audioMotion = new AudioMotionAnalyzer(container, {
        mode: 5, // 1/2 octave bands (30 bands)
        showBgColor: true,
        bgAlpha: 1, // Fond blanc opaque
        overlay: false,
        showPeaks: false, // Pas de pics
        showScaleX: false,
        showScaleY: false,
        smoothing: 0.7, // Bonne réactivité
        barSpace: 0.4,
        reflexRatio: 0, // Pas de reflets
        ledBars: true, // LED bar effect
        lumiBars: false,
        radial: false,
        fillAlpha: 1,
        lineWidth: 0,
        maxFreq: 16000,
        minFreq: 30,
        height: 64,
        width: 334,
      });

      // Créer et enregistrer le gradient personnalisé vert (du foncé vers le clair)
      audioMotion.registerGradient('customGreen', {
        bgColor: '#ffffff', // Fond blanc
        colorStops: [
          { pos: 0, color: '#2D5F4F' },    // Vert foncé (base)
          { pos: 0.3, color: '#3A7A65' },  // Vert moyen-foncé
          { pos: 0.7, color: '#45997D' },  // Vert moyen
          { pos: 1, color: '#4FB094' }     // Vert de la charte (sommet)
        ]
      });

      // Appliquer le gradient après l'avoir enregistré
      audioMotion.gradient = 'customGreen';

      console.log('AudioMotion analyzer created successfully');
      console.log('Gradient applied:', audioMotion.gradient);
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

    // Listen for transcription start events
    listen('transcription-started', () => {
      console.log('Indicator: Transcription started event received');
      isTranscribing = true;
    }).then((fn) => {
      unlistenTranscriptionStart = fn;
      console.log('Transcription-started listener registered');
    });

    // Listen for transcription completed events
    listen('transcription-completed', () => {
      console.log('Indicator: Transcription completed event received');
      // Hide everything at once to avoid two-step closing animation
      isTranscribing = false;
      isRecording = false;
    }).then((fn) => {
      unlistenTranscriptionComplete = fn;
      console.log('Transcription-completed listener registered');
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
      if (unlistenTranscriptionStart) unlistenTranscriptionStart();
      if (unlistenTranscriptionComplete) unlistenTranscriptionComplete();
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
  <div class="analyzer-wrapper" class:visible={isRecording || isTranscribing}>
    <!-- Audio spectrum visualization (always rendered but hidden when transcribing) -->
    <div class="spectrum-container" class:hidden={isTranscribing}>
      <div bind:this={container} class="audio-motion-container"></div>
      <div class="info-bar">
        <span class="info-left">Mode: {activeModeName}</span>
        <span class="info-right">Modèle: {activeModelName}</span>
      </div>
    </div>

    <!-- Transcription in progress indicator -->
    {#if isTranscribing}
      <div class="transcription-indicator">
        <div class="spinner"></div>
        <span class="transcription-text">Transcription en cours...</span>
      </div>
    {/if}
  </div>
</div>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500&display=swap');

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: #202020;
  }

  .indicator-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #202020;
  }

  .analyzer-wrapper {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.3s ease, transform 0.3s ease;
    padding: 4px;
    background: #202020;
  }

  .analyzer-wrapper.visible {
    opacity: 1;
    transform: translateY(0);
  }

  .spectrum-container {
    display: flex;
    flex-direction: column;
  }

  .spectrum-container.hidden {
    display: none;
  }

  .audio-motion-container {
    width: 334px;
    height: 64px;
    border-radius: 8px 8px 0 0;
    overflow: hidden;
  }

  .info-bar {
    width: 334px;
    height: 30px;
    background: #202020;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 8px;
    box-sizing: border-box;
    border-radius: 0 0 8px 8px;
  }

  .info-left,
  .info-right {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 10px;
    font-weight: 400;
    color: #666666;
  }

  .transcription-indicator {
    width: 334px;
    height: 98px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: #000000;
    border-radius: 8px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(79, 176, 148, 0.2);
    border-top-color: #4FB094;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .transcription-text {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    font-weight: 400;
    color: #666666;
  }

  :global(.audio-motion-container canvas) {
    display: block;
  }
</style>