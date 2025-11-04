<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onHotkeySelected: (hotkey: string) => void;
  }

  let { isOpen = $bindable(false), onClose, onHotkeySelected }: Props = $props();

  let pressedKeys = $state<Set<string>>(new Set());
  let currentHotkey = $state<string>("");
  let errorMessage = $state<string>("");
  let isValidating = $state<boolean>(false);

  // Map of KeyboardEvent.code to display names
  const keyDisplayMap: Record<string, string> = {
    ControlLeft: "Ctrl",
    ControlRight: "Ctrl",
    AltLeft: "Alt",
    AltRight: "Alt",
    ShiftLeft: "Shift",
    ShiftRight: "Shift",
    MetaLeft: "Super",
    MetaRight: "Super",
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    Escape: "Esc",
    Backspace: "Backspace",
    Delete: "Delete",
  };

  function getKeyDisplay(code: string): string {
    if (keyDisplayMap[code]) {
      return keyDisplayMap[code];
    }

    // Handle Key* format (KeyA -> A)
    if (code.startsWith("Key")) {
      return code.substring(3);
    }

    // Handle Digit* format (Digit1 -> 1)
    if (code.startsWith("Digit")) {
      return code.substring(5);
    }

    // Handle F-keys
    if (code.match(/^F\d+$/)) {
      return code;
    }

    // Handle Arrow keys
    if (code.startsWith("Arrow")) {
      return code.substring(5);
    }

    return code;
  }

  function formatHotkeyString(keys: Set<string>): string {
    const modifiers: string[] = [];
    let mainKey = "";

    const modifierOrder = ["Ctrl", "Alt", "Shift", "Super"];

    for (const code of keys) {
      const display = getKeyDisplay(code);

      if (modifierOrder.includes(display)) {
        if (!modifiers.includes(display)) {
          modifiers.push(display);
        }
      } else {
        mainKey = display;
      }
    }

    // Sort modifiers in standard order
    modifiers.sort((a, b) => modifierOrder.indexOf(a) - modifierOrder.indexOf(b));

    if (mainKey) {
      return [...modifiers, mainKey].join("+");
    } else if (modifiers.length > 0) {
      return modifiers.join("+");
    }

    return "";
  }

  function handleKeyDown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    if (event.code === "Escape") {
      onClose();
      return;
    }

    pressedKeys.add(event.code);
    currentHotkey = formatHotkeyString(pressedKeys);
    errorMessage = "";
  }

  async function handleKeyUp(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    const releasedKey = event.code;

    // Check if this is a main key (not a modifier)
    const display = getKeyDisplay(releasedKey);
    const isModifier = ["Ctrl", "Alt", "Shift", "Super"].includes(display);

    // If a main key was released and we have a valid hotkey, validate it
    if (!isModifier && currentHotkey && pressedKeys.size > 0) {
      await validateAndSelectHotkey(currentHotkey);
    }

    // Clear the pressed keys
    pressedKeys.clear();
    currentHotkey = "";
  }

  async function validateAndSelectHotkey(hotkey: string) {
    if (!hotkey) return;

    isValidating = true;
    errorMessage = "";

    try {
      // Test if the hotkey is available
      const isAvailable = await invoke<boolean>("test_hotkey_available", { hotkey });

      if (isAvailable) {
        onHotkeySelected(hotkey);
        onClose();
      } else {
        errorMessage = "Ce raccourci est déjà utilisé par une autre application.";
      }
    } catch (error) {
      errorMessage = `Erreur: ${error}`;
    } finally {
      isValidating = false;
    }
  }

  function resetState() {
    pressedKeys.clear();
    currentHotkey = "";
    errorMessage = "";
    isValidating = false;
  }

  $effect(() => {
    if (isOpen) {
      resetState();
    }
  });
</script>

{#if isOpen}
  <div
    class="modal-overlay fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") onClose();
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <Card class="w-full max-w-md">
      <CardHeader>
        <CardTitle>Configurer le raccourci clavier</CardTitle>
        <CardDescription>
          Appuyez sur la combinaison de touches que vous souhaitez utiliser
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div
          class="capture-area border-2 border-dashed border-primary rounded-lg p-8 text-center focus:outline-none focus:border-solid"
          tabindex="0"
          onkeydown={handleKeyDown}
          onkeyup={handleKeyUp}
          role="textbox"
          aria-label="Zone de capture du raccourci clavier"
        >
          {#if currentHotkey}
            <div class="text-2xl font-bold text-primary">
              {currentHotkey}
            </div>
            <p class="text-sm text-muted-foreground mt-2">
              Relâchez pour valider
            </p>
          {:else}
            <div class="text-muted-foreground">
              <p class="text-lg">En attente...</p>
              <p class="text-sm mt-2">Appuyez sur une combinaison de touches</p>
            </div>
          {/if}
        </div>

        {#if errorMessage}
          <div class="error-message p-3 rounded-lg bg-red-100 text-red-700 border border-red-300">
            <p class="text-sm font-semibold">⚠️ {errorMessage}</p>
          </div>
        {/if}

        {#if isValidating}
          <div class="text-center text-sm text-muted-foreground">
            Validation en cours...
          </div>
        {/if}

        <div class="flex justify-end gap-2">
          <Button variant="outline" onclick={onClose} disabled={isValidating}>
            Annuler
          </Button>
        </div>

        <div class="text-xs text-muted-foreground">
          <p><strong>Astuce :</strong> Utilisez Échap pour annuler</p>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}

<style>
  .modal-overlay {
    animation: fadeIn 0.2s ease-in-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .capture-area {
    min-height: 120px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    cursor: pointer;
    transition: border-color 0.2s;
  }

  .capture-area:focus {
    border-color: hsl(var(--primary));
  }
</style>
