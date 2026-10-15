import { createLocale } from "./helper";
import { es } from "./es";

export const es419 = createLocale({
  ...es,
  settings: {
    ...es.settings,
    title: "Configuración",
    languageHelp: "Elige tu idioma preferido. Compatible con los 30 idiomas oficiales de Steam.",
    saveSettings: "Guardar configuración",
    savedSuccess: "Configuración guardada exitosamente."
  }
});
