import { createLocale } from "./helper";
import { ptBR } from "./pt-BR";

export const pt = createLocale({
  ...ptBR,
  nav: {
    ...ptBR.nav,
    storage: "Armazenamento",
    settings: "Definições"
  },
  settings: {
    ...ptBR.settings,
    title: "Definições",
    languageHelp: "Escolha o seu idioma preferido. Compatível com todos os 30 idiomas oficiais do Steam.",
    saveSettings: "Guardar definições",
    savedSuccess: "Definições guardadas com sucesso."
  }
});
