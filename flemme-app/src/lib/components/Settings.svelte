<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Tabs, TabsList, TabsTrigger, TabsContent } from "$lib/components/ui/tabs";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";

  // Settings state
  let hotkey = $state("Ctrl+Alt+R");
  let language = $state("fr");
  let autoPaste = $state(true);
  let selectedModel = $state("ggml-small.bin");
  let loading = $state(true);
  let saveStatus = $state("");

  const languages = [
    { value: "fr", label: "Français" },
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
    { value: "de", label: "Deutsch" }
  ];

  const models = [
    { value: "ggml-tiny.bin", label: "Tiny (75 MB)" },
    { value: "ggml-base.bin", label: "Base (142 MB)" },
    { value: "ggml-small.bin", label: "Small (466 MB)" },
    { value: "ggml-medium.bin", label: "Medium (1.5 GB)" }
  ];

  // Load settings on mount
  onMount(async () => {
    try {
      const settings = await invoke("get_settings");
      hotkey = settings.hotkey;
      language = settings.language;
      autoPaste = settings.auto_paste;
      selectedModel = settings.model_name;
      console.log("Settings loaded:", settings);
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      loading = false;
    }
  });

  async function handleSave() {
    try {
      await invoke("save_settings", {
        settings: {
          hotkey,
          language,
          auto_paste: autoPaste,
          model_name: selectedModel
        }
      });
      saveStatus = "Paramètres enregistrés avec succès!";
      setTimeout(() => saveStatus = "", 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      saveStatus = "Erreur lors de l'enregistrement";
      setTimeout(() => saveStatus = "", 3000);
    }
  }
</script>

<div class="settings-container p-6">
  <div class="header mb-6">
    <h1 class="text-3xl font-bold">Paramètres Flemme</h1>
    <p class="text-muted-foreground">Configurez votre application de transcription vocale</p>
  </div>

  <Tabs value="parametres" class="w-full">
    <TabsList class="grid w-full grid-cols-3">
      <TabsTrigger value="parametres">Paramètres</TabsTrigger>
      <TabsTrigger value="modeles">Modèles IA</TabsTrigger>
      <TabsTrigger value="vocabulaire">Vocabulaire</TabsTrigger>
    </TabsList>

    <!-- Tab 1: Paramètres -->
    <TabsContent value="parametres" class="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Raccourci clavier</CardTitle>
          <CardDescription>
            Configurez le raccourci pour démarrer/arrêter l'enregistrement
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <Label for="hotkey">Raccourci actuel</Label>
            <Badge variant="secondary">{hotkey}</Badge>
          </div>
          <Button variant="outline" class="w-full">
            Modifier le raccourci
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Langue de transcription</CardTitle>
          <CardDescription>
            Sélectionnez la langue principale pour la transcription
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Select bind:value={language}>
            <SelectTrigger>
              <SelectValue placeholder="Sélectionnez une langue" />
            </SelectTrigger>
            <SelectContent>
              {#each languages as lang}
                <SelectItem value={lang.value}>{lang.label}</SelectItem>
              {/each}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Collage automatique</CardTitle>
          <CardDescription>
            Coller automatiquement le texte transcrit après l'enregistrement
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex items-center space-x-2">
            <Switch id="auto-paste" bind:checked={autoPaste} />
            <Label for="auto-paste">
              {autoPaste ? "Activé" : "Désactivé"}
            </Label>
          </div>
        </CardContent>
      </Card>

      <Separator />

      <div class="flex justify-between items-center">
        {#if saveStatus}
          <p class="text-sm {saveStatus.includes('succès') ? 'text-green-600' : 'text-red-600'}">
            {saveStatus}
          </p>
        {:else}
          <div></div>
        {/if}
        <Button onclick={handleSave} disabled={loading}>Enregistrer les paramètres</Button>
      </div>
    </TabsContent>

    <!-- Tab 2: Modèles IA -->
    <TabsContent value="modeles" class="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Modèle Whisper</CardTitle>
          <CardDescription>
            Sélectionnez le modèle de transcription à utiliser
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <Select bind:value={selectedModel}>
            <SelectTrigger>
              <SelectValue placeholder="Sélectionnez un modèle" />
            </SelectTrigger>
            <SelectContent>
              {#each models as model}
                <SelectItem value={model.value}>{model.label}</SelectItem>
              {/each}
            </SelectContent>
          </Select>

          <div class="text-sm text-muted-foreground">
            <p><strong>Modèle actuel:</strong> {selectedModel}</p>
            <p class="mt-2">
              Les modèles plus grands offrent une meilleure précision mais nécessitent plus de
              mémoire et de temps de traitement.
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Gestion des modèles</CardTitle>
          <CardDescription>
            Téléchargez ou supprimez des modèles Whisper
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <Button variant="outline" class="w-full">
            Télécharger un nouveau modèle
          </Button>
          <Button variant="outline" class="w-full">
            Gérer les modèles installés
          </Button>
        </CardContent>
      </Card>
    </TabsContent>

    <!-- Tab 3: Vocabulaire -->
    <TabsContent value="vocabulaire" class="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Vocabulaire personnalisé</CardTitle>
          <CardDescription>
            Ajoutez des mots ou expressions pour améliorer la transcription
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="text-sm text-muted-foreground">
            <p>
              Fonctionnalité à venir : vous pourrez ajouter des mots techniques, des noms propres,
              ou du jargon spécifique pour améliorer la précision de la transcription.
            </p>
          </div>
          <Button variant="outline" class="w-full" disabled>
            Ajouter un mot
          </Button>
        </CardContent>
      </Card>
    </TabsContent>
  </Tabs>
</div>

<style>
  .settings-container {
    max-width: 800px;
    margin: 0 auto;
  }
</style>
