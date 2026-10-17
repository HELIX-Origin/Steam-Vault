import type { UniversalPlugin } from "../framework";

export const ambientSoundtracksPlugin: UniversalPlugin = {
  id: "media-ambient",
  name: "Game Audio & Soundtracks",
  version: "1.0.0",
  pluginType: "media",
  author: "HELIX-Origin",
  description: "Plays ambient background themes and official game soundtracks on selection.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["audio_playback", "ambiance"],
  getSettingsSchema: () => ({
    requiresApiKey: false,
    fields: [
      {
        key: "volume",
        label: "Master Audio Volume %",
        type: "number",
        default: 50,
        description: "Background playback volume for ambient themes."
      },
      {
        key: "loopPlayback",
        label: "Loop Audio Track",
        type: "boolean",
        default: true,
        description: "Seamlessly repeat soundtrack themes while browsing title details."
      }
    ]
  })
};
