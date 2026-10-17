import type { UniversalPlugin } from "../framework";

export const trailersPlayerPlugin: UniversalPlugin = {
  id: "media-trailers",
  name: "Video Trailer Player",
  version: "1.0.0",
  pluginType: "media",
  author: "HELIX-Origin",
  description: "Streams and previews official video trailers in the game detail hero banner.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["video_playback", "trailers"],
  getSettingsSchema: () => ({
    requiresApiKey: false,
    fields: [
      {
        key: "autoPlay",
        label: "Autoplay Video Trailers",
        type: "boolean",
        default: false,
        description: "Automatically stream game trailers upon opening the game details modal."
      },
      {
        key: "muted",
        label: "Start Muted",
        type: "boolean",
        default: true,
        description: "Mute video audio by default when playback begins."
      }
    ]
  })
};
